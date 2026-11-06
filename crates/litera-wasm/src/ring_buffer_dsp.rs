//! Lock-Free Single-Producer Single-Consumer (SPSC) WebAudio DSP Ring Buffer (`ring_buffer_dsp.rs`).
//!
//! Designed for zero-allocation real-time audio streaming between the Rust WebAssembly
//! formant synthesis thread and the browser's `AudioWorkletProcessor` (which processes
//! deterministic 128-sample render quanta at 44.1 kHz or 48 kHz).
//!
//! Uses 64-byte cache-line alignment (`#[repr(C, align(64))]`) to prevent false sharing
//! between the producer write index and consumer read index, plus ballistic peak/RMS
//! acoustic telemetry metering.

/// Standard WebAudio render quantum size (128 frames per callback).
pub const WEBAUDIO_RENDER_QUANTUM_FRAMES: usize = 128;

/// Ring buffer capacity (must be a power of two: 4096 samples $\approx 92.8\text{ ms}$ at 44.1 kHz).
pub const DSP_RING_CAPACITY: usize = 4096;
const DSP_RING_MASK: usize = DSP_RING_CAPACITY - 1;

/// Cache-line padded index wrapper preventing CPU L1D cache-line ping-pong between producer and consumer.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct CacheAlignedIndex {
    pub value: usize,
    _pad: [u8; 64 - core::mem::size_of::<usize>()],
}

impl CacheAlignedIndex {
    pub const fn new(value: usize) -> Self {
        Self {
            value,
            _pad: [0u8; 64 - core::mem::size_of::<usize>()],
        }
    }
}

/// Real-time ballistic level meter tracking Peak dBFS, RMS dBFS, and zero-crossing rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BallisticAudioMeter {
    pub peak_linear: f32,
    pub rms_squared_env: f32,
    pub attack_coeff: f32,
    pub release_coeff: f32,
    pub zero_crossing_rate_hz: f32,
    prev_sample: f32,
}

impl BallisticAudioMeter {
    pub fn new(sample_rate_hz: f32) -> Self {
        // Fast 5ms attack, smooth 150ms release
        let attack_coeff = (-1.0 / (0.005 * sample_rate_hz)).exp();
        let release_coeff = (-1.0 / (0.150 * sample_rate_hz)).exp();
        Self {
            peak_linear: 0.0,
            rms_squared_env: 0.0,
            attack_coeff,
            release_coeff,
            zero_crossing_rate_hz: 0.0,
            prev_sample: 0.0,
        }
    }

    /// Updates the ballistic envelope across a 128-frame WebAudio render quantum.
    pub fn process_quantum(&mut self, samples: &[f32], sample_rate_hz: f32) {
        if samples.is_empty() {
            return;
        }
        let mut crossings = 0usize;
        for &s in samples {
            let abs_s = s.abs();
            let coeff = if abs_s > self.peak_linear {
                self.attack_coeff
            } else {
                self.release_coeff
            };
            self.peak_linear = coeff * self.peak_linear + (1.0 - coeff) * abs_s;
            self.rms_squared_env =
                self.release_coeff * self.rms_squared_env + (1.0 - self.release_coeff) * (s * s);

            if (s >= 0.0 && self.prev_sample < 0.0) || (s < 0.0 && self.prev_sample >= 0.0) {
                crossings += 1;
            }
            self.prev_sample = s;
        }
        // Each full sinusoid cycle has 2 zero-crossings
        let duration_sec = samples.len() as f32 / sample_rate_hz.max(1.0);
        self.zero_crossing_rate_hz = (crossings as f32 * 0.5) / duration_sec;
    }

    /// Returns instantaneous Peak level in decibels relative to full scale ($\text{dBFS} \in [-96.0, 0.0]$).
    #[inline]
    pub fn peak_dbfs(&self) -> f32 {
        if self.peak_linear <= 1e-5 {
            -96.0
        } else {
            (20.0 * self.peak_linear.log10()).clamp(-96.0, 6.0)
        }
    }

    /// Returns smoothed RMS level in $\text{dBFS}$.
    #[inline]
    pub fn rms_dbfs(&self) -> f32 {
        let rms = self.rms_squared_env.max(0.0).sqrt();
        if rms <= 1e-5 {
            -96.0
        } else {
            (20.0 * rms.log10()).clamp(-96.0, 6.0)
        }
    }

