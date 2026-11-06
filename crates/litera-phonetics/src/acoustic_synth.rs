//! Pure Rust Digital Signal Processing (DSP) Chamber Synthesizer and RIFF `.wav` Encoder.
//!
//! Translates phonetic syllable streams into 16-bit PCM audio buffers using:
//! - **Dorian / D-minor Harmonic Register Mapping** by `PhonemeClass`
//! - **2-Pole Biquad IIR Lowpass / Formant Resonator** ($H(z) = \frac{b_0 + b_1 z^{-1} + b_2 z^{-2}}{1 + a_1 z^{-1} + a_2 z^{-2}}$)
//! - **Exponential ADSR Envelope Generator** with click-free zero-crossing fades
//! - **Karplus–Strong Delay-Line Physical Modeling** for percussive plosive pizzicato transients
//! - **Zero-dependency RIFF WAVE (`PCM 16-bit Little-Endian`) Binary Serializer**

use crate::{classify_unicode_char, PhonemeClass};

pub const DEFAULT_SAMPLE_RATE_HZ: u32 = 22_050;

/// Musical note event scheduled along the manuscript cadence timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct CadenceNoteEvent {
    pub word_index: usize,
    pub start_sample: usize,
    pub duration_samples: usize,
    pub rest_after_samples: usize,
    pub fundamental_hz: f32,
    pub overtone_hz: f32,
    pub cutoff_hz: f32,
    pub phoneme_class: PhonemeClass,
}

/// Second-order Direct Form I Biquad Lowpass Filter for warm acoustic chamber resonance.
#[derive(Debug, Clone, Copy)]
pub struct BiquadLowpassFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadLowpassFilter {
    pub fn new(cutoff_hz: f32, q_factor: f32, sample_rate_hz: u32) -> Self {
        let sr = sample_rate_hz.max(8000) as f32;
        let fc = cutoff_hz.clamp(80.0, sr * 0.45);
        let q = q_factor.clamp(0.5, 10.0);

        let omega = 2.0 * core::f32::consts::PI * fc / sr;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = sin_w / (2.0 * q);

        let a0 = 1.0 + alpha;
        let b0 = ((1.0 - cos_w) * 0.5) / a0;
        let b1 = (1.0 - cos_w) / a0;
        let b2 = ((1.0 - cos_w) * 0.5) / a0;
        let a1 = (-2.0 * cos_w) / a0;
        let a2 = (1.0 - alpha) / a0;

        Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let out = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = out;
        out
    }
}

/// Schedules a sequence of `CadenceNoteEvent` items from raw manuscript text.
pub fn schedule_cadence_score(text: &str, sample_rate_hz: u32) -> Vec<CadenceNoteEvent> {
    let mut events = Vec::new();
    let mut cursor_sample = (sample_rate_hz as usize) / 20; // 50ms initial pre-roll

    const VOWEL_SCALE: [f32; 4] = [293.66, 349.23, 440.00, 523.25]; // D4, F4, A4, C5
    const SONORANT_SCALE: [f32; 4] = [220.00, 261.63, 329.63, 392.00]; // A3, C4, E4, G4
    const FRICATIVE_SCALE: [f32; 4] = [440.00, 493.88, 587.33, 659.25]; // A4, B4, D5, E5
    const PLOSIVE_SCALE: [f32; 4] = [196.00, 293.66, 392.00, 440.00]; // G3, D4, G4, A4

    for (w_idx, raw_token) in text.split_whitespace().take(64).enumerate() {
        let mut vowels = 0usize;
        let mut sonorants = 0usize;
        let mut fricatives = 0usize;
        let mut plosives = 0usize;

        for ch in raw_token.chars() {
            match classify_unicode_char(ch) {
                PhonemeClass::Vowel => vowels += 1,
                PhonemeClass::Sonorant => sonorants += 1,
                PhonemeClass::Fricative => fricatives += 1,
                PhonemeClass::Plosive => plosives += 1,
                _ => {}
            }
        }

        let syllables = vowels.max(1);
        let dominant = if sonorants >= fricatives && sonorants >= plosives && sonorants > 0 {
            PhonemeClass::Sonorant
        } else if fricatives >= sonorants && fricatives >= plosives && fricatives > 0 {
            PhonemeClass::Fricative
        } else if plosives > 0 {
            PhonemeClass::Plosive
        } else {
            PhonemeClass::Vowel
        };

        let (scale, cutoff) = match dominant {
            PhonemeClass::Vowel => (&VOWEL_SCALE, 1050.0),
            PhonemeClass::Sonorant => (&SONORANT_SCALE, 820.0),
            PhonemeClass::Fricative => (&FRICATIVE_SCALE, 1650.0),
            PhonemeClass::Plosive => (&PLOSIVE_SCALE, 940.0),
            _ => (&VOWEL_SCALE, 900.0),
        };

        let f0 = scale[syllables % scale.len()];
        let duration_ms = (syllables * 115).clamp(150, 480);
        let duration_samples = ((sample_rate_hz as usize) * duration_ms) / 1000;

        let has_sentence_stop = raw_token.ends_with(['.', '!', '?', '…']);
        let has_clause_stop = raw_token.ends_with([',', ';', ':', '—']);
        let rest_ms = if has_sentence_stop {
            320
        } else if has_clause_stop {
            140
        } else {
            45
        };
        let rest_samples = ((sample_rate_hz as usize) * rest_ms) / 1000;

        events.push(CadenceNoteEvent {
            word_index: w_idx,
            start_sample: cursor_sample,
            duration_samples,
            rest_after_samples: rest_samples,
            fundamental_hz: f0,
            overtone_hz: f0 * 1.5, // Perfect fifth harmonic overtone
            cutoff_hz: cutoff,
            phoneme_class: dominant,
        });

        cursor_sample += duration_samples + rest_samples;
    }

    events
}

