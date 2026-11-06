//! Klatt Cascade/Parallel Formant Vowel Synthesizer (`formant_synthesizer.rs`).
//!
//! Models Dennis H. Klatt's (1980) acoustic vocal-tract transfer function
//! $T(f) = \prod_{k=1}^3 H_k(f)$, where each formant $F_1, F_2, F_3$ is a 2-pole digital
//! resonator driven by a Rosenberg glottal pulse source $g(t)$ with fundamental pitch $F_0$:
//! - **$F_1$ (250–850 Hz)**: Pharyngeal cavity volume (inversely proportional to tongue height).
//! - **$F_2$ (750–2400 Hz)**: Oral cavity length (proportional to tongue advancement / palatalization).
//! - **$F_3$ (2100–3100 Hz)**: Lip rounding and retroflexion/rhoticity (`р`, `ж`, `ш`).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FormantTriplet {
    pub vowel: char,
    /// First formant $F_1$ in Hz (jaw opening / vertical tongue height).
    pub f1_hz: f32,
    /// Second formant $F_2$ in Hz (horizontal tongue frontness / palatalization).
    pub f2_hz: f32,
    /// Third formant $F_3$ in Hz (lip rounding & pharyngeal timbre).
    pub f3_hz: f32,
    /// Bandwidths $(B_1, B_2, B_3)$ in Hz at -3 dB points.
    pub bandwidths_hz: (f32, f32, f32),
}

/// Canonical acoustic formant frequencies $(F_1, F_2, F_3)$ for Russian and Cardinal vowels
/// based on Fant (1960, *Acoustic Theory of Speech Production*) and Bondarko (1998).
pub const RUSSIAN_VOWEL_FORMANTS: [FormantTriplet; 10] = [
    FormantTriplet {
        vowel: 'а',
        f1_hz: 720.0,
        f2_hz: 1280.0,
        f3_hz: 2450.0,
        bandwidths_hz: (80.0, 95.0, 120.0),
    },
    FormantTriplet {
        vowel: 'я',
        f1_hz: 680.0,
        f2_hz: 1520.0,
        f3_hz: 2580.0,
        bandwidths_hz: (75.0, 90.0, 120.0),
    },
    FormantTriplet {
        vowel: 'о',
        f1_hz: 510.0,
        f2_hz: 860.0,
        f3_hz: 2380.0,
        bandwidths_hz: (70.0, 85.0, 115.0),
    },
    FormantTriplet {
        vowel: 'ё',
        f1_hz: 490.0,
        f2_hz: 1320.0,
        f3_hz: 2420.0,
        bandwidths_hz: (70.0, 90.0, 115.0),
    },
    FormantTriplet {
        vowel: 'э',
        f1_hz: 540.0,
        f2_hz: 1740.0,
        f3_hz: 2540.0,
        bandwidths_hz: (70.0, 95.0, 120.0),
    },
    FormantTriplet {
        vowel: 'е',
        f1_hz: 440.0,
        f2_hz: 1980.0,
        f3_hz: 2680.0,
        bandwidths_hz: (65.0, 95.0, 125.0),
    },
    FormantTriplet {
        vowel: 'и',
        f1_hz: 280.0,
        f2_hz: 2280.0,
        f3_hz: 3020.0,
        bandwidths_hz: (55.0, 90.0, 130.0),
    },
    FormantTriplet {
        vowel: 'ы',
        f1_hz: 330.0,
        f2_hz: 1560.0,
        f3_hz: 2420.0,
        bandwidths_hz: (60.0, 90.0, 120.0),
    },
    FormantTriplet {
        vowel: 'у',
        f1_hz: 310.0,
        f2_hz: 720.0,
        f3_hz: 2250.0,
        bandwidths_hz: (60.0, 80.0, 110.0),
    },
    FormantTriplet {
        vowel: 'ю',
        f1_hz: 315.0,
        f2_hz: 1180.0,
        f3_hz: 2340.0,
        bandwidths_hz: (60.0, 85.0, 115.0),
    },
];

