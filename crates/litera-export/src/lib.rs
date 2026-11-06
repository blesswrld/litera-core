//! `litera-export`: Compiles analyzed manuscripts and page geometry canons into
//! production-ready **Typst (`.typ`)**, **XeLaTeX / LuaLaTeX (`.tex`)**,
//! **Encapsulated PostScript Level 3 (`.eps`)**, **Adobe InDesign ICML (`.icml`)**,
//! **Archival HTML5 Book Editions (`.html`)**, **Smart Editorial Micro-Typography**,
//! and **Vector SVG Galley Proof (`.svg`)** documents.

pub mod html_book;
pub mod icml_indesign;
pub mod latex;
pub mod markdown_ast;
pub mod pdf_page_tree;
pub mod postscript_eps;
pub mod svg_galley;
pub mod typst;

pub use html_book::{compile_archival_html_edition, HtmlBookExportConfig};
pub use icml_indesign::{compile_indesign_icml_story, IcmlExportConfig};
pub use latex::{compile_xelatex_manuscript, LatexExportConfig};
pub use markdown_ast::{
    apply_editorial_microtypography, parse_editorial_markdown, EditorialBlock,
    MicroTypographyReport,
};
pub use pdf_page_tree::{compile_pdf_17_document, format_pdf_tj_kerning_array, PdfDocumentConfig};
pub use postscript_eps::{compile_postscript_eps_galley, mm_to_ps_points, PostScriptGalleyConfig};
pub use svg_galley::{render_svg_galley_proof, SvgGalleyConfig};
pub use typst::{compile_typst_manuscript, TypstExportConfig};
