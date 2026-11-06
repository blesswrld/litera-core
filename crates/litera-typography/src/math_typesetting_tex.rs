//! Donald Knuth TeX Appendix G Mathematical Formula Typesetting Engine (`math_typesetting_tex.rs`).
//!
//! Implements the classical TeX mathematical box and glue engine defined in *The TeXbook*
//! (Chapters 17–18 and Appendix G):
//! - **Seven Math Atom Classes**: `Ord` (ordinary symbol), `Op` (large operator $\sum, \int$),
//!   `Bin` (binary operation $+, -$), `Rel` (relation $=, \equiv, \to$), `Open` (opening delimiter $(, [$),
//!   `Close` (closing delimiter $), ]$), `Punct` (punctuation $,$), and `Inner` (fractions).
//! - **The $8 \times 8$ Inter-Atom Spacing Matrix**: Assigns `0`, `thinmuskip` ($3\text{ mu}$),
//!   `medmuskip` ($4\text{ mu}$), or `thickmuskip` ($5\text{ mu}$) where $1\text{ em} = 18\text{ mu}$ (math units),
//!   automatically suppressing `medmuskip` and `thickmuskip` in `Script` and `ScriptScript` styles.
//! - **Context-Sensitive Unary/Binary Promotion**: Automatically converts a `Bin` atom (such as $-$ or $+$)
//!   into an `Ord` atom when preceded by `Bin`, `Op`, `Rel`, `Open`, `Punct`, or the start of a formula.

/// TeX Mathematical Style (`D`, `T`, `S`, `SS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexMathStyle {
    /// Display style (`\displaystyle`): full size, operators expanded.
    Display,
    /// Text style (`\textstyle`): inline equations within prose paragraphs.
    Text,
    /// Script style (`\scriptstyle`): first-level superscripts and subscripts ($70\%$ scale).
    Script,
    /// Scriptscript style (`\scriptscriptstyle`): second-level nested scripts ($50\%$ scale).
    ScriptScript,
}

impl TexMathStyle {
    /// Scale multiplier relative to the surrounding text body size.
    #[inline]
    pub const fn font_scale_factor(self) -> f32 {
        match self {
            TexMathStyle::Display | TexMathStyle::Text => 1.00,
            TexMathStyle::Script => 0.70,
            TexMathStyle::ScriptScript => 0.50,
        }
    }

    /// Returns the next smaller style used for superscripts/subscripts.
    #[inline]
    pub const fn script_style(self) -> Self {
        match self {
            TexMathStyle::Display | TexMathStyle::Text => TexMathStyle::Script,
            TexMathStyle::Script | TexMathStyle::ScriptScript => TexMathStyle::ScriptScript,
        }
    }
}

/// The 8 fundamental TeX mathematical atom classes (*The TeXbook*, p. 158).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MathAtomClass {
    /// Class 0: Ordinary variable or symbol ($x, \alpha, \partial, 0$).
    Ord = 0,
    /// Class 1: Large operator ($\sum, \int, \prod, \lim, \log$).
    Op = 1,
    /// Class 2: Binary operation ($+, -, \times, \cdot, \oplus$).
    Bin = 2,
    /// Class 3: Relation ($=, <, >, \equiv, \approx, \in, \to$).
    Rel = 3,
    /// Class 4: Opening delimiter ($(, [, \{, \langle$).
    Open = 4,
    /// Class 5: Closing delimiter ($), ], \}, \rangle$).
    Close = 5,
    /// Class 6: Punctuation ($, , ; , !$).
    Punct = 6,
    /// Class 7: Inner subformula (fractions `\frac`, `\left...\right`).
    Inner = 7,
}

impl MathAtomClass {
    pub const fn tex_name(self) -> &'static str {
        match self {
            MathAtomClass::Ord => "\\mathord",
            MathAtomClass::Op => "\\mathop",
            MathAtomClass::Bin => "\\mathbin",
            MathAtomClass::Rel => "\\mathrel",
            MathAtomClass::Open => "\\mathopen",
            MathAtomClass::Close => "\\mathclose",
            MathAtomClass::Punct => "\\mathpunct",
            MathAtomClass::Inner => "\\mathinner",
        }
    }
}

/// Mathematical glue unit (`mu`, where $18\text{ mu} = 1\text{ em}$).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathMuSkip {
    /// No space ($0\text{ mu}$).
    Zero,
    /// `\thinmuskip`: $3\text{ mu} = 1/6\text{ em}$ (always active in all styles).
    ThinAlways,
    /// Conditional `\thinmuskip`: $3\text{ mu}$ in `Display`/`Text`, $0\text{ mu}$ in `Script`/`ScriptScript`.
    ThinConditional,
    /// Conditional `\medmuskip`: $4\text{ mu} = 2/9\text{ em}$ in `Display`/`Text`, $0\text{ mu}$ in scripts.
    MedConditional,
    /// Conditional `\thickmuskip`: $5\text{ mu} = 5/18\text{ em}$ in `Display`/`Text`, $0\text{ mu}$ in scripts.
    ThickConditional,
    /// Invalid transition in TeX's table (marked `*` in *The TeXbook* p. 170, resolved by promoting `Bin` to `Ord`).
    InvalidBinTransition,
}