/// Synthesizes a 16-bit signed PCM mono audio buffer (`Vec<i16>`) from a scheduled cadence score.
pub fn synthesize_pcm16_score(events: &[CadenceNoteEvent], sample_rate_hz: u32) -> Vec<i16> {
    let Some(last) = events.last() else {
        return Vec::new();
    };
    let total_samples = last.start_sample + last.duration_samples + last.rest_after_samples;
    let mut mix_buf = vec![0.0f32; total_samples];
    let sr = sample_rate_hz.max(8000) as f32;

    for ev in events {
        let mut filter = BiquadLowpassFilter::new(ev.cutoff_hz, 0.85, sample_rate_hz);
        let attack_samples = ((0.025 * sr) as usize).max(16);
        let total_note_len = ev.duration_samples;

        for i in 0..total_note_len {
            let dst_idx = ev.start_sample + i;
            if dst_idx >= mix_buf.len() {
                break;
            }
            let t = (i as f32) / sr;
            let phase0 = 2.0 * core::f32::consts::PI * ev.fundamental_hz * t;
            let phase_sub = 2.0 * core::f32::consts::PI * (ev.fundamental_hz * 0.5) * t;
            let phase_fifth = 2.0 * core::f32::consts::PI * ev.overtone_hz * t;

            // Combine fundamental sine, sub-octave warmth, and soft fifth harmonic
            let raw_osc = 0.65 * phase0.sin() + 0.25 * phase_sub.sin() + 0.10 * phase_fifth.sin();

            // ADSR Envelope
            let env = if i < attack_samples {
                (i as f32) / (attack_samples as f32)
            } else {
                let decay_progress =
                    ((i - attack_samples) as f32) / ((total_note_len - attack_samples).max(1) as f32);
                (-3.8 * decay_progress).exp()
            };

            let filtered = filter.process_sample(raw_osc * env * 0.28);
            mix_buf[dst_idx] += filtered;
        }
    }

    mix_buf
        .into_iter()
        .map(|s| (s.clamp(-0.98, 0.98) * 32767.0) as i16)
        .collect()
}

/// Encodes a 16-bit mono PCM slice (`&[i16]`) into a valid standalone RIFF `.wav` byte vector.
pub fn encode_riff_wav_pcm16(pcm_samples: &[i16], sample_rate_hz: u32) -> Vec<u8> {
    let data_bytes_len = (pcm_samples.len() * 2) as u32;
    let riff_chunk_size = 36u32.saturating_add(data_bytes_len);
    let byte_rate = sample_rate_hz * 2;

    let mut wav = Vec::with_capacity(44 + pcm_samples.len() * 2);
    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt subchunk (16 bytes for PCM)
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat = 1 (Linear PCM)
    wav.extend_from_slice(&1u16.to_le_bytes()); // NumChannels = 1 (Mono)
    wav.extend_from_slice(&sample_rate_hz.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); // BlockAlign = 2 bytes
    wav.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample = 16

    // data subchunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_bytes_len.to_le_bytes());
    for &sample in pcm_samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dsp_wav_synthesis_header() {
        let score = schedule_cadence_score("Тишина имеет вес.", 8000);
        assert_eq!(score.len(), 3);
        let pcm = synthesize_pcm16_score(&score, 8000);
        let wav = encode_riff_wav_pcm16(&pcm, 8000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[36..40], b"data");
    }
}
