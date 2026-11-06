//! Syntactic Cadence, Punctuation Caesura Grammar, Parataxis vs. Hypotaxis Index,
//! and Classical Rhetorical Figure Detection (Anaphora, Epiphora, Polysyndeton, Asyndeton).
//!
//! Analyzes how clause boundaries, conjunctions, and punctuation pauses shape the architectural
//! rhythm of a paragraph beyond individual word metrics.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhetoricalFigureKind {
    /// Repetition of the opening word(s) across consecutive sentences or clauses (Анафора).
    Anaphora,
    /// Repetition of the closing word(s) across consecutive sentences or clauses (Эпифора).
    Epiphora,
    /// Deliberate accumulation of coordinating conjunctions `и ... и ... и` (Полисиндетон / Многосоюзие).
    Polysyndeton,
    /// Rapid succession of comma-separated clauses without conjunctions (Асиндетон / Бессоюзие).
    Asyndeton,
    /// Sharp juxtaposition of a long periodic sentence ($\ge 18$ words) with a short point ($\le 5$ words).
    RhythmicCodaContrast,
}

impl RhetoricalFigureKind {
    pub const fn russian_title(self) -> &'static str {
        match self {
            RhetoricalFigureKind::Anaphora => "Анафора (единоначатие периодов)",
            RhetoricalFigureKind::Epiphora => "Эпифора (звуковой возврат в финале фраз)",
            RhetoricalFigureKind::Polysyndeton => "Полисиндетон (многосоюзие, плавное нагнетание)",
            RhetoricalFigureKind::Asyndeton => "Асиндетон (бессоюзие, стремительный перечень)",
            RhetoricalFigureKind::RhythmicCodaContrast => {
                "Ритмический пуант (контраст длинного периода и короткого удара)"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedRhetoricalFigure {
    pub kind: RhetoricalFigureKind,
    pub sentence_index: usize,
    pub excerpt: String,
    pub explanation: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PunctuationCadenceProfile {
    pub periods: usize,
    pub commas: usize,
    pub semicolons: usize,
    pub colons: usize,
    pub em_dashes: usize,
    pub guillemets: usize,
    pub parentheses: usize,
    pub ellipses: usize,
    /// Ratio of subordinating conjunctions (`что`, `чтобы`, `когда`, `если`, `который`, `поскольку`, `хотя`)
    /// to total clauses — higher values denote hypotactic periodic architecture.
    pub hypotaxis_ratio: f32,
    /// Ratio of coordinating or zero-conjunction links (`и`, `а`, `но`, `,`) — higher values denote parataxis.
    pub parataxis_ratio: f32,
    pub detected_figures: Vec<DetectedRhetoricalFigure>,
}

const SUBORDINATORS_RU_EN: [&str; 22] = [
    "что", "чтобы", "когда", "пока", "если", "хотя", "поскольку", "также", "который",
    "которая", "которое", "которые", "которого", "которой", "где", "куда", "откуда",
    "because", "although", "while", "whereas", "which",
];

const COORDINATORS_RU_EN: [&str; 10] = [
    "и", "да", "но", "а", "или", "либо", "зато", "and", "but", "or",
];

/// Analyzes punctuation caesuras, hypotaxis/parataxis balance, and rhetorical figures.
pub fn analyze_syntactic_cadence(text: &str) -> PunctuationCadenceProfile {
    let mut periods = 0usize;
    let mut commas = 0usize;
    let mut semicolons = 0usize;
    let mut colons = 0usize;
    let mut em_dashes = 0usize;
    let mut guillemets = 0usize;
    let mut parentheses = 0usize;
    let mut ellipses = 0usize;

    for ch in text.chars() {
        match ch {
            '.' | '!' | '?' => periods += 1,
            ',' => commas += 1,
            ';' => semicolons += 1,
            ':' => colons += 1,
            '—' | '–' => em_dashes += 1,
            '«' | '»' | '“' | '”' => guillemets += 1,
            '(' | ')' | '[' | ']' => parentheses += 1,
            '…' => ellipses += 1,
            _ => {}
        }
    }

    let mut sub_count = 0usize;
    let mut coord_count = 0usize;

    for tok in text.split_whitespace() {
        let clean: String = tok
            .trim_matches(|c: char| !c.is_alphabetic())
            .chars()
            .flat_map(|c| c.to_lowercase())
            .collect();
        if SUBORDINATORS_RU_EN.contains(&clean.as_str()) {
            sub_count += 1;
        } else if COORDINATORS_RU_EN.contains(&clean.as_str()) {
            coord_count += 1;
        }
    }

    let sentences: Vec<&str> = text
        .split(['.', '!', '?', '…', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.chars().any(char::is_alphabetic))
        .collect();

    let mut detected_figures = Vec::new();
    let mut prev_first_word: Option<String> = None;
    let mut prev_last_word: Option<String> = None;
    let mut prev_word_count: usize = 0;

    for (idx, sent) in sentences.iter().enumerate() {
        let words: Vec<String> = sent
            .split_whitespace()
            .map(|w| {
                w.trim_matches(|c: char| !c.is_alphabetic())
                    .chars()
                    .flat_map(|c| c.to_lowercase())
                    .collect::<String>()
            })
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            continue;
        }

        let first = words.first().cloned().unwrap_or_default();
        let last = words.last().cloned().unwrap_or_default();

        // Check Anaphora
        if let Some(ref pf) = prev_first_word {
            if pf == &first && first.chars().count() >= 2 {
                detected_figures.push(DetectedRhetoricalFigure {
                    kind: RhetoricalFigureKind::Anaphora,
                    sentence_index: idx,
                    excerpt: format!("«{}…»", first),
                    explanation: "Повтор начального слова связывает соседние предложения в единую интонационную волну.",
                });
            }
        }

        // Check Epiphora
        if let Some(ref pl) = prev_last_word {
            if pl == &last && last.chars().count() >= 3 {
                detected_figures.push(DetectedRhetoricalFigure {
                    kind: RhetoricalFigureKind::Epiphora,
                    sentence_index: idx,
                    excerpt: format!("«…{}»", last),
                    explanation: "Совпадение финальных слов замыкает соседние периоды акустической рифмой.",
                });
            }
        }

        // Check Rhythmic Coda Contrast (Legato >= 18 words followed by Staccato <= 4 words)
        if prev_word_count >= 18 && words.len() <= 4 {
            detected_figures.push(DetectedRhetoricalFigure {
                kind: RhetoricalFigureKind::RhythmicCodaContrast,
                sentence_index: idx,
                excerpt: String::from(*sent),
                explanation: "Резкое сокращение длины фразы после развёрнутого периода создаёт мощный смысловой пуант.",
            });
        }

        // Check Polysyndeton (3+ 'и'/'ни'/'или' in a single sentence)
        let conj_in_sent = words
            .iter()
            .filter(|w| matches!(w.as_str(), "и" | "ни" | "или" | "да" | "and" | "nor"))
            .count();
        if conj_in_sent >= 3 {
            detected_figures.push(DetectedRhetoricalFigure {
                kind: RhetoricalFigureKind::Polysyndeton,
                sentence_index: idx,
                excerpt: String::from(*sent),
                explanation: "Многократный повтор союза замедляет темп чтения и придаёт перечислению торжественную весомость.",
            });
        }

        // Check Asyndeton (3+ commas in a sentence with 0 coordinating conjunctions)
        let commas_in_sent = sent.chars().filter(|&c| c == ',').count();
        if commas_in_sent >= 3 && conj_in_sent == 0 {
            detected_figures.push(DetectedRhetoricalFigure {
                kind: RhetoricalFigureKind::Asyndeton,
                sentence_index: idx,
                excerpt: String::from(*sent),
                explanation: "Бессоюзное сцепление однородных членов или клауз ускоряет ритмический пульс фразы.",
            });
        }

        prev_first_word = Some(first);
        prev_last_word = Some(last);
        prev_word_count = words.len();
    }

    let total_links = (sub_count + coord_count + commas).max(1) as f32;
    PunctuationCadenceProfile {
        periods,
        commas,
        semicolons,
        colons,
        em_dashes,
        guillemets,
        parentheses,
        ellipses,
        hypotaxis_ratio: (sub_count as f32 / total_links).clamp(0.0, 1.0),
        parataxis_ratio: ((coord_count + commas) as f32 / total_links).clamp(0.0, 1.0),
        detected_figures,
    }
}