impl MathMuSkip {
    /// Evaluates the exact horizontal spacing in `mu` ($1/18\text{ em}$) for a given `TexMathStyle`.
    #[inline]
    pub const fn to_mu_units(self, style: TexMathStyle) -> u8 {
        let in_script = matches!(style, TexMathStyle::Script | TexMathStyle::ScriptScript);
        match self {
            MathMuSkip::Zero | MathMuSkip::InvalidBinTransition => 0,
            MathMuSkip::ThinAlways => 3,
            MathMuSkip::ThinConditional => {
                if in_script {
                    0
                } else {
                    3
                }
            }
            MathMuSkip::MedConditional => {
                if in_script {
                    0
                } else {
                    4
                }
            }
            MathMuSkip::ThickConditional => {
                if in_script {
                    0
                } else {
                    5
                }
            }
        }
    }
}

/// Knuth's canonical $8 \times 8$ inter-atom spacing lookup table (*The TeXbook*, Chapter 18, p. 170).
/// Rows correspond to left atom class, columns to right atom class.
pub fn lookup_tex_math_spacing(left: MathAtomClass, right: MathAtomClass) -> MathMuSkip {
    use MathAtomClass::*;
    use MathMuSkip::*;

    match (left, right) {
        // Left = Ord (0)
        (Ord, Ord) | (Ord, Open) | (Ord, Close) | (Ord, Punct) => Zero,
        (Ord, Op) => ThinAlways,
        (Ord, Bin) => MedConditional,
        (Ord, Rel) => ThickConditional,
        (Ord, Inner) => ThinConditional,

        // Left = Op (1)
        (Op, Ord) | (Op, Op) => ThinAlways,
        (Op, Bin) => InvalidBinTransition,
        (Op, Rel) => ThickConditional,
        (Op, Open) | (Op, Close) | (Op, Punct) => Zero,
        (Op, Inner) => ThinConditional,

        // Left = Bin (2)
        (Bin, Ord) | (Bin, Op) | (Bin, Open) | (Bin, Inner) => MedConditional,
        (Bin, Bin) | (Bin, Rel) | (Bin, Close) | (Bin, Punct) => InvalidBinTransition,

        // Left = Rel (3)
        (Rel, Ord) | (Rel, Op) | (Rel, Open) | (Rel, Inner) => ThickConditional,
        (Rel, Bin) => InvalidBinTransition,
        (Rel, Rel) | (Rel, Close) | (Rel, Punct) => Zero,

        // Left = Open (4)
        (Open, Bin) => InvalidBinTransition,
        (Open, _) => Zero,

        // Left = Close (5)
        (Close, Ord) | (Close, Open) | (Close, Close) | (Close, Punct) => Zero,
        (Close, Op) => ThinAlways,
        (Close, Bin) => MedConditional,
        (Close, Rel) => ThickConditional,
        (Close, Inner) => ThinConditional,

        // Left = Punct (6)
        (Punct, Bin) => InvalidBinTransition,
        (Punct, _) => ThinConditional,

        // Left = Inner (7)
        (Inner, Ord) | (Inner, Open) | (Inner, Punct) | (Inner, Inner) => ThinConditional,
        (Inner, Op) => ThinAlways,
        (Inner, Bin) => MedConditional,
        (Inner, Rel) => ThickConditional,
        (Inner, Close) => Zero,
    }
}

/// Classifies a single Unicode mathematical character into its canonical TeX `MathAtomClass`.
#[inline]
pub fn classify_math_symbol(ch: char) -> MathAtomClass {
    match ch {
        '+' | '-' | '−' | '±' | '×' | '÷' | '·' | '⋅' | '⊕' | '⊗' | '∪' | '∩' | '*' => {
            MathAtomClass::Bin
        }
        '=' | '<' | '>' | '≤' | '≥' | '≠' | '≡' | '≈' | '∼' | '∈' | '⊂' | '⊆' | '→' | '↦'
        | '⇔' | '⇒' | ':' => MathAtomClass::Rel,
        '(' | '[' | '{' | '⟨' | '⌈' | '⌊' => MathAtomClass::Open,
        ')' | ']' | '}' | '⟩' | '⌉' | '⌋' => MathAtomClass::Close,
        ',' | ';' | '!' => MathAtomClass::Punct,
        '∑' | '∏' | '∫' | '∮' | '⋃' | '⋂' => MathAtomClass::Op,
        _ => MathAtomClass::Ord,
    }
}

