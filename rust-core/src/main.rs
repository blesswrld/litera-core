//! `litera-cli`: Multi-Subcommand Terminal Workstation for Acoustic Typography & Computational Typesetting.
//!
//! Subcommands:
//! - `litera-cli analyze`      — Full phonetic, cadence, and ink-density passport
//! - `litera-cli anthology`    — Lists and analyzes all 8 bilingual anthology specimens
//! - `litera-cli verify`       — Executes the architectural invariant verification suite

use litera_core::{
    analyze_manuscript, run_full_architectural_verification, TempoClass, ANTHOLOGY_SPECIMENS,
};
use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    let subcommand = args.get(1).map(String::as_str).unwrap_or("analyze");

    match subcommand {
        "anthology" => run_anthology_report(),
        "verify" => run_verification_suite(),
        _ => {
            let mut input = String::new();
            if !atty_stdin_fallback()
                && io::stdin().read_to_string(&mut input).is_ok()
                && !input.trim().is_empty()
            {
                print_detailed_report("Поток STDIN", &input);
            } else {
                let spec = &ANTHOLOGY_SPECIMENS[0];
                print_detailed_report(spec.title, spec.text);
            }
        }
    }
}

fn atty_stdin_fallback() -> bool {
    env::var("LITERA_FORCE_SAMPLE").is_ok()
}

fn print_detailed_report(title: &str, text: &str) {
    let profile = analyze_manuscript(text);
    println!("┌────────────────────────────────────────────────────────────────────────────┐");
    println!("│ ЛИТЕРА (v0.4.0) · АКУСТИЧЕСКАЯ ТИПОГРАФИКА И КАДЕНЦИЯ НА RUST              │");
    println!("├────────────────────────────────────────────────────────────────────────────┤");
    println!("│ Документ : {:<63} │", title);
    println!(
        "│ Объём    : {} слов · {} слогов · {} предл. · {} символов UTF-8          │",
        profile.word_count, profile.syllable_count, profile.sentence_count, profile.char_count
    );
    println!(
        "│ Фонетика : Гласные {:>4.1}% | Сонорные {:>4.1}% | Щелевые {:>4.1}% | Взрывные {:>4.1}% │",
        profile.vowel_ratio * 100.0,
        profile.sonorant_ratio * 100.0,
        profile.fricative_ratio * 100.0,
        profile.plosive_ratio * 100.0
    );
    println!(
        "│ Набор    : Масса чернил {:>4.1}/100 | Индекс динамики ритма {:>4.0}%              │",
        profile.mean_ink_density,
        profile.cadence_variance * 100.0
    );
    println!(
        "│ Хеш FNV1a: 0x{:016X}                                              │",
        profile.acoustic_hash
    );
    println!("└────────────────────────────────────────────────────────────────────────────┘");

    for s in &profile.sentences {
        let label = match s.tempo_tag {
            TempoClass::Staccato => "Staccato",
            TempoClass::Moderato => "Moderato",
            TempoClass::Legato => "Legato  ",
        };
        println!(
            "  [{:02}] {} ({:>2} сл., {:>2} слог.): {}",
            s.index + 1,
            label,
            s.word_count,
            s.syllable_count,
            s.raw_slice
        );
    }
}

fn run_anthology_report() {
    println!("=== АРХИВНАЯ АНТОЛОГИЯ ЛИТЕРА ({} ТЕКСТОВ) ===", ANTHOLOGY_SPECIMENS.len());
    for spec in &ANTHOLOGY_SPECIMENS {
        let prof = analyze_manuscript(spec.text);
        println!(
            "• [{}] {} ({}) — {} сл., плотность {:.1}/100, вариативность {:.0}%",
            spec.id,
            spec.title,
            spec.year,
            prof.word_count,
            prof.mean_ink_density,
            prof.cadence_variance * 100.0
        );
    }
}

fn run_verification_suite() {
    println!("=== ПРОВЕРКА АРХИТЕКТУРНЫХ ИНВАРИАНТОВ (litera-core) ===");
    let checks = run_full_architectural_verification();
    for check in &checks {
        let status = if check.passed { "PASS" } else { "FAIL" };
        println!(
            "  [{}] {:<36} ({}) -> {}",
            status, check.check_id, check.subsystem, check.metric_summary
        );
    }
}