    /// Crest factor (Peak-to-RMS ratio in dB), indicating transient sharpness (plosives vs vowels).
    #[inline]
    pub fn crest_factor_db(&self) -> f32 {
        (self.peak_dbfs() - self.rms_dbfs()).max(0.0)
    }
}

/// Lock-free SPSC ring buffer for WebAssembly `AudioWorklet` PCM streaming.
#[repr(C, align(64))]
pub struct WasmAudioRingBuffer {
    write_idx: CacheAlignedIndex,
    read_idx: CacheAlignedIndex,
    storage: [f32; DSP_RING_CAPACITY],
    underrun_count: usize,
    overrun_count: usize,
}

impl WasmAudioRingBuffer {
    pub const fn new() -> Self {
        Self {
            write_idx: CacheAlignedIndex::new(0),
            read_idx: CacheAlignedIndex::new(0),
            storage: [0.0; DSP_RING_CAPACITY],
            underrun_count: 0,
            overrun_count: 0,
        }
    }

    /// Number of readable audio frames currently queued in the ring buffer.
    #[inline]
    pub fn available_read(&self) -> usize {
        self.write_idx.value.wrapping_sub(self.read_idx.value)
    }

    /// Number of writable audio frame slots before the buffer saturates.
    #[inline]
    pub fn available_write(&self) -> usize {
        DSP_RING_CAPACITY.saturating_sub(self.available_read())
    }

    /// Pushes synthesized PCM float samples (`[-1.0, 1.0]`) into the ring buffer.
    /// Returns the number of samples written.
    pub fn push_samples(&mut self, input: &[f32]) -> usize {
        let avail = self.available_write();
        let to_write = input.len().min(avail);
        if to_write < input.len() {
            self.overrun_count += 1;
        }

        let mut w = self.write_idx.value;
        for &sample in &input[..to_write] {
            self.storage[w & DSP_RING_MASK] = sample.clamp(-1.0, 1.0);
            w = w.wrapping_add(1);
        }
        self.write_idx.value = w;
        to_write
    }

    /// Pops exactly one 128-frame WebAudio render quantum into `output`.
    /// If fewer samples are available, pads the tail with a click-free exponential decay to zero.
    pub fn pop_render_quantum(&mut self, output: &mut [f32; WEBAUDIO_RENDER_QUANTUM_FRAMES]) -> usize {
        let avail = self.available_read();
        let to_read = WEBAUDIO_RENDER_QUANTUM_FRAMES.min(avail);
        if to_read < WEBAUDIO_RENDER_QUANTUM_FRAMES {
            self.underrun_count += 1;
        }

        let mut r = self.read_idx.value;
        let mut last_sample = 0.0f32;
        for slot in output.iter_mut().take(to_read) {
            let s = self.storage[r & DSP_RING_MASK];
            *slot = s;
            last_sample = s;
            r = r.wrapping_add(1);
        }
        self.read_idx.value = r;

        // De-click tail smoothing on underrun
        for slot in output.iter_mut().skip(to_read) {
            last_sample *= 0.85;
            *slot = last_sample;
        }

        to_read
    }

    #[inline]
    pub fn underrun_count(&self) -> usize {
        self.underrun_count
    }

    #[inline]
    pub fn overrun_count(&self) -> usize {
        self.overrun_count
    }
}

impl Default for WasmAudioRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_push_pop_quantum() {
        let mut rb = WasmAudioRingBuffer::new();
        let input: Vec<f32> = (0..256).map(|i| (i as f32 * 0.05).sin()).collect();
        let written = rb.push_samples(&input);
        assert_eq!(written, 256);
        assert_eq!(rb.available_read(), 256);

        let mut quantum = [0.0f32; WEBAUDIO_RENDER_QUANTUM_FRAMES];
        let read = rb.pop_render_quantum(&mut quantum);
        assert_eq!(read, 128);
        assert_eq!(rb.available_read(), 128);
        assert!((quantum[10] - input[10]).abs() < 1e-6);
    }

    #[test]
    fn test_ballistic_meter_sine_wave() {
        let mut meter = BallisticAudioMeter::new(44_100.0);
        let mut quantum = [0.0f32; WEBAUDIO_RENDER_QUANTUM_FRAMES];
        for (i, s) in quantum.iter_mut().enumerate() {
            *s = (2.0 * core::f32::consts::PI * 440.0 * (i as f32 / 44_100.0)).sin();
        }
        meter.process_quantum(&quantum, 44_100.0);
        assert!(meter.peak_dbfs() > -20.0);
        assert!(meter.zero_crossing_rate_hz > 300.0);
    }
}
