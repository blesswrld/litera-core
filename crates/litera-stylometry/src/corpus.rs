//! Comparative Stylometric Fingerprinting and Burrows's Delta Author Attribution.
//!
//! Compares an input manuscript's 6-dimensional stylometric feature vector
//! against reference centroids of canonical stylists (Osip Mandelstam, Anton Chekhov,
//! Vladimir Nabokov, Joseph Brodsky, Alexander Pushkin, and Robert Bringhurst's technical prose).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassicalAuthorId {
    Mandelstam,
    Chekhov,
    Nabokov,
    Brodsky,
    Pushkin,
    Bringhurst,
}

impl ClassicalAuthorId {
    pub const fn russian_name(self) -> &'static str {
        match self {
            ClassicalAuthorId::Mandelstam => "Осип Мандельштам («Камень», акмеистическая проза)",
            ClassicalAuthorId::Chekhov => "Антон Чехов (камерная лаконичная проза)",
            ClassicalAuthorId::Nabokov => "Владимир Набоков (развёрнутый метафорический период)",
            ClassicalAuthorId::Brodsky => "Иосиф Бродский (интонационный эссеизм и анжамбеман)",
            ClassicalAuthorId::Pushkin => "Александр Пушкин («Повести Белкина», ясность фразы)",
            ClassicalAuthorId::Bringhurst => "Роберт Брингхёрст (поэтическая типографика)",
        }
    }

    pub const fn stylistic_signature(self) -> &'static str {
        match self {
            ClassicalAuthorId::Mandelstam => {
                "Высокая плотность сонорных и взрывных, архитектурная весомость слова, средний шаг 11–15 слов."
            }
            ClassicalAuthorId::Chekhov => {
                "Прозрачный синтаксис, короткие и средние предложения (8–13 слов), высокий динамический контраст."
            }
            ClassicalAuthorId::Nabokov => {
                "Длинные сложноподчинённые периоды Legato (19–28 слов), богатая аллитерация и редкая лексика."
            }
            ClassicalAuthorId::Brodsky => {
                "Резкая смена регистров, длинные рассуждения с коротким афористичным пуантом в финале абзаца."
            }
            ClassicalAuthorId::Pushkin => {
                "Стремительный глагольный темп Staccato/Moderato (6–11 слов), гармонический баланс гласных."
            }
            ClassicalAuthorId::Bringhurst => {
                "Точный терминологический ритм, чередование тезиса и развёрнутой визуальной метафоры."
            }
        }
    }
}

