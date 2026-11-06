//! Unicode Standard Annex #14 (UAX #14) Line Breaking Algorithm & Non-Breaking Glue Rules (`unicode_bidi_linebreak.rs`).
//!
//! Implements the deterministic character classification and pair-table decision matrix specified
//! in Unicode UAX #14 (*Unicode Line Breaking Algorithm*), ensuring that:
//! - Opening punctuation (`OP`: `(`, `[`, `«`, `„`, `“`) never breaks from the following word (`LB14`).
//! - Closing punctuation and exclamation/question marks (`CL`, `CP`, `EX`, `IS`: `)`, `]`, `»`, `.`, `,`, `:`, `;`, `!`, `?`)
//!   never break from the preceding word (`LB13`, `LB16`).
//! - Non-breaking spaces (`GL`: `\u{00A0}`, `\u{202F}`) and Word Joiners (`WJ`: `\u{2060}`) weld
//!   adjacent tokens into an indivisible typographic unit (`LB11`, `LB12`).
//! - En-dashes and hyphens (`BA`, `HY`) permit a break *after* the hyphen, whereas Russian
//!   em-dashes (`B2`: `—`) glued on the left with `\u{00A0}` break only *after* the em-dash.

/// Subset of official Unicode UAX #14 Line Break Classes pertinent to European & Slavic book typesetting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Uax14BreakClass {
    /// `BK`: Mandatory Break (`\n`, `\u{2028}`, `\u{2029}`)
    MandatoryBreak,
    /// `WJ`: Word Joiner (`\u{2060}`, `\u{FEFF}`) — prohibits break before and after
    WordJoiner,
    /// `GL`: Non-breaking Glue (`\u{00A0}` NBSP, `\u{202F}` Narrow NBSP)
    NonBreakingGlue,
    /// `SP`: Ordinary Breaking Space (` `)
    Space,
    /// `OP`: Opening Punctuation (`(`, `[`, `{`, `«`, `„`, `“`)
    OpenPunctuation,
    /// `CL`: Closing Punctuation (`)`, `]`, `}`, `»`, `”`)
    ClosePunctuation,
    /// `EX`: Exclamation / Interrogation (`!`, `?`, `‽`)
    Exclamation,
    /// `IS`: Infix Numeric / Sentence Separator (`.`, `,`, `:`, `;`)
    InfixSeparator,
    /// `HY`: Hyphen-Minus (`-`, `\u{00AD}` Soft Hyphen, `‐`)
    Hyphen,
    /// `B2`: Break Opportunity Before and After (`—` Em-Dash, unless glued by `GL`)
    BreakBothEmDash,
    /// `IN`: Inseparable Leaders (`…` Horizontal Ellipsis `\u{2026}`)
    Inseparable,
    /// `NU`: Numeric Digit (`0..=9`)
    Numeric,
    /// `AL`: Ordinary Alphabetic & Cyrillic/Latin Letter
    Alphabetic,
}

/// Line-breaking permission at a character boundary `(i, i+1)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakAction {
    /// `!`: Mandatory newline break (`LB4`–`LB6`).
    Mandatory,
    /// `×`: Break strictly prohibited by UAX #14 (`LB7`–`LB16`).
    Prohibited,
    /// `÷`: Direct break allowed at inter-word whitespace or after hyphen (`LB18`, `LB21`).
    AllowedDirect,
}

/// Classifies any Unicode scalar value into its UAX #14 `Uax14BreakClass`.
#[inline]
pub fn classify_uax14_char(ch: char) -> Uax14BreakClass {
    match ch {
        '\n' | '\r' | '\u{000C}' | '\u{2028}' | '\u{2029}' => Uax14BreakClass::MandatoryBreak,
        '\u{2060}' | '\u{FEFF}' => Uax14BreakClass::WordJoiner,
        '\u{00A0}' | '\u{202F}' | '\u{2007}' => Uax14BreakClass::NonBreakingGlue,
        ' ' | '\t' => Uax14BreakClass::Space,
        '(' | '[' | '{' | '«' | '„' | '“' | '‘' => Uax14BreakClass::OpenPunctuation,
        ')' | ']' | '}' | '»' | '”' | '’' => Uax14BreakClass::ClosePunctuation,
        '!' | '?' => Uax14BreakClass::Exclamation,
        '.' | ',' | ':' | ';' => Uax14BreakClass::InfixSeparator,
        '-' | '\u{00AD}' | '‐' | '–' => Uax14BreakClass::Hyphen,
        '—' => Uax14BreakClass::BreakBothEmDash,
        '…' | '\u{2025}' => Uax14BreakClass::Inseparable,
        '0'..='9' => Uax14BreakClass::Numeric,
        _ => Uax14BreakClass::Alphabetic,
    }
}

