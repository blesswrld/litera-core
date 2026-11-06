//! Standalone Archival HTML5 + CSS Paged Media (`@page`) Book Galley Exporter (`html_book.rs`).
//!
//! Generates a complete, self-contained HTML5 monograph with `@page` print rules,
//! OpenType font-feature-settings (`"liga" 1, "onum" 1, "kern" 1`), and marginalia layout.

use crate::markdown_ast::apply_editorial_microtypography;

#[derive(Debug, Clone)]
pub struct HtmlBookExportConfig<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub body_text: &'a str,
    pub colophon_note: &'a str,
    pub font_size_px: u16,
    pub line_height: f32,
    pub use_dropcap: bool,
    pub dark_theme: bool,
}

/// Compiles a standalone archival HTML5 book edition ready for browser viewing or PDF printing (`Ctrl+P`).
pub fn compile_archival_html_edition(cfg: &HtmlBookExportConfig<'_>) -> String {
    let micro = apply_editorial_microtypography(cfg.body_text);
    let (bg, fg, muted, accent, border) = if cfg.dark_theme {
        ("#0A0A0A", "#F5F2EB", "#A8A29E", "#D97706", "#22201E")
    } else {
        ("#FBF9F5", "#1C1917", "#78716C", "#9A3412", "#E6E1D6")
    };

    let dropcap_css = if cfg.use_dropcap {
        format!(
            "p.manuscript-body::first-letter {{ float: left; font-size: 4.1rem; line-height: 0.78; padding: 0.2rem 0.7rem 0.1rem 0; font-weight: 600; color: {accent}; }}"
        )
    } else {
        String::new()
    };

    format!(
        r#"<!doctype html>
<html lang="ru">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{title} — Издание ЛИТЕРА</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,400;0,600;1,400&family=Plus+Jakarta+Sans:wght@400;500&display=swap" rel="stylesheet" />
  <style>
    @page {{
      size: 165mm 240mm;
      margin: 24mm 32mm 34mm 20mm;
    }}
    body {{
      margin: 0;
      padding: 4rem 1.5rem;
      background: {bg};
      color: {fg};
      font-family: 'Cormorant Garamond', Georgia, serif;
      font-feature-settings: "kern" 1, "liga" 1, "onum" 1;
      -webkit-font-smoothing: antialiased;
    }}
    main {{
      max-width: 66ch;
      margin: 0 auto;
    }}
    .kicker {{
      font-family: 'Plus Jakarta Sans', sans-serif;
      font-size: 0.75rem;
      letter-spacing: 0.14em;
      text-transform: uppercase;
      color: {accent};
      margin-bottom: 0.75rem;
    }}
    h1 {{
      font-size: 2.75rem;
      font-weight: 500;
      line-height: 1.12;
      margin: 0 0 1.5rem 0;
      padding-bottom: 1.25rem;
      border-bottom: 1px solid {border};
    }}
    p.manuscript-body {{
      font-size: {font_size}px;
      line-height: {line_height:.2};
      text-align: justify;
      hyphens: auto;
      margin: 0 0 2.5rem 0;
    }}
    {dropcap_css}
    footer {{
      padding-top: 1.25rem;
      border-top: 1px solid {border};
      font-size: 0.95rem;
      font-style: italic;
      color: {muted};
    }}
  </style>
</head>
<body>
  <main>
    <div class="kicker">{subtitle}</div>
    <h1>{title}</h1>
    <p class="manuscript-body">{body}</p>
    <footer>{colophon} · Неразрывных связок: {nbsp}, тире: {dashes}</footer>
  </main>
</body>
</html>
"#,
        title = cfg.title,
        subtitle = cfg.subtitle,
        body = micro.transformed_text,
        colophon = cfg.colophon_note,
        font_size = cfg.font_size_px,
        line_height = cfg.line_height,
        dropcap_css = dropcap_css,
        bg = bg,
        fg = fg,
        muted = muted,
        accent = accent,
        border = border,
        nbsp = micro.non_breaking_ties_applied,
        dashes = micro.em_dashes_applied,
    )
}