/// 6-dimensional normalized stylometric feature vector for a manuscript.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StylometricVector {
    /// Mean sentence length in words.
    pub mean_sentence_words: f32,
    /// Sentence length coefficient of variation (std_dev / mean).
    pub sentence_length_cv: f32,
    /// Mean word length in syllables.
    pub mean_word_syllables: f32,
    /// Vowel-to-letter ratio.
    pub vowel_ratio: f32,
    /// Sonorant consonant ratio.
    pub sonorant_ratio: f32,
    /// Plosive consonant ratio.
    pub plosive_ratio: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AuthorCentroid {
    pub id: ClassicalAuthorId,
    pub vector: StylometricVector,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthorProximityMatch {
    pub author: ClassicalAuthorId,
    pub author_name: &'static str,
    pub signature_note: &'static str,
    /// Burrows's Delta ($z$-score normalized Manhattan distance; lower = closer stylistic match).
    pub burrows_delta: f32,
    /// Normalized similarity percentage (0..=100%).
    pub similarity_pct: f32,
}

pub const REFERENCE_CENTROIDS: [AuthorCentroid; 6] = [
    AuthorCentroid {
        id: ClassicalAuthorId::Mandelstam,
        vector: StylometricVector {
            mean_sentence_words: 13.8,
            sentence_length_cv: 0.42,
            mean_word_syllables: 2.48,
            vowel_ratio: 0.425,
            sonorant_ratio: 0.245,
            plosive_ratio: 0.175,
        },
    },
    AuthorCentroid {
        id: ClassicalAuthorId::Chekhov,
        vector: StylometricVector {
            mean_sentence_words: 10.6,
            sentence_length_cv: 0.48,
            mean_word_syllables: 2.22,
            vowel_ratio: 0.440,
            sonorant_ratio: 0.220,
            plosive_ratio: 0.160,
        },
    },
    AuthorCentroid {
        id: ClassicalAuthorId::Nabokov,
        vector: StylometricVector {
            mean_sentence_words: 21.5,
            sentence_length_cv: 0.54,
            mean_word_syllables: 2.62,
            vowel_ratio: 0.432,
            sonorant_ratio: 0.252,
            plosive_ratio: 0.165,
        },
    },
    AuthorCentroid {
        id: ClassicalAuthorId::Brodsky,
        vector: StylometricVector {
            mean_sentence_words: 17.4,
            sentence_length_cv: 0.58,
            mean_word_syllables: 2.55,
            vowel_ratio: 0.418,
            sonorant_ratio: 0.238,
            plosive_ratio: 0.182,
        },
    },
    AuthorCentroid {
        id: ClassicalAuthorId::Pushkin,
        vector: StylometricVector {
            mean_sentence_words: 8.8,
            sentence_length_cv: 0.38,
            mean_word_syllables: 2.15,
            vowel_ratio: 0.448,
            sonorant_ratio: 0.230,
            plosive_ratio: 0.155,
        },
    },
    AuthorCentroid {
        id: ClassicalAuthorId::Bringhurst,
        vector: StylometricVector {
            mean_sentence_words: 12.4,
            sentence_length_cv: 0.45,
            mean_word_syllables: 2.42,
            vowel_ratio: 0.430,
            sonorant_ratio: 0.240,
            plosive_ratio: 0.162,
        },
    },
];

/// Standard deviations across corpus dimensions for Burrows's Delta $z$-score normalization.
const CORPUS_STD_DEVS: StylometricVector = StylometricVector {
    mean_sentence_words: 4.8,
    sentence_length_cv: 0.14,
    mean_word_syllables: 0.25,
    vowel_ratio: 0.025,
    sonorant_ratio: 0.025,
    plosive_ratio: 0.025,
};

/// Computes Burrows's Delta distance $\Delta(A, B) = \frac{1}{D} \sum_{i=1}^D \left| \frac{a_i - b_i}{\sigma_i} \right|$
/// against all canonical author centroids, sorted from closest stylistic match to farthest.
pub fn compute_author_proximities(sample: StylometricVector) -> Vec<AuthorProximityMatch> {
    let mut matches: Vec<AuthorProximityMatch> = REFERENCE_CENTROIDS
        .iter()
        .map(|centroid| {
            let c = centroid.vector;
            let d1 = ((sample.mean_sentence_words - c.mean_sentence_words)
                / CORPUS_STD_DEVS.mean_sentence_words)
                .abs();
            let d2 = ((sample.sentence_length_cv - c.sentence_length_cv)
                / CORPUS_STD_DEVS.sentence_length_cv)
                .abs();
            let d3 = ((sample.mean_word_syllables - c.mean_word_syllables)
                / CORPUS_STD_DEVS.mean_word_syllables)
                .abs();
            let d4 = ((sample.vowel_ratio - c.vowel_ratio) / CORPUS_STD_DEVS.vowel_ratio).abs();
            let d5 = ((sample.sonorant_ratio - c.sonorant_ratio) / CORPUS_STD_DEVS.sonorant_ratio)
                .abs();
            let d6 =
                ((sample.plosive_ratio - c.plosive_ratio) / CORPUS_STD_DEVS.plosive_ratio).abs();

            let delta = (d1 + d2 + d3 + d4 + d5 + d6) / 6.0;
            let similarity = (100.0 * (-0.42 * delta).exp()).clamp(12.0, 99.4);

            AuthorProximityMatch {
                author: centroid.id,
                author_name: centroid.id.russian_name(),
                signature_note: centroid.id.stylistic_signature(),
                burrows_delta: delta,
                similarity_pct: similarity,
            }
        })
        .collect();

    matches.sort_by(|a, b| {
        a.burrows_delta
            .partial_cmp(&b.burrows_delta)
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    matches
}
