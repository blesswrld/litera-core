//! `litera-typography`: Computational Typesetting, Knuth–Plass Optimal Line-Breaking,
//! Frank Liang TeX Hyphenation, OpenType SFNT Binary Font Inspection,
//! OpenType `GSUB`/`GPOS` Shaping, Unicode UAX #14 Line-Breaking State Machine,
//! Parametric Bézier Outline Geometry, Optical Kerning, and Classical Page Geometry Canons.

pub mod baseline_composer;
pub mod bezier_outlines;
pub mod grid;
pub mod harfbuzz_shaper;
pub mod hyphenation;
pub mod justification_hz;
pub mod kerning;
pub mod knuth_plass;
pub mod math_typesetting_tex;
pub mod metrics;
pub mod opentype_tables;
pub mod optical_sizing_varfont;
pub mod unicode_bidi_linebreak;

pub use baseline_composer::{
    compose_spread_pages, BaselineGridSpec, ComposedLineSlot, ComposedPage, SpreadCompositionReport,
};
pub use bezier_outlines::{
    inspect_typeface_contrast, ContourSegment, GlyphOpticalContrastSpec, GlyphOutlineContour,
    Point2D,
};
pub use grid::{
    compute_page_geometry, CanonPreset, MarginProportions, ModularScaleStep, PageDimensionsMm,
    PageGeometryReport,
};
pub use harfbuzz_shaper::{
    shape_text_run, OpenTypeFeatureTag, ShapedGlyph, ShapedGlyphBuffer, ShapingFeatureSet,
};
pub use hyphenation::{
    hyphenate_text_preview, hyphenate_word, parse_liang_pattern, HyphenatedWord,
    LIANG_PATTERNS_RU_EN,
};
pub use justification_hz::{
    compute_hz_word_advances, glyph_counter_elasticity, optimize_lines_with_hz_program,
    requires_hz_emergency_pass, HzJustifiedLine, HzParagraphReport, HzProgramConfig,
};
pub use kerning::{
    analyze_text_kerning, kern_pair_units, left_protrusion_units, right_protrusion_units,
    KerningEvent, KerningSummary,
};
pub use knuth_plass::{
    compare_paragraph_breaking, layout_paragraph_greedy, layout_paragraph_knuth_plass, FitnessClass,
    GalleyItem, LineBreakComparison, TypesetLine,
};
pub use math_typesetting_tex::{
    classify_math_symbol, lookup_tex_math_spacing, typeset_math_formula, MathAtomClass,
    MathMuSkip, PositionedMathAtom, TexFormulaLayout, TexMathStyle,
};
pub use metrics::{
    compute_glyph_metrics, measure_string_em_units, FontCategory, GlyphMetric, ParagraphInkProfile,
};
pub use opentype_tables::{
    parse_opentype_sfnt, synthesize_cormorant_sfnt_specimen, HeadTableMetrics, HheaTableMetrics,
    OpenTypeFontInspection, SfntFlavor, SfntTableRecord,
};
pub use optical_sizing_varfont::{
    generate_optical_scale_specimen_table, interpolate_optical_master, normalize_opsz_with_avar,
    HistoricalBodySizeGrade, OpticalMasterInstance, VariationAxisDescriptor,
    CORMORANT_VARIATION_AXES,
};
pub use unicode_bidi_linebreak::{
    classify_uax14_char, evaluate_uax14_pair, segment_uax14_break_units, BreakAction,
    Uax14BreakClass, Uax14Segment,
};
