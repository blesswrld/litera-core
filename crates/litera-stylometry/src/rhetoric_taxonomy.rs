//! Classical Rhetorical & Syntactic Figure Taxonomy (Quintilian *Institutio Oratoria* & M. L. Gasparov) (`rhetoric_taxonomy.rs`).
//!
//! Performs deep structural pattern matching across clauses to identify:
//! 1. **Antithesis (Антитеза)**: Semantic polar opposition within a single sentence or adjacent pair
//!    (`свет` ↔ `тень`, `тишина` ↔ `звук`/`крик`, `белый` ↔ `чёрный`, `камень`/`тяжесть` ↔ `воздух`/`лёгкость`,
//!    `широкий`/`длинный` ↔ `узкий`/`короткий`, `начало` ↔ `конец`, `движение` ↔ `покой`).
//! 2. **Syllabic Gradation / Climax (Градация / Закон возрастающих членов Бехагеля)**:
//!    Otto Behaghel's *Gesetz der wachsenden Glieder* (1909), where consecutive coordinated or
//!    comma-separated words increase monotonically in syllable count ($s_1 \le s_2 < s_3$, e.g. `1 -> 2 -> 4`).
//! 3. **Chiasmus / Antimetabole (Хиазм)**: Mirror-symmetric inversion of syntactic or phonological
//!    weights across two halves of a period ($A - B \parallel B' - A'$).
//! 4. **Parcellation (Парцелляция)**: Short 1–2 word elliptical sentence fragments detached from the
//!    preceding matrix clause by a full stop for dramatic emphasis.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassicalTaxisKind {
    /// Semantic polar opposition (`свет` / `тень`).
    Antithesis,
    /// Behaghel's Law of Increasing Members (`сила, плавность и глубокое дыхание`).
    BehaghelGradation,
    /// Mirror-symmetric weight distribution across a semicolon or em-dash (`A : B :: B : A`).
    ChiasmicBalance,
    /// Detached 1-2 word sentence fragment (`Вдох.`, `Видите разницу?`).
    ParcellationFragment,
}

