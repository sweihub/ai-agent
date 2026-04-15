#[derive(Clone, Debug)]
pub struct AnsiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl AnsiColor {
    pub fn default_fg() -> Self {
        Self {
            r: 229,
            g: 229,
            b: 229,
        }
    }

    pub fn default_bg() -> Self {
        Self {
            r: 30,
            g: 30,
            b: 30,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextSpan {
    pub text: String,
    pub color: AnsiColor,
    pub bold: bool,
}

pub type ParsedLine = Vec<TextSpan>;

pub fn parse_ansi(text: &str) -> Vec<ParsedLine> {
    let mut lines: Vec<ParsedLine> = Vec::new();
    let raw_lines = text.split('\n');

    for line in raw_lines {
        let mut spans: Vec<TextSpan> = Vec::new();
        let mut current_color = AnsiColor::default_fg();
        let mut bold = false;
        let mut i = 0;

        while i < line.len() {
            if line.chars().nth(i) == Some('\x1b') && line.chars().nth(i + 1) == Some('[') {
                let mut j = i + 2;
                while j < line.len() && !line.chars().nth(j).map_or(false, |c| c.is_alphabetic()) {
                    j += 1;
                }

                if line.chars().nth(j) == Some('m') {
                    let codes: Vec<u32> = line[i + 2..j]
                        .split(';')
                        .filter_map(|s| s.parse().ok())
                        .collect();

                    let mut k = 0;
                    while k < codes.len() {
                        let code = codes[k];
                        if code == 0 {
                            current_color = AnsiColor::default_fg();
                            bold = false;
                        } else if code == 1 {
                            bold = true;
                        } else if (30..=37).contains(&code) || (90..=97).contains(&code) {
                            current_color = get_ansi_color(code);
                        } else if code == 39 {
                            current_color = AnsiColor::default_fg();
                        }
                        k += 1;
                    }
                }
                i = j + 1;
                continue;
            }

            let text_start = i;
            while i < line.len() && line.chars().nth(i) != Some('\x1b') {
                i += 1;
            }

            let span_text = &line[text_start..i];
            if !span_text.is_empty() {
                spans.push(TextSpan {
                    text: span_text.to_string(),
                    color: current_color.clone(),
                    bold,
                });
            }
        }

        if spans.is_empty() {
            spans.push(TextSpan {
                text: String::new(),
                color: AnsiColor::default_fg(),
                bold: false,
            });
        }

        lines.push(spans);
    }

    lines
}

fn get_ansi_color(code: u32) -> AnsiColor {
    match code {
        30 => AnsiColor { r: 0, g: 0, b: 0 },
        31 => AnsiColor {
            r: 205,
            g: 49,
            b: 49,
        },
        32 => AnsiColor {
            r: 13,
            g: 188,
            b: 121,
        },
        33 => AnsiColor {
            r: 229,
            g: 229,
            b: 16,
        },
        34 => AnsiColor {
            r: 36,
            g: 114,
            b: 200,
        },
        35 => AnsiColor {
            r: 188,
            g: 63,
            b: 188,
        },
        36 => AnsiColor {
            r: 17,
            g: 168,
            b: 205,
        },
        37 => AnsiColor {
            r: 229,
            g: 229,
            b: 229,
        },
        90 => AnsiColor {
            r: 102,
            g: 102,
            b: 102,
        },
        91 => AnsiColor {
            r: 241,
            g: 76,
            b: 76,
        },
        92 => AnsiColor {
            r: 35,
            g: 209,
            b: 139,
        },
        93 => AnsiColor {
            r: 245,
            g: 245,
            b: 67,
        },
        94 => AnsiColor {
            r: 59,
            g: 142,
            b: 234,
        },
        95 => AnsiColor {
            r: 214,
            g: 112,
            b: 214,
        },
        96 => AnsiColor {
            r: 41,
            g: 184,
            b: 219,
        },
        97 => AnsiColor {
            r: 255,
            g: 255,
            b: 255,
        },
        _ => AnsiColor::default_fg(),
    }
}

pub fn ansi_to_svg(ansi_text: &str) -> String {
    let lines = parse_ansi(ansi_text);
    let font_family = "Menlo, Monaco, monospace";
    let font_size = 14;
    let line_height = 22;
    let padding_x = 24;
    let padding_y = 24;

    let bg = AnsiColor::default_bg();
    let background_color = format!("rgb({}, {}, {})", bg.r, bg.g, bg.b);

    let max_line_length = lines
        .iter()
        .map(|spans| spans.iter().map(|s| s.text.len()).sum::<usize>())
        .max()
        .unwrap_or(1);

    let width = (max_line_length as f64 * (font_size as f64 * 0.6)) as u32 + padding_x * 2;
    let height = lines.len() as u32 * line_height + padding_y * 2;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
  <rect width="100%" height="100%" fill="{}" rx="8" ry="8"/>
  <style>
    text {{ font-family: {}; font-size: {}px; white-space: pre; }}
    .b {{ font-weight: bold; }}
  </style>
"#,
        width, height, width, height, background_color, font_family, font_size
    );

    for (line_idx, spans) in lines.iter().enumerate() {
        let y = (padding_y as u32 + (line_idx + 1) as u32 * line_height
            - (line_height - font_size) / 2) as u32;
        svg += &format!(
            "  <text x=\"{}\" y=\"{}\" xml:space=\"preserve\">",
            padding_x, y
        );

        for span in spans {
            if span.text.is_empty() {
                continue;
            }
            let color_str = format!("rgb({}, {}, {})", span.color.r, span.color.g, span.color.b);
            let bold_class = if span.bold { " class=\"b\"" } else { "" };
            let escaped = escape_xml(&span.text);
            svg += &format!(
                "<tspan fill=\"{}\"{}>{}</tspan>",
                color_str, bold_class, escaped
            );
        }

        svg += "</text>\n";
    }

    svg += "</svg>";
    svg
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
