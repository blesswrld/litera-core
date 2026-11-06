//! Readability & Syntactic Complexity Profiling (Flesch–Kincaid & Oborneva Russian Adaptation).
//!
//! Russian words average ~2.3–2.5 syllables compared to ~1.4–1.6 in English.
//! I. V. Oborneva (2006) calibrated the Flesch Reading Ease formula specifically for
//! Russian morphology:
//! $$\text{FRE}_{\text{RU}} = 206.835 - 1.3 \cdot \text{ASL} - 60.1 \cdot \text{ASW}$$
//! where $\text{ASL}$ is average sentence length in words and $\text{ASW}$ is average syllables per word.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudienceRegister {
    /// Easy, conversational, crystalline prose (FRE >= 70).
    CrystallineAccessible,
    /// Balanced literary and journalistic register (50 <= FRE < 70).
    LiteraryEditorial,
    /// Dense philosophical, academic, or periodic register (30 <= FRE < 50).
    AcademicMonograph,
    /// Highly intricate, baroque, or specialized treatise (FRE < 30).
    DenseTreatise,
}

impl AudienceRegister {
    pub const fn russian_label(self) -> &'static str {
        match self {
            AudienceRegister::CrystallineAccessible => "Прозрачный и ясный строй (Высокая читаемость)",
            AudienceRegister::LiteraryEditorial => "Классический литературно-эссеистический регистр",
            AudienceRegister::AcademicMonograph => "Плотный аналитический или философский стиль",
            AudienceRegister::DenseTreatise => "Сложноподчинённый барочный или академический период",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntacticComplexityClass {
    ParatacticSimple,
    BalancedHypotactic,
    PeriodicPolyphonic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadabilityReport {
    /// Oborneva / Flesch Reading Ease index scaled 0..=100 (higher = easier to read).
    pub flesch_oborneva_index: f32,
    /// Estimated grade/education level required for effortless comprehension.
    pub estimated_grade_level: f32,
    pub average_sentence_length: f32,
    pub average_syllables_per_word: f32,
    /// Proportion of polysyllabic words ($\ge 4$ syllables).
    pub polysyllabic_word_ratio: f32,
    pub register: AudienceRegister,
    pub syntactic_class: SyntacticComplexityClass,
}

/// Computes the Russian/English calibrated readability report from aggregate manuscript counts.
pub fn compute_readability_metrics(
    word_count: usize,
    syllable_count: usize,
    sentence_count: usize,
    polysyllabic_words: usize,
    is_predominantly_cyrillic: bool,
) -> ReadabilityReport {
    if word_count == 0 {
        return ReadabilityReport {
            flesch_oborneva_index: 100.0,
            estimated_grade_level: 1.0,
            average_sentence_length: 0.0,
            average_syllables_per_word: 0.0,
            polysyllabic_word_ratio: 0.0,
            register: AudienceRegister::CrystallineAccessible,
            syntactic_class: SyntacticComplexityClass::ParatacticSimple,
        };
    }

    let asl = (word_count as f32) / (sentence_count.max(1) as f32);
    let asw = (syllable_count as f32) / (word_count as f32);
    let poly_ratio = (polysyllabic_words as f32) / (word_count as f32);

    let fre = if is_predominantly_cyrillic {
        // Oborneva (2006) Russian Flesch adaptation
        (206.835 - 1.3 * asl - 60.1 * asw).clamp(0.0, 100.0)
    } else {
        // Classic Kincaid (1975) English Flesch Reading Ease
        (206.835 - 1.015 * asl - 84.6 * asw).clamp(0.0, 100.0)
    };

    let grade = if is_predominantly_cyrillic {
        (0.5 * asl + 8.4 * asw - 15.59).clamp(1.0, 18.0)
    } else {
        (0.39 * asl + 11.8 * asw - 15.59).clamp(1.0, 18.0)
    };

    let register = if fre >= 70.0 {
        AudienceRegister::CrystallineAccessible
    } else if fre >= 50.0 {
        AudienceRegister::LiteraryEditorial
    } else if fre >= 30.0 {
        AudienceRegister::AcademicMonograph
    } else {
        AudienceRegister::DenseTreatise
    };

    let syntactic_class = if asl <= 9.5 {
        SyntacticComplexityClass::ParatacticSimple
    } else if asl <= 18.0 {
        SyntacticComplexityClass::BalancedHypotactic
    } else {
        SyntacticComplexityClass::PeriodicPolyphonic
    };

    ReadabilityReport {
        flesch_oborneva_index: fre,
        estimated_grade_level: grade,
        average_sentence_length: asl,
        average_syllables_per_word: asw,
        polysyllabic_word_ratio: poly_ratio,
        register,
        syntactic_class,
    }
}
