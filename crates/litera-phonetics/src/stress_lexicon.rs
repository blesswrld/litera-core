//! Andrey Zaliznyak Accentual Paradigms (`a, b, c, d, e, f`) and Literary Russian Stress Lexicon (`stress_lexicon.rs`).
//!
//! In Russian phonology, stress is both **free** (can fall on any syllable) and **mobile**
//! (shifts across inflectional forms within a word's paradigm). Following A. A. Zaliznyak's
//! *Grammatical Dictionary of the Russian Language* (1977), every inflected noun, adjective,
//! and verb belongs to one of six primary accentual paradigms:
//! - **Paradigm `a`**: Fixed stress on the stem throughout all singular and plural cases (`кни́га`, `ти́хий`, `бума́га`, `засе́чка`).
//! - **Paradigm `b`**: Fixed stress on the inflectional ending (or final stem syllable when ending is zero: `стол` -> `стола́`, `тишина́`, `строка́`).
//! - **Paradigm `c`**: Stem stress in singular (`го́лос`, `сло́во`, `мо́ре`, `о́блако`), shifting to the ending in plural (`голоса́`, `слова́`, `моря́`, `облака́`).
//! - **Paradigm `d`**: Ending stress in singular (`душа́`, `вода́`, `земля́`, `окно́`), shifting to the stem in plural (`ду́ши`, `во́ды`, `зе́мли`, `о́кна`).
//! - **Paradigm `e`**: Stem stress in singular and nominative plural, shifting to the ending in oblique plural (`ка́мень` -> `ка́мни`, `камне́й`).
//! - **Paradigm `f`**: Ending stress throughout except nominative/accusative plural (`голова́` -> `го́ловы`, `голо́в`).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZaliznyakParadigm {
    /// `a`: Неподвижное ударение на основе во всех формах (`бума́га`, `архитекту́ра`).
    FixedStemA,
    /// `b`: Неподвижное ударение на окончании (`тишина́`, `чернила́` / `стол` -> `стола́`).
    FixedEndingB,
    /// `c`: Ударение на основе в ед. ч., на окончании во мн. ч. (`го́лос` -> `голоса́`, `сло́во` -> `слова́`).
    MobileSingStemPluralEndC,
    /// `d`: Ударение на окончании в ед. ч., на основе во мн. ч. (`вода́` -> `во́ды`, `окно́` -> `о́кна`).
    MobileSingEndPluralStemD,
    /// `e`: Ударение на основе в ед. ч. и И. п. мн. ч., на окончании в косвенных мн. ч. (`ка́мень` -> `камне́й`).
    MobileObliquePluralE,
    /// `f`: Подвижное ударение с переносом на первый слог основы в И./В. п. (`голова́` -> `го́ловы` / `во́лны`).
    MobileInitialPluralF,
}

