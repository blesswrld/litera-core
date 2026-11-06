use litera_core::{analyze_manuscript, TempoClass};
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_ok() && !input.trim().is_empty() {
        print_report(&input);
    } else {
        let sample = "Тишина имеет собственный вес. Когда взгляд скользит по широкому полю тёплой бумаги, каждая засечка шрифта удерживает равновесие между светом и тенью.";
        print_report(sample);
    }
}

fn print_report(text: &str) {
    let profile = analyze_manuscript(text);
    println!("=== ЛИТЕРА // АКУСТИЧЕСКИЙ ПАСПОРТ ТЕКСТА ===");
    println!("Слов: {} | Слогов: {} | Предложений: {}", profile.word_count, profile.syllable_count, profile.sentence_count);
    println!("Гласные: {:.1}% | Сонорные: {:.1}% | Фрикативные: {:.1}% | Взрывные: {:.1}%",
        profile.vowel_ratio * 100.0,
        profile.sonorant_ratio * 100.0,
        profile.fricative_ratio * 100.0,
        profile.plosive_ratio * 100.0
    );
    println!("Плотность набора (Ink Density): {:.1}/100 | Вариативность ритма: {:.2}", profile.mean_ink_density, profile.cadence_variance);
    for s in &profile.sentences {
        let label = match s.tempo_tag {
            TempoClass::Staccato => "Staccato",
            TempoClass::Moderato => "Moderato",
            TempoClass::Legato => "Legato",
        };
        println!("  [{:02}] ({}, {} сл., {} слог.): {}", s.index + 1, label, s.word_count, s.syllable_count, s.raw_slice);
    }
}
