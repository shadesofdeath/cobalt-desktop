//! a tiny markdown renderer for the about pages and changelogs
//! (headings, paragraphs, bullet lists, **bold**, *italic*, `code`, [links](url)).

use super::widgets::{self, Ctx};
use crate::theme;
use egui::{text::LayoutJob, Align, Color32, Sense, TextFormat, Ui, Vec2};

#[derive(Clone, Debug)]
enum Span {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Link { text: String, url: String },
}

fn parse_inline(s: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut buf = String::new();
    let mut i = 0;
    let flush = |buf: &mut String, out: &mut Vec<Span>| {
        if !buf.is_empty() {
            out.push(Span::Text(std::mem::take(buf)));
        }
    };
    while i < chars.len() {
        let c = chars[i];
        if c == '`' {
            if let Some(end) = chars[i + 1..].iter().position(|&x| x == '`') {
                flush(&mut buf, &mut out);
                out.push(Span::Code(chars[i + 1..i + 1 + end].iter().collect()));
                i += end + 2;
                continue;
            }
        }
        if c == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let rest = &chars[i + 2..];
            if let Some(end) = find_seq(rest, &['*', '*']) {
                flush(&mut buf, &mut out);
                out.push(Span::Bold(rest[..end].iter().collect()));
                i += end + 4;
                continue;
            }
        }
        if c == '*' && i + 1 < chars.len() && chars[i + 1] != ' ' {
            let rest = &chars[i + 1..];
            if let Some(end) = rest.iter().position(|&x| x == '*') {
                if end > 0 {
                    flush(&mut buf, &mut out);
                    out.push(Span::Italic(rest[..end].iter().collect()));
                    i += end + 2;
                    continue;
                }
            }
        }
        if c == '[' {
            let rest = &chars[i + 1..];
            if let Some(close) = rest.iter().position(|&x| x == ']') {
                if rest.get(close + 1) == Some(&'(') {
                    if let Some(pclose) = rest[close + 2..].iter().position(|&x| x == ')') {
                        flush(&mut buf, &mut out);
                        let text: String = rest[..close].iter().collect();
                        let url: String = rest[close + 2..close + 2 + pclose].iter().collect();
                        out.push(Span::Link { text, url });
                        i += 1 + close + 2 + pclose + 1;
                        continue;
                    }
                }
            }
        }
        buf.push(c);
        i += 1;
    }
    flush(&mut buf, &mut out);
    out
}