/// Evaluates the UAX #14 pair rule between an effective `left` class (ignoring intervening `SP`
/// where required by `LB14`–`LB16`) and the immediate `right` class, given whether an ordinary
/// `SP` (` `) intervened between them.
#[inline]
pub fn evaluate_uax14_pair(
    effective_left: Uax14BreakClass,
    immediate_left: Uax14BreakClass,
    right: Uax14BreakClass,
) -> BreakAction {
    // LB4–LB6: Always break after mandatory break, never break right before mandatory break
    if immediate_left == Uax14BreakClass::MandatoryBreak {
        return BreakAction::Mandatory;
    }
    if right == Uax14BreakClass::MandatoryBreak {
        return BreakAction::Prohibited;
    }

    // LB7: Do not break before spaces
    if right == Uax14BreakClass::Space {
        return BreakAction::Prohibited;
    }

    // LB11: Do not break before or after Word Joiner (WJ)
    if immediate_left == Uax14BreakClass::WordJoiner || right == Uax14BreakClass::WordJoiner {
        return BreakAction::Prohibited;
    }

    // LB12: Do not break after Non-Breaking Glue (GL: \u{00A0})
    if immediate_left == Uax14BreakClass::NonBreakingGlue {
        return BreakAction::Prohibited;
    }

    // LB12a: Do not break before Non-Breaking Glue unless preceded by SP or HY
    if right == Uax14BreakClass::NonBreakingGlue
        && !matches!(
            immediate_left,
            Uax14BreakClass::Space | Uax14BreakClass::Hyphen
        )
    {
        return BreakAction::Prohibited;
    }

    // LB13: Do not break before `]`, `)`, `»`, `!`, `?`, `.`, `,`, `:`, `;` even after spaces
    if matches!(
        right,
        Uax14BreakClass::ClosePunctuation
            | Uax14BreakClass::Exclamation
            | Uax14BreakClass::InfixSeparator
    ) {
        return BreakAction::Prohibited;
    }

    // LB14: Do not break after Opening Punctuation (`(`, `[`, `«`), even across spaces
    if effective_left == Uax14BreakClass::OpenPunctuation {
        return BreakAction::Prohibited;
    }

    // LB17: Do not break between consecutive em-dashes or ellipses (`B2 B2`, `IN IN`)
    if effective_left == Uax14BreakClass::BreakBothEmDash
        && right == Uax14BreakClass::BreakBothEmDash
    {
        return BreakAction::Prohibited;
    }
    if effective_left == Uax14BreakClass::Inseparable && right == Uax14BreakClass::Inseparable {
        return BreakAction::Prohibited;
    }

    // LB18: Break after spaces (`SP ÷`)
    if immediate_left == Uax14BreakClass::Space {
        return BreakAction::AllowedDirect;
    }

    // LB21: Break after hyphens (`HY ÷ AL`) when not followed by numeric or close punctuation
    if immediate_left == Uax14BreakClass::Hyphen
        && matches!(
            right,
            Uax14BreakClass::Alphabetic | Uax14BreakClass::OpenPunctuation
        )
    {
        return BreakAction::AllowedDirect;
    }

    // LB23–LB28: Do not break within words (`AL × AL`), numbers (`NU × NU`), or alphanumeric identifiers
    BreakAction::Prohibited
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uax14Segment<'a> {
    pub slice: &'a str,
    pub byte_start: usize,
    pub byte_end: usize,
    pub break_after: BreakAction,
}

/// Segments a string into indivisible UAX #14 typographic break units (`Uax14Segment<'a>`).
/// Note how words glued by `\u{00A0}` (e.g. `"в\u{00A0}окне"` or `"слово\u{00A0}—"`) remain
/// welded into a single unbreakable `Uax14Segment`!
pub fn segment_uax14_break_units<'a>(text: &'a str) -> Vec<Uax14Segment<'a>> {
    let indexed: Vec<(usize, char)> = text.char_indices().collect();
    if indexed.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut seg_start_byte = 0usize;
    let mut effective_left = classify_uax14_char(indexed[0].1);

    for i in 0..indexed.len() {
        let (_, ch) = indexed[i];
        let imm_left = classify_uax14_char(ch);
        if imm_left != Uax14BreakClass::Space {
            effective_left = imm_left;
        }

        if i + 1 < indexed.len() {
            let (next_byte, next_ch) = indexed[i + 1];
            let right_cls = classify_uax14_char(next_ch);
            let action = evaluate_uax14_pair(effective_left, imm_left, right_cls);

            if matches!(action, BreakAction::Mandatory | BreakAction::AllowedDirect) {
                let slice = &text[seg_start_byte..next_byte];
                if !slice.trim().is_empty() {
                    segments.push(Uax14Segment {
                        slice: slice.trim_end_matches([' ', '\t', '\n', '\r']),
                        byte_start: seg_start_byte,
                        byte_end: next_byte,
                        break_after: action,
                    });
                }
                seg_start_byte = next_byte;
            }
        } else {
            let slice = &text[seg_start_byte..text.len()];
            if !slice.trim().is_empty() {
                segments.push(Uax14Segment {
                    slice: slice.trim_end_matches([' ', '\t', '\n', '\r']),
                    byte_start: seg_start_byte,
                    byte_end: text.len(),
                    break_after: BreakAction::Mandatory,
                });
            }
        }
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uax14_nbsp_welds_preposition_and_em_dash() {
        let sample = "Он вошёл в\u{00A0}зал\u{00A0}— тишина замерла.";
        let units = segment_uax14_break_units(sample);
        // "Он", "вошёл", "в\u{00A0}зал\u{00A0}—", "тишина", "замерла."
        assert_eq!(units.len(), 5);
        assert_eq!(units[2].slice, "в\u{00A0}зал\u{00A0}—");
    }
}