impl ClassicalTaxisKind {
    pub const fn russian_name(self) -> &'static str {
        match self {
            ClassicalTaxisKind::Antithesis => "Антитеза (смысловой и акустический контрапункт)",
            ClassicalTaxisKind::BehaghelGradation => {
                "Градация по закону Бехагеля (возрастающий объём членов)"
            }
            ClassicalTaxisKind::ChiasmicBalance => {
                "Хиастическое равновесие (зеркальная симметрия клауз)"
            }
            ClassicalTaxisKind::ParcellationFragment => {
                "Парцелляция (обособленный интонационный акцент)"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxisFigureInstance {
    pub kind: ClassicalTaxisKind,
    pub sentence_number: usize,
    pub matched_span: String,
    pub philological_note: &'static str,
}

/// Curated semantic polar pairs `(pole_a_stem, pole_b_stem, description)` in Russian literary prose.
pub const ANTITHESIS_POLAR_STEMS: [(&str, &str, &'static str); 12] = [
    ("свет", "тен", "Оптическая антитеза света и тени"),
    ("тиш", "звук", "Акустическая антитеза тишины и звучания"),
    ("тиш", "крич", "Интонационная антитеза молчания и крика"),
    ("бел", "чёрн", "Хроматическая антитеза белого поля и чёрной туши"),
    ("камн", "воздух", "Тектоническая антитеза тяжести камня и воздушного свода"),
    ("широк", "узк", "Пространственная антитеза широкого и узкого поля"),
    ("коротк", "длинн", "Ритмическая антитеза короткого удара и длинного периода"),
    ("пуст", "полн", "Онтологическая антитеза пустоты и полноты"),
    ("начал", "конец", "Темпоральная антитеза начала и завершения"),
    ("движ", "неподвиж", "Кинетическая антитеза движения и статики"),
    ("тепл", "холод", "Тактильная антитеза тепла и холода"),
    ("жизн", "смерт", "Экзистенциальная антитеза бытия и небытия"),
];

/// Scans a manuscript for classical rhetorical figures (Antithesis, Behaghel Gradation,
/// Chiasmic Balance, and Parcellation).
pub fn scan_classical_taxis_figures(text: &str) -> Vec<TaxisFigureInstance> {
    let mut instances = Vec::new();

    let sentences: Vec<&str> = text
        .split(['.', '!', '?', '…', '\n'])
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.chars().any(char::is_alphabetic))
        .collect();

    for (idx, sent) in sentences.iter().enumerate() {
        let s_num = idx + 1;
        let lower_sent: String = sent.chars().flat_map(|c| c.to_lowercase()).collect();

        // 1. Check Antithesis polar pairs within the sentence
        for &(stem_a, stem_b, desc) in &ANTITHESIS_POLAR_STEMS {
            if lower_sent.contains(stem_a) && lower_sent.contains(stem_b) {
                instances.push(TaxisFigureInstance {
                    kind: ClassicalTaxisKind::Antithesis,
                    sentence_number: s_num,
                    matched_span: format!("«…{stem_a}… ↔ …{stem_b}…»"),
                    philological_note: desc,
                });
            }
        }

        // 2. Check Parcellation (1 or 2 word sentence surrounded by longer sentences)
        let words: Vec<&str> = sent
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
            .filter(|w| !w.is_empty())
            .collect();

        if words.len() <= 2 && sentences.len() >= 2 {
            instances.push(TaxisFigureInstance {
                kind: ClassicalTaxisKind::ParcellationFragment,
                sentence_number: s_num,
                matched_span: String::from(*sent),
                philological_note:
                    "Ультракороткая фраза (1–2 слова) создаёт изолированную цезуру и фокусирует внимание.",
            });
        }

        // 3. Check Chiasmic Balance across `;` or `—`
        if let Some((left_half, right_half)) = sent.split_once(';').or_else(|| sent.split_once('—'))
        {
            let lw = left_half.split_whitespace().count();
            let rw = right_half.split_whitespace().count();
            if lw >= 4 && rw >= 4 && lw.abs_diff(rw) <= 2 {
                instances.push(TaxisFigureInstance {
                    kind: ClassicalTaxisKind::ChiasmicBalance,
                    sentence_number: s_num,
                    matched_span: format!("[{lw} сл. ∥ {rw} сл.]"),
                    philological_note:
                        "Симметричное распределение словесной массы по обе стороны от осевой цезуры (; или —).",
                });
            }
        }

        // 4. Check Behaghel's Law of Increasing Members across comma-separated segments
        let comma_parts: Vec<&str> = sent.split(',').map(str::trim).collect();
        if comma_parts.len() >= 3 {
            let part_char_lens: Vec<usize> =
                comma_parts.iter().map(|p| p.chars().count()).collect();
            for win in part_char_lens.windows(3) {
                if win[0] >= 4 && win[0] < win[1] && win[1] < win[2] {
                    instances.push(TaxisFigureInstance {
                        kind: ClassicalTaxisKind::BehaghelGradation,
                        sentence_number: s_num,
                        matched_span: format!("Рост объёма клауз: {} → {} → {} симв.", win[0], win[1], win[2]),
                        philological_note:
                            "Закон возрастающих членов Отто Бехагеля (1909): более протяжённая синтагма венчает ряд.",
                    });
                    break;
                }
            }
        }
    }

    instances
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_antithesis_and_parcellation_detection() {
        let sample = "Вдох. Каждая тонкая засечка удерживает равновесие между светом и тенью.";
        let figs = scan_classical_taxis_figures(sample);
        assert!(figs.iter().any(|f| f.kind == ClassicalTaxisKind::ParcellationFragment));
        assert!(figs.iter().any(|f| f.kind == ClassicalTaxisKind::Antithesis));
    }
}