/// Positioned mathematical atom with resolved class and horizontal coordinates in `mu` ($1/18\text{ em}$).
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedMathAtom {
    pub symbol: char,
    pub original_class: MathAtomClass,
    pub resolved_class: MathAtomClass,
    pub preceding_skip_mu: u8,
    pub x_offset_mu: f32,
    pub advance_width_mu: f32,
}

/// Complete horizontal layout of a mathematical formula under TeX Appendix G rules.
#[derive(Debug, Clone, PartialEq)]
pub struct TexFormulaLayout {
    pub style: TexMathStyle,
    pub atoms: Vec<PositionedMathAtom>,
    pub total_width_mu: f32,
    pub total_width_em: f32,
    pub unary_promotions_applied: usize,
}

/// Typesets a mathematical expression according to Knuth's Appendix G algorithm:
/// 1. Pass 1: Classify atoms and apply context-sensitive `Bin` $\to$ `Ord` unary promotion.
/// 2. Pass 2: Insert exact `mu` skips from the $8 \times 8$ matrix and accumulate coordinates.
pub fn typeset_math_formula(expression: &str, style: TexMathStyle) -> TexFormulaLayout {
    let raw_chars: Vec<char> = expression.chars().filter(|c| !c.is_whitespace()).collect();
    if raw_chars.is_empty() {
        return TexFormulaLayout {
            style,
            atoms: Vec::new(),
            total_width_mu: 0.0,
            total_width_em: 0.0,
            unary_promotions_applied: 0,
        };
    }

    let orig_classes: Vec<MathAtomClass> =
        raw_chars.iter().map(|c| classify_math_symbol(*c)).collect();
    let mut resolved = orig_classes.clone();
    let mut unary_promotions = 0usize;

    // Pass 1: TeX Rule 5 & 6 — Promote `Bin` to `Ord` if at the start/end of formula,
    // or if preceded by `Bin`, `Op`, `Rel`, `Open`, `Punct`, or followed by `Rel`, `Close`, `Punct`.
    for i in 0..resolved.len() {
        if resolved[i] == MathAtomClass::Bin {
            let preceded_by_non_operand = i == 0
                || matches!(
                    resolved[i - 1],
                    MathAtomClass::Bin
                        | MathAtomClass::Op
                        | MathAtomClass::Rel
                        | MathAtomClass::Open
                        | MathAtomClass::Punct
                );
            let followed_by_closing = i + 1 == resolved.len()
                || matches!(
                    resolved[i + 1],
                    MathAtomClass::Rel | MathAtomClass::Close | MathAtomClass::Punct
                );

            if preceded_by_non_operand || followed_by_closing {
                resolved[i] = MathAtomClass::Ord;
                unary_promotions += 1;
            }
        }
    }

    // Pass 2: Compute inter-atom `mu` glue and position each glyph
    let scale = style.font_scale_factor();
    let mut x_mu = 0.0f32;
    let mut atoms = Vec::with_capacity(raw_chars.len());

    for i in 0..raw_chars.len() {
        let skip_mu = if i == 0 {
            0
        } else {
            lookup_tex_math_spacing(resolved[i - 1], resolved[i]).to_mu_units(style)
        };
        x_mu += skip_mu as f32;

        let base_width_mu = match resolved[i] {
            MathAtomClass::Op => 14.0,
            MathAtomClass::Bin | MathAtomClass::Rel => 11.0,
            MathAtomClass::Open | MathAtomClass::Close | MathAtomClass::Punct => 6.0,
            _ => 9.5,
        } * scale;

        atoms.push(PositionedMathAtom {
            symbol: raw_chars[i],
            original_class: orig_classes[i],
            resolved_class: resolved[i],
            preceding_skip_mu: skip_mu,
            x_offset_mu: x_mu,
            advance_width_mu: base_width_mu,
        });

        x_mu += base_width_mu;
    }

    TexFormulaLayout {
        style,
        atoms,
        total_width_mu: x_mu,
        total_width_em: x_mu / 18.0,
        unary_promotions_applied: unary_promotions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unary_minus_promotion_vs_binary_minus() {
        // In "-x + y = -z", the first '-' and the '-' after '=' must be promoted to Ord (0 skip after '-'),
        // while '+' remains Bin (4 mu medmuskip on both sides).
        let layout = typeset_math_formula("-x + y = -z", TexMathStyle::Display);
        assert_eq!(layout.unary_promotions_applied, 2);
        assert_eq!(layout.atoms[0].resolved_class, MathAtomClass::Ord); // leading '-'
        assert_eq!(layout.atoms[1].preceding_skip_mu, 0); // no space between unary '-' and 'x'
        assert_eq!(layout.atoms[2].resolved_class, MathAtomClass::Bin); // binary '+'
        assert_eq!(layout.atoms[2].preceding_skip_mu, 4); // medmuskip = 4 mu
    }
}