fn find_seq(hay: &[char], needle: &[char]) -> Option<usize> {
    if hay.len() < needle.len() {
        return None;
    }
    (0..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

enum Block {
    Heading(u8, String),
    Paragraph(String),
    Bullet(u8, String),
    Blank,
}

fn parse_blocks(md: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut para = String::new();
    let mut in_html = false;
    let flush_para = |para: &mut String, blocks: &mut Vec<Block>| {
        let p = para.trim().to_string();
        if !p.is_empty() {
            blocks.push(Block::Paragraph(p));
        }
        para.clear();
    };
    for raw in md.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();
        // skip svelte/html tags that the web app embeds in the markdown
        if trimmed.starts_with('<') {
            flush_para(&mut para, &mut blocks);
            if !trimmed.ends_with('>') || trimmed.starts_with("<script") {
                in_html = true;
            }
            if trimmed.ends_with("</script>") || (trimmed.ends_with("/>") && !trimmed.starts_with("<script")) {
                in_html = false;
            }
            continue;
        }
        if in_html {
            if trimmed.ends_with("/>") || trimmed.ends_with("</script>") || trimmed.starts_with("/>") {
                in_html = false;
            }
            continue;
        }
        if trimmed.starts_with("{#") || trimmed.starts_with("{/") || trimmed.starts_with("{:") {
            continue;
        }
        if trimmed.is_empty() {
            flush_para(&mut para, &mut blocks);
            blocks.push(Block::Blank);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            flush_para(&mut para, &mut blocks);
            blocks.push(Block::Heading(3, rest.to_string()));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            flush_para(&mut para, &mut blocks);
            blocks.push(Block::Heading(2, rest.to_string()));
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            flush_para(&mut para, &mut blocks);
            blocks.push(Block::Heading(1, rest.to_string()));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            flush_para(&mut para, &mut blocks);
            let indent = (line.len() - trimmed.len()) as u8 / 2;
            blocks.push(Block::Bullet(indent.min(3), trimmed[2..].to_string()));
        } else {
            if !para.is_empty() {
                para.push('\n');
            }
            para.push_str(trimmed);
        }
    }
    flush_para(&mut para, &mut blocks);
    blocks
}

/// text with inline styling, wrapped to `width`
fn inline_job(spans: &[Span], size: f32, color: Color32, width: f32, line_height: f32, t: &crate::theme::Theme) -> (LayoutJob, Vec<(usize, usize, String)>) {
    let mut job = LayoutJob::default();
    job.wrap.max_width = width;
    let mut links = Vec::new();
    for span in spans {
        let (text, mut fmt) = match span {
            Span::Text(s) => (s.clone(), TextFormat::simple(theme::regular(size), color)),
            Span::Bold(s) => (s.clone(), TextFormat::simple(theme::bold(size), color)),
            Span::Italic(s) => {
                let mut f = TextFormat::simple(theme::regular(size), color);
                f.italics = true;
                (s.clone(), f)
            }
            Span::Code(s) => {
                let mut f = TextFormat::simple(theme::regular(size), color);
                f.background = t.inset;
                (s.clone(), f)
            }
            Span::Link { text, url } => {
                let mut f = TextFormat::simple(theme::regular(size), color);
                f.underline = egui::Stroke::new(1.0, color);
                let start = job.text.len();
                links.push((start, start + text.len(), url.clone()));
                (text.clone(), f)
            }
        };
        fmt.line_height = Some(size * line_height);
        job.append(&text, 0.0, fmt);
    }
    (job, links)
}

fn paragraph(ui: &mut Ui, c: &Ctx, text: &str, size: f32, line_height: f32) {
    let width = ui.available_width().max(60.0);
    let spans = parse_inline(text);
    let (job, links) = inline_job(&spans, size, c.t().text, width, line_height, c.t());
    let galley = ui.painter().layout_job(job);
    let (rect, resp) = ui.allocate_exact_size(galley.size(), Sense::click());
    ui.painter().galley(rect.min, galley.clone(), c.t().text);
    if !links.is_empty() {
        if let Some(pos) = resp.hover_pos() {
            let local = pos - rect.min;
            let cursor = galley.cursor_from_pos(local);
            let idx = cursor.index;
            for (start, end, url) in &links {
                let (cs, ce) = (byte_to_char(&galley.text(), *start), byte_to_char(&galley.text(), *end));
                if idx.0 >= cs && idx.0 < ce {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    if resp.clicked() {
                        open_link(url);
                    }
                }
            }
        }
    }
}

fn byte_to_char(s: &str, byte: usize) -> usize {
    s[..byte.min(s.len())].chars().count()
}

fn open_link(url: &str) {
    if url.starts_with("http") {
        let _ = open::that_detached(url);
    } else if url.starts_with('/') {
        // internal link (e.g. /settings/privacy) - open on cobalt.tools
        let _ = open::that_detached(format!("https://cobalt.tools{url}"));
    }
}

/// renders markdown as the `.long-text` style (14.5px, line-height 1.8)
pub fn render(ui: &mut Ui, c: &Ctx, md: &str) {
    render_sized(ui, c, md, 14.5, 1.8);
}

pub fn render_sized(ui: &mut Ui, c: &Ctx, md: &str, size: f32, line_height: f32) {
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
    let blocks = parse_blocks(md);
    let mut last_blank = true;
    for block in blocks {
        match block {
            Block::Blank => {
                if !last_blank {
                    ui.add_space(size * 0.9);
                }
                last_blank = true;
            }
            Block::Heading(level, text) => {
                let (fsize, top) = match level {
                    1 => (24.0, 12.0),
                    2 => (19.0, 10.0),
                    _ => (17.0, 8.0),
                };
                if !last_blank {
                    ui.add_space(top);
                }
                widgets::text(ui, strip_inline(&text), theme::medium(fsize), c.t().text);
                if level == 2 {
                    ui.add_space(6.0);
                    let w = ui.available_width();
                    let (r, _) = ui.allocate_exact_size(Vec2::new(w, 1.5), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, c.t().border_strong);
                }
                ui.add_space(8.0);
                last_blank = false;
            }
            Block::Paragraph(text) => {
                paragraph(ui, c, &text, size, line_height);
                last_blank = false;
            }
            Block::Bullet(indent, text) => {
                ui.horizontal_top(|ui| {
                    ui.add_space(16.0 + indent as f32 * 18.0);
                    ui.allocate_ui_with_layout(Vec2::new(14.0, size * line_height), egui::Layout::left_to_right(Align::Center), |ui| {
                        let (r, _) = ui.allocate_exact_size(Vec2::new(14.0, size * line_height), Sense::hover());
                        ui.painter().circle_filled(egui::pos2(r.left() + 3.0, r.center().y), 2.2, c.t().text);
                    });
                    ui.add_space(4.0);
                    ui.vertical(|ui| paragraph(ui, c, &text, size, line_height));
                });
                last_blank = false;
            }
        }
    }
}

fn strip_inline(s: &str) -> String {
    parse_inline(s)
        .into_iter()
        .map(|sp| match sp {
            Span::Text(t) | Span::Bold(t) | Span::Italic(t) | Span::Code(t) => t,
            Span::Link { text, .. } => text,
        })
        .collect()
}
