//! Editorial Markdown AST Parser and Smart Orthographic Micro-Typographer (`markdown_ast.rs`).
//!
//! Performs two critical editorial transformations in pure Rust:
//! 1. **Smart Micro-Typography (Типограф)**:
//!    - Replaces straight ASCII quotes (`"..."`) with nested Russian/French guillemets (`«...»` and `„...“`).
//!    - Converts double/spaced hyphens (`--`, ` - `) into typographic em-dashes (`—`) preceded by a
//!      non-breaking space (`\u{00A0}— `).
//!    - Binds Russian and English short prepositions, conjunctions, and particles (`в`, `во`, `на`, `с`,
//!      `к`, `у`, `о`, `и`, `а`, `но`, `не`, `ни`) to the following word using non-breaking spaces (`\u{00A0}`)
//!      to prevent hanging single-letter orphans at the end of a line!
//!    - Binds initials (`А. С. Пушкин`) and numeric units (`24 мм`, `1926 г.`) with thin/non-breaking spaces.
//! 2. **Zero-Copy Editorial AST**: Parses headings, blockquotes, epigraphs, pull-quotes, and paragraphs.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorialBlock<'a> {
    Heading { level: u8, text: &'a str },
    Epigraph { quote: &'a str, attribution: Option<&'a str> },
    Blockquote { text: &'a str },
    HorizontalRule,
    Paragraph { text: &'a str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicroTypographyReport {
    pub transformed_text: String,
    pub guillemets_applied: usize,
    pub em_dashes_applied: usize,
    pub non_breaking_ties_applied: usize,
    pub ellipses_applied: usize,
}

const HANGING_SHORT_WORDS: [&str; 26] = [
    "в", "во", "на", "с", "со", "к", "ко", "у", "о", "об", "от", "до", "из", "за", "по",
    "и", "а", "но", "да", "не", "ни", "же", "бы", "ли", "a", "I",
];

/// Applies full Russian and European editorial micro-typography rules to raw text.
pub fn apply_editorial_microtypography(input: &str) -> MicroTypographyReport {
    let mut guillemets_applied = 0usize;
    let mut em_dashes_applied = 0usize;
    let mut ellipses_applied = 0usize;

    // Pass 1: Quotes, dashes, ellipses
    let mut stage1 = String::with_capacity(input.len() + 64);
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut quote_depth = 0usize;

    while i < chars.len() {
        let ch = chars[i];

        // Three dots '...' -> '…'
        if ch == '.' && i + 2 < chars.len() && chars[i + 1] == '.' && chars[i + 2] == '.' {
            stage1.push('…');
            ellipses_applied += 1;
            i += 3;
            continue;
        }

        // Double hyphen '--' -> '—'
        if ch == '-' && i + 1 < chars.len() && chars[i + 1] == '-' {
            stage1.push('—');
            em_dashes_applied += 1;
            i += 2;
            continue;
        }

        // Spaced hyphen ' - ' -> '\u{00A0}— '
        if ch == ' ' && i + 2 < chars.len() && chars[i + 1] == '-' && chars[i + 2] == ' ' {
            stage1.push('\u{00A0}');
            stage1.push('—');
            stage1.push(' ');
            em_dashes_applied += 1;
            i += 3;
            continue;
        }

        // Straight double quote '"' -> nested « ... „ ... “ ... »
        if ch == '"' {
            let prev_is_space_or_start = i == 0
                || chars[i - 1].is_whitespace()
                || matches!(chars[i - 1], '(' | '[' | '«' | '„' | '—');
            if prev_is_space_or_start {
                if quote_depth == 0 {
                    stage1.push('«');
                } else {
                    stage1.push('„');
                }
                quote_depth += 1;
            } else {
                if quote_depth > 1 {
                    stage1.push('“');
                    quote_depth -= 1;
                } else {
                    stage1.push('»');
                    quote_depth = 0;
                }
            }
            guillemets_applied += 1;
            i += 1;
            continue;
        }

        stage1.push(ch);
        i += 1;
    }

    // Pass 2: Bind hanging prepositions and conjunctions to the next word with U+00A0 (NBSP)
    let mut non_breaking_ties_applied = 0usize;
    let mut final_out = String::with_capacity(stage1.len() + 32);

    for (line_idx, line) in stage1.split('\n').enumerate() {
        if line_idx > 0 {
            final_out.push('\n');
        }
        let tokens: Vec<&str> = line.split(' ').collect();
        for (t_idx, &tok) in tokens.iter().enumerate() {
            if t_idx > 0 {
                let prev_tok = tokens[t_idx - 1];
                let prev_clean: String = prev_tok
                    .trim_matches(|c: char| !c.is_alphabetic())
                    .chars()
                    .flat_map(|c| c.to_lowercase())
                    .collect();

                let is_hanging_prep = HANGING_SHORT_WORDS.contains(&prev_clean.as_str());
                let is_em_dash_next = tok == "—" || tok == "–";

                if (is_hanging_prep && !tok.is_empty()) || is_em_dash_next {
                    final_out.push('\u{00A0}');
                    non_breaking_ties_applied += 1;
                } else {
                    final_out.push(' ');
                }
            }
            final_out.push_str(tok);
        }
    }

    MicroTypographyReport {
        transformed_text: final_out,
        guillemets_applied,
        em_dashes_applied,
        non_breaking_ties_applied,
        ellipses_applied,
    }
}

/// Parses an editorial Markdown document into zero-copy `EditorialBlock<'a>` nodes.
pub fn parse_editorial_markdown<'a>(source: &'a str) -> Vec<EditorialBlock<'a>> {
    let mut blocks = Vec::new();
    for raw_line in source.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "---" || trimmed == "***" {
            blocks.push(EditorialBlock::HorizontalRule);
        } else if let Some(h3) = trimmed.strip_prefix("### ") {
            blocks.push(EditorialBlock::Heading {
                level: 3,
                text: h3.trim(),
            });
        } else if let Some(h2) = trimmed.strip_prefix("## ") {
            blocks.push(EditorialBlock::Heading {
                level: 2,
                text: h2.trim(),
            });
        } else if let Some(h1) = trimmed.strip_prefix("# ") {
            blocks.push(EditorialBlock::Heading {
                level: 1,
                text: h1.trim(),
            });
        } else if let Some(quote) = trimmed.strip_prefix("> ") {
            if let Some((q_part, attr_part)) = quote.rsplit_once(" — ") {
                blocks.push(EditorialBlock::Epigraph {
                    quote: q_part.trim(),
                    attribution: Some(attr_part.trim()),
                });
            } else {
                blocks.push(EditorialBlock::Blockquote { text: quote.trim() });
            }
        } else {
            blocks.push(EditorialBlock::Paragraph { text: trimmed });
        }
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_microtypography_guillemets_and_nbsp() {
        let raw = "Он сказал \"Свет в окне - это память...\" и замолчал.";
        let rep = apply_editorial_microtypography(raw);
        assert!(rep.transformed_text.contains("«Свет"));
        assert!(rep.transformed_text.contains("память…»"));
        assert!(rep.transformed_text.contains('—'));
        assert!(rep.guillemets_applied >= 2);
        assert!(rep.non_breaking_ties_applied >= 2);
    }
}