impl ZaliznyakParadigm {
    pub const fn code(self) -> &'static str {
        match self {
            ZaliznyakParadigm::FixedStemA => "a (неподвижное на основе)",
            ZaliznyakParadigm::FixedEndingB => "b (неподвижное на окончании)",
            ZaliznyakParadigm::MobileSingStemPluralEndC => "c (основа ед. ч. → окончание мн. ч.)",
            ZaliznyakParadigm::MobileSingEndPluralStemD => "d (окончание ед. ч. → основа мн. ч.)",
            ZaliznyakParadigm::MobileObliquePluralE => "e (сдвиг в косвенных падежах мн. ч.)",
            ZaliznyakParadigm::MobileInitialPluralF => "f (подвижное с ретракцией на корень)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexiconStressEntry {
    pub word_form: &'static str,
    pub lemma: &'static str,
    /// 1-based syllable index carrying primary stress (`1` = first syllable).
    pub stressed_syllable_1based: u8,
    pub total_syllables: u8,
    pub paradigm: ZaliznyakParadigm,
}

/// Curated high-frequency literary & typographic Russian lexicon with exact Zaliznyak paradigms
/// and verified 1-based stressed syllable indices.
pub const LITERARY_STRESS_LEXICON: [LexiconStressEntry; 72] = [
    LexiconStressEntry { word_form: "тишина", lemma: "тишина", stressed_syllable_1based: 3, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "тишины", lemma: "тишина", stressed_syllable_1based: 3, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "тишине", lemma: "тишина", stressed_syllable_1based: 3, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "имеет", lemma: "иметь", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "собственный", lemma: "собственный", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "когда", lemma: "когда", stressed_syllable_1based: 2, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "скользит", lemma: "скользить", stressed_syllable_1based: 2, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "широкому", lemma: "широкий", stressed_syllable_1based: 2, total_syllables: 4, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "широкую", lemma: "широкий", stressed_syllable_1based: 2, total_syllables: 4, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "ширину", lemma: "ширина", stressed_syllable_1based: 3, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "полю", lemma: "поле", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileSingStemPluralEndC },
    LexiconStressEntry { word_form: "поле", lemma: "поле", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileSingStemPluralEndC },
    LexiconStressEntry { word_form: "полях", lemma: "поле", stressed_syllable_1based: 2, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileSingStemPluralEndC },
    LexiconStressEntry { word_form: "тёплой", lemma: "тёплый", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "архивной", lemma: "архивный", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "бумаги", lemma: "бумага", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "бумаге", lemma: "бумага", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "каждая", lemma: "каждый", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "каждом", lemma: "каждый", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "тонкая", lemma: "тонкий", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "тонкий", lemma: "тонкий", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "засечка", lemma: "засечка", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "удерживает", lemma: "удерживать", stressed_syllable_1based: 2, total_syllables: 5, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "равновесие", lemma: "равновесие", stressed_syllable_1based: 3, total_syllables: 5, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "между", lemma: "между", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "светом", lemma: "свет", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "тенью", lemma: "тень", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileObliquePluralE },
    LexiconStressEntry { word_form: "хороший", lemma: "хороший", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "кричит", lemma: "кричать", stressed_syllable_1based: 2, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "себе", lemma: "себя", stressed_syllable_1based: 2, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "требует", lemma: "требовать", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "внимания", lemma: "внимание", stressed_syllable_1based: 2, total_syllables: 4, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "становится", lemma: "становиться", stressed_syllable_1based: 2, total_syllables: 4, paradigm: ZaliznyakParadigm::MobileSingEndPluralStemD },
    LexiconStressEntry { word_form: "прозрачным", lemma: "прозрачный", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "сосудом", lemma: "сосуд", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "голоса", lemma: "голос", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::MobileSingStemPluralEndC },
    LexiconStressEntry { word_form: "сохраняя", lemma: "сохранять", stressed_syllable_1based: 3, total_syllables: 4, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "дыхание", lemma: "дыхание", stressed_syllable_1based: 2, total_syllables: 4, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "словами", lemma: "слово", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::MobileSingStemPluralEndC },
    LexiconStressEntry { word_form: "слова", lemma: "слово", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileSingStemPluralEndC },
    LexiconStressEntry { word_form: "слово", lemma: "слово", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileSingStemPluralEndC },
    LexiconStressEntry { word_form: "паузу", lemma: "пауза", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "после", lemma: "после", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "точки", lemma: "точка", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "мерный", lemma: "мерный", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "согласных", lemma: "согласный", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "слушайте", lemma: "слушать", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "музыку", lemma: "музыка", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "кружевом", lemma: "кружево", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::MobileSingStemPluralEndC },
    LexiconStressEntry { word_form: "камнем", lemma: "камень", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileObliquePluralE },
    LexiconStressEntry { word_form: "камень", lemma: "камень", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileObliquePluralE },
    LexiconStressEntry { word_form: "воздухом", lemma: "воздух", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "держится", lemma: "держаться", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::MobileSingEndPluralStemD },
    LexiconStressEntry { word_form: "высокий", lemma: "высокий", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "готический", lemma: "готический", stressed_syllable_1based: 2, total_syllables: 4, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "летаем", lemma: "летать", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "поднимаемся", lemma: "подниматься", stressed_syllable_1based: 3, total_syllables: 5, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "башни", lemma: "башня", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "построить", lemma: "построить", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "тяжести", lemma: "тяжесть", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "отвесе", lemma: "отвес", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "каменщика", lemma: "каменщик", stressed_syllable_1based: 1, total_syllables: 4, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "натяжении", lemma: "натяжение", stressed_syllable_1based: 3, total_syllables: 5, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "тетивы", lemma: "тетива", stressed_syllable_1based: 3, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "уберите", lemma: "убрать", stressed_syllable_1based: 3, total_syllables: 4, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "лишнее", lemma: "лишний", stressed_syllable_1based: 1, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "звено", lemma: "звено", stressed_syllable_1based: 2, total_syllables: 2, paradigm: ZaliznyakParadigm::MobileSingEndPluralStemD },
    LexiconStressEntry { word_form: "арка", lemma: "арка", stressed_syllable_1based: 1, total_syllables: 2, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "зазвучит", lemma: "зазвучать", stressed_syllable_1based: 3, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedEndingB },
    LexiconStressEntry { word_form: "иначе", lemma: "иначе", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "страницы", lemma: "страница", stressed_syllable_1based: 2, total_syllables: 3, paradigm: ZaliznyakParadigm::FixedStemA },
    LexiconStressEntry { word_form: "архитектура", lemma: "архитектура", stressed_syllable_1based: 4, total_syllables: 5, paradigm: ZaliznyakParadigm::FixedStemA },
];

/// Looks up a word form in the curated Zaliznyak literary lexicon.
#[inline]
pub fn lookup_lexicon_stress(word: &str) -> Option<&'static LexiconStressEntry> {
    let lower: String = word
        .trim_matches(|c: char| !c.is_alphabetic())
        .chars()
        .flat_map(|c| c.to_lowercase())
        .collect();
    LITERARY_STRESS_LEXICON
        .iter()
        .find(|entry| entry.word_form == lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zaliznyak_paradigm_lookup() {
        let entry = lookup_lexicon_stress("Тишина").expect("in lexicon");
        assert_eq!(entry.stressed_syllable_1based, 3);
        assert_eq!(entry.paradigm, ZaliznyakParadigm::FixedEndingB);

        let entry2 = lookup_lexicon_stress("равновесие").expect("in lexicon");
        assert_eq!(entry2.stressed_syllable_1based, 3);
    }
}
