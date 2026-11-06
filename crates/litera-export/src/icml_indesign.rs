//! Adobe InDesign ICML (`<?aid style="50" type="snippet" ...?>`) Story Exporter (`icml_indesign.rs`).
//!
//! Exports an analyzed manuscript with smart micro-typography applied into an industry-standard
//! Adobe InCopy / InDesign `.icml` XML package containing `ParagraphStyle` definitions
//! with optical margin alignment (`StoryPreference OpticalMarginAlignment="true"`),
//! Adobe Paragraph Composer (InDesign's Knuth–Plass implementation), and drop-cap rules.

use crate::markdown_ast::apply_editorial_microtypography;

#[derive(Debug, Clone)]
pub struct IcmlExportConfig<'a> {
    pub story_title: &'a str,
    pub body_text: &'a str,
    pub colophon_note: &'a str,
    pub font_family: &'a str,
    pub point_size: f32,
    pub leading_pt: f32,
    pub dropcap_lines: u8,
}

/// Compiles a complete Adobe InDesign `.icml` XML document.
pub fn compile_indesign_icml_story(cfg: &IcmlExportConfig<'_>) -> String {
    let micro = apply_editorial_microtypography(cfg.body_text);
    let mut xml = String::with_capacity(micro.transformed_text.len() + 2048);

    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
    xml.push_str("<?aid style=\"50\" type=\"snippet\" readerVersion=\"6.0\" featureSet=\"257\" product=\"18.0(15)\"?>\n");
    xml.push_str("<Document DOMVersion=\"18.0\" Self=\"litera_doc\">\n");
    xml.push_str("  <RootParagraphStyleGroup Self=\"litera_para_styles\">\n");
    xml.push_str(&format!(
        "    <ParagraphStyle Self=\"ParagraphStyle/LiteraHeading\" Name=\"LiteraHeading\" AppliedFont=\"{}\" PointSize=\"{:.1}\" Justification=\"LeftAlign\" />\n",
        escape_xml_attr(cfg.font_family),
        cfg.point_size * 2.0
    ));
    xml.push_str(&format!(
        "    <ParagraphStyle Self=\"ParagraphStyle/LiteraBodyAntiqua\" Name=\"LiteraBodyAntiqua\" AppliedFont=\"{}\" PointSize=\"{:.1}\" Leading=\"{:.1}\" Justification=\"LeftJustified\" Composer=\"HL Composer\" DropCapLines=\"{}\" DropCapCharacters=\"1\" HyphenateLadderLimit=\"2\" />\n",
        escape_xml_attr(cfg.font_family),
        cfg.point_size,
        cfg.leading_pt,
        cfg.dropcap_lines
    ));
    xml.push_str("  </RootParagraphStyleGroup>\n");

    xml.push_str("  <Story Self=\"litera_main_story\" TrackChanges=\"false\" StoryTitle=\"");
    xml.push_str(&escape_xml_attr(cfg.story_title));
    xml.push_str("\">\n");
    xml.push_str(&format!(
        "    <StoryPreference OpticalMarginAlignment=\"true\" OpticalMarginSize=\"{:.1}\" FrameType=\"TextFrameType\" />\n",
        cfg.point_size
    ));

    // Title paragraph
    xml.push_str("    <ParagraphStyleRange AppliedParagraphStyle=\"ParagraphStyle/LiteraHeading\">\n");
    xml.push_str("      <CharacterStyleRange AppliedCharacterStyle=\"CharacterStyle/$ID/[No character style]\">\n");
    xml.push_str("        <Content>");
    xml.push_str(&escape_xml_attr(cfg.story_title));
    xml.push_str("</Content>\n");
    xml.push_str("        <Br />\n");
    xml.push_str("      </CharacterStyleRange>\n");
    xml.push_str("    </ParagraphStyleRange>\n");

    // Body paragraph
    xml.push_str("    <ParagraphStyleRange AppliedParagraphStyle=\"ParagraphStyle/LiteraBodyAntiqua\">\n");
    xml.push_str("      <CharacterStyleRange AppliedCharacterStyle=\"CharacterStyle/$ID/[No character style]\">\n");
    xml.push_str("        <Content>");
    xml.push_str(&escape_xml_attr(&micro.transformed_text));
    xml.push_str("</Content>\n");
    xml.push_str("        <Br />\n");
    xml.push_str("      </CharacterStyleRange>\n");
    xml.push_str("    </ParagraphStyleRange>\n");

    xml.push_str("  </Story>\n");
    xml.push_str("</Document>\n");
    xml
}

fn escape_xml_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}