/// Looks up the Klatt/Fant formant triplet $(F_1, F_2, F_3)$ for a vowel character.
#[inline]
pub fn lookup_vowel_formants(ch: char) -> Option<FormantTriplet> {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    let mapped = match lower {
        'a' => 'а',
        'e' => 'е',
        'i' | 'y' => 'и',
        'o' => 'о',
        'u' => 'у',
        other => other,
    };
    RUSSIAN_VOWEL_FORMANTS
        .iter()
        .copied()
        .find(|f| f.vowel == mapped)
}

/// Digital 2-pole Klatt Formant Resonator:
/// $$y[n] = A \cdot x[n] + B \cdot y[n-1] + C \cdot y[n-2]$$
/// where $C = -\exp(-2\pi B_k T)$, $B = 2 \exp(-\pi B_k T) \cos(2\pi F_k T)$, and $A = 1 - B - C$.
#[derive(Debug, Clone, Copy)]
pub struct KlattFormantResonator {
    coeff_a: f32,
    coeff_b: f32,
    coeff_c: f32,
    y1: f32,
    y2: f32,
}

impl KlattFormantResonator {
    pub fn new(formant_hz: f32, bandwidth_hz: f32, sample_rate_hz: u32) -> Self {
        let t = 1.0 / (sample_rate_hz.max(8000) as f32);
        let pi = core::f32::consts::PI;
        let r = (-pi * bandwidth_hz * t).exp();
        let coeff_c = -(r * r);
        let coeff_b = 2.0 * r * (2.0 * pi * formant_hz * t).cos();
        let coeff_a = 1.0 - coeff_b - coeff_c;
        Self {
            coeff_a,
            coeff_b,
            coeff_c,
            y1: 0.0,
            y2: 0.0,
        }
    }

    #[inline]
    pub fn step(&mut self, x: f32) -> f32 {
        let y = self.coeff_a * x + self.coeff_b * self.y1 + self.coeff_c * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Synthesizes a short Klatt cascade 3-formant vowel grain (`Vec<f32>`) for a given vowel and pitch $F_0$.
pub fn synthesize_klatt_vowel_grain(
    triplet: FormantTriplet,
    pitch_f0_hz: f32,
    duration_samples: usize,
    sample_rate_hz: u32,
) -> Vec<f32> {
    let mut r1 = KlattFormantResonator::new(triplet.f1_hz, triplet.bandwidths_hz.0, sample_rate_hz);
    let mut r2 = KlattFormantResonator::new(triplet.f2_hz, triplet.bandwidths_hz.1, sample_rate_hz);
    let mut r3 = KlattFormantResonator::new(triplet.f3_hz, triplet.bandwidths_hz.2, sample_rate_hz);

    let sr = sample_rate_hz.max(8000) as f32;
    let period_samples = (sr / pitch_f0_hz.max(60.0)).max(8.0);
    let mut out = Vec::with_capacity(duration_samples);

    for n in 0..duration_samples {
        // Rosenberg glottal pulse model: open phase (0..0.40 T), closing phase (0.40..0.56 T)
        let phase = ((n as f32) % period_samples) / period_samples;
        let glottal = if phase < 0.40 {
            let u = phase / 0.40;
            3.0 * u * u - 2.0 * u * u * u
        } else if phase < 0.56 {
            let u = (phase - 0.40) / 0.16;
            1.0 - u * u
        } else {
            0.0
        };

        // Radiation derivative approximation (first-order pre-emphasis)
        let excited = glottal - 0.5;
        let v1 = r1.step(excited);
        let v2 = r2.step(v1);
        let v3 = r3.step(v2);

        // Smooth Hanning window envelope
        let env = 0.5
            * (1.0
                - (2.0 * core::f32::consts::PI * (n as f32) / (duration_samples.max(1) as f32))
                    .cos());
        out.push((v3 * env * 0.35).clamp(-1.0, 1.0));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_klatt_formant_lookup_and_grain() {
        let f_i = lookup_vowel_formants('и').expect("vowel и");
        let f_u = lookup_vowel_formants('у').expect("vowel у");
        // High front 'и' has much higher F2 than high back rounded 'у'
        assert!(f_i.f2_hz > f_u.f2_hz * 2.5);

        let grain = synthesize_klatt_vowel_grain(f_i, 220.0, 256, 16000);
        assert_eq!(grain.len(), 256);
        assert!(grain.iter().all(|x| x.is_finite()));
    }
}
