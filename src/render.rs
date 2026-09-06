//! From a layout dump to text. The wireframe is for the shape of the page,
//! the list is for the numbers, the findings are the things a pair of eyes
//! would have caught.

use std::collections::BTreeMap;
use std::fmt::Write;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Page {
    pub title: String,
    pub width: f64,
    pub height: f64,
    #[serde(rename = "docWidth")]
    pub doc_width: f64,
    #[serde(rename = "docHeight")]
    pub doc_height: f64,
    #[serde(default)]
    pub lang: String,
    pub boxes: Vec<Box_>,
}

#[derive(Deserialize, Clone)]
pub struct Box_ {
    pub parent: i64,
    pub depth: usize,
    pub tag: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub cls: Vec<String>,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub display: String,
    #[serde(default)]
    pub position: String,
    #[serde(rename = "fontSize", default)]
    pub font_size: f64,
    #[serde(rename = "fontWeight", default)]
    pub font_weight: u32,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub bg: String,
    #[serde(rename = "effBg", default)]
    pub eff_bg: String,
    #[serde(default)]
    pub border: bool,
    #[serde(default)]
    pub overflow: String,
    #[serde(default)]
    pub clipped: bool,
    #[serde(default)]
    pub pad: [f64; 4],
    #[serde(default)]
    pub margin: [f64; 4],
    #[serde(default)]
    pub gap: f64,
    #[serde(default)]
    pub alt: Option<String>,
}

pub struct Options {
    /// Characters across the wireframe. One character is `width / cols` CSS px.
    pub cols: usize,
    /// Wireframe rows are capped so a long page does not become a wall.
    pub max_rows: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self { cols: 100, max_rows: 240 }
    }
}

const CONTAINERS: &[&str] = &[
    "header", "nav", "main", "section", "article", "aside", "footer", "form", "fieldset", "table",
    "ul", "ol", "dl", "dialog", "figure", "details", "blockquote", "pre",
];
const TEXTISH: &[&str] = &["h1", "h2", "h3", "h4", "h5", "h6", "p", "li", "label", "dt", "dd", "caption", "summary", "legend"];
const WIDGETS: &[&str] = &["button", "input", "select", "textarea", "img", "video", "canvas", "svg", "iframe"];

impl Box_ {
    fn name(&self) -> String {
        let mut s = self.tag.clone();
        if !self.id.is_empty() {
            s.push('#');
            s.push_str(&self.id);
        }
        for c in &self.cls {
            s.push('.');
            s.push_str(c);
        }
        s
    }
    fn is_block(&self) -> bool {
        !self.display.starts_with("inline") || self.display == "inline-block" || self.display == "inline-flex" || self.display == "inline-grid"
    }
    fn right(&self) -> f64 {
        self.x + self.w
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
    fn is_textish(&self) -> bool {
        TEXTISH.contains(&self.tag.as_str())
    }
    /// Which boxes get drawn. Containers, widgets, headings and paragraphs,
    /// plus anything that painted its own edge or background.
    fn drawn(&self) -> bool {
        if self.w < 1.0 || self.h < 1.0 {
            return false;
        }
        let t = self.tag.as_str();
        CONTAINERS.contains(&t) || WIDGETS.contains(&t) || self.is_textish() || self.border || !self.bg.is_empty()
            || (t == "a" && self.is_block())
    }
    /// Text that is actually on the page: hidden `<option>`s and closed
    /// `<details>` bodies measure 0×0 and are left out.
    fn has_text(&self) -> bool {
        !self.text.is_empty() && self.w > 0.0 && self.h > 0.0
    }
}

// ---------- colour ----------

/// Computed colours arrive as `rgb()`/`rgba()`, or as `oklch()`/`oklab()` when the
/// stylesheet wrote them that way. Everything is brought back to 0..255 sRGB.
fn parse_rgb(s: &str) -> Option<([f64; 3], f64)> {
    let s = s.trim();
    let (kind, inner) = s.split_once('(')?;
    let inner = inner.strip_suffix(')')?;
    let parts: Vec<f64> = inner
        .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.trim_end_matches('%').parse().ok())
        .collect();
    if parts.len() < 3 {
        return None;
    }
    let alpha = *parts.get(3).unwrap_or(&1.0);
    match kind {
        "rgb" | "rgba" => Some(([parts[0], parts[1], parts[2]], alpha)),
        "oklab" => Some((oklab_to_srgb(parts[0], parts[1], parts[2]), alpha)),
        "oklch" => {
            let (l, c, h) = (parts[0], parts[1], parts[2].to_radians());
            Some((oklab_to_srgb(l, c * h.cos(), c * h.sin()), alpha))
        },
        _ => None,
    }
}

fn oklab_to_srgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;
    let (l3, m3, s3) = (l_.powi(3), m_.powi(3), s_.powi(3));
    let lin = [
        4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3,
        -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_614_8 * m3 + 1.707_614_701_0 * s3,
    ];
    lin.map(|c| {
        let c = c.clamp(0.0, 1.0);
        let srgb = if c <= 0.003_130_8 { 12.92 * c } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
        (srgb * 255.0).round()
    })
}

fn hex(s: &str) -> String {
    match parse_rgb(s) {
        Some(([r, g, b], a)) if a >= 1.0 => format!("#{:02x}{:02x}{:02x}", r as u8, g as u8, b as u8),
        Some(([r, g, b], a)) => format!("#{:02x}{:02x}{:02x}/{:.2}", r as u8, g as u8, b as u8, a),
        None => s.to_string(),
    }
}

fn luminance([r, g, b]: [f64; 3]) -> f64 {
    let f = |c: f64| {
        let c = c / 255.0;
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
}

fn contrast(fg: &str, bg: &str) -> Option<f64> {
    let (f, fa) = parse_rgb(fg)?;
    let (b, _) = parse_rgb(bg)?;
    // Text drawn with alpha is blended onto its background first.
    let f = [0, 1, 2].map(|k| f[k] * fa + b[k] * (1.0 - fa));
    let (l1, l2) = (luminance(f), luminance(b));
    let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    Some((hi + 0.05) / (lo + 0.05))
}

// ---------- wireframe ----------

/// Shorten a label to `max` cells; a quoted text keeps its closing quote so a
/// cut is visible as a cut, not as a different word.
fn cells(s: &str) -> usize {
    s.chars().map(|c| if is_wide(c) { 2 } else { 1 }).sum()
}

fn fit(text: &str, max: usize) -> String {
    if cells(text) <= max {
        return text.to_string();
    }
    let tail = if text.ends_with('"') { "…\"" } else { "…" };
    if max <= tail.len() + 1 {
        return text.chars().take(max).collect();
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = if is_wide(ch) { 2 } else { 1 };
        if used + w > max - tail.len() {
            break;
        }
        out.push(ch);
        used += w;
    }
    out + tail
}

fn is_wide(ch: char) -> bool {
    matches!(ch as u32,
        0x1100..=0x115F | 0x2E80..=0x303E | 0x3041..=0x33FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF | 0xFE30..=0xFE4F | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6 | 0x1F300..=0x1F64F | 0x1F900..=0x1F9FF | 0x20000..=0x3FFFD)
}

struct Canvas {
    cols: usize,
    rows: usize,
    cells: Vec<char>,
}

impl Canvas {
    fn new(cols: usize, rows: usize) -> Self {
        Self { cols, rows, cells: vec![' '; cols * rows] }
    }
    fn put(&mut self, col: usize, row: usize, ch: char) {
        if col < self.cols && row < self.rows {
            self.cells[row * self.cols + col] = ch;
        }
    }
    fn rect(&mut self, c0: usize, r0: usize, c1: usize, r1: usize, light: bool) {
        let (h, v, tl, tr, bl, br) = if light {
            ('·', '·', '·', '·', '·', '·')
        } else {
            ('─', '│', '┌', '┐', '└', '┘')
        };
        if c1 <= c0 || r1 <= r0 {
            return;
        }
        for c in c0..=c1 {
            self.put(c, r0, h);
            self.put(c, r1, h);
        }
        for r in r0..=r1 {
            self.put(c0, r, v);
            self.put(c1, r, v);
        }
        self.put(c0, r0, tl);
        self.put(c1, r0, tr);
        self.put(c0, r1, bl);
        self.put(c1, r1, br);
    }
    /// Wide (CJK) glyphs take two cells; the second is a marker that prints nothing.
    fn label(&mut self, col: usize, row: usize, max: usize, text: &str, filler: char) {
        // A deeper box drawn at the same corner overwrites the start of its parent's
        // name; clear what would otherwise remain as a tail.
        for k in 0..max {
            if let Some(cell) = self.cells.get_mut(row * self.cols + col + k) {
                if !"─│┌┐└┘·╌".contains(*cell) {
                    *cell = filler;
                }
            }
        }
        // ...and the head of a name we are about to step into the middle of.
        let mut k = col;
        while k > 0 {
            k -= 1;
            let cell = &mut self.cells[row * self.cols + k];
            if *cell == ' ' || "─│┌┐└┘·╌".contains(*cell) {
                break;
            }
            *cell = filler;
        }
        let text = fit(text, max);
        let mut k = 0;
        for ch in text.chars() {
            let w = if is_wide(ch) { 2 } else { 1 };
            if k + w > max {
                break;
            }
            self.put(col + k, row, ch);
            if w == 2 {
                self.put(col + k + 1, row, '\0');
            }
            k += w;
        }
    }
    fn to_string(&self) -> String {
        let mut s = String::new();
        for r in 0..self.rows {
            let line: String = self.cells[r * self.cols..(r + 1) * self.cols].iter().filter(|c| **c != '\0').collect();
            s.push_str(line.trim_end());
            s.push('\n');
        }
        s
    }
}

fn wireframe(page: &Page, opts: &Options, out: &mut String) {
    let cell_w = page.width / opts.cols as f64;
    let cell_h = cell_w * 2.0; // terminal glyphs are about twice as tall as wide
    let full_rows = (page.doc_height / cell_h).ceil() as usize;
    let rows = full_rows.min(opts.max_rows).max(1);
    let mut canvas = Canvas::new(opts.cols + 1, rows + 1);
    let col = |x: f64| (x / cell_w).round() as usize;
    let row = |y: f64| (y / cell_h).round() as usize;

    let vh_row = row(page.height);
    if vh_row < rows {
        for c in 0..=opts.cols {
            canvas.put(c, vh_row, '╌');
        }
    }

    let drawn: Vec<&Box_> = page.boxes.iter().filter(|b| b.drawn()).collect();
    let geo = |b: &Box_| (col(b.x), row(b.y), col(b.right()), row(b.bottom()));

    // Frames first, labels after, so a child's edge never eats its parent's name.
    for b in &drawn {
        let (c0, r0, c1, r1) = geo(b);
        if r1 > r0 && c1.saturating_sub(c0) >= 2 {
            canvas.rect(c0, r0, c1, r1, b.is_textish() || b.tag == "a");
        }
    }
    for b in &drawn {
        let (c0, r0, c1, r1) = geo(b);
        let light = b.is_textish() || b.tag == "a";
        let width = c1.saturating_sub(c0);
        let room = if r1 == r0 || width < 2 { width.max(1) } else { width - 1 };
        // Full name, then just the words, then just the tag: the first that fits.
        let mut candidates = vec![];
        if b.has_text() && (light || WIDGETS.contains(&b.tag.as_str())) {
            candidates.push(format!("{} {:?}", b.tag, b.text));
            candidates.push(format!("{:?}", b.text));
        } else if let Some(alt) = &b.alt {
            candidates.push(format!("img[alt={alt:?}]"));
        }
        candidates.push(b.name());
        candidates.push(b.tag.clone());
        let label = candidates.iter().find(|c| cells(c) <= room).cloned().unwrap_or_else(|| fit(&b.tag, room));

        if r1 == r0 || width < 2 {
            // Too thin to frame: a line of label is still worth more than nothing.
            canvas.label(c0, r0, room, &label, ' ');
        } else if light {
            // Text boxes carry their words inside.
            canvas.label(c0 + 1, r0 + if r1 - r0 >= 2 { 1 } else { 0 }, room, &label, ' ');
        } else {
            // Containers wear their name on the top edge, leaving the inside to children.
            canvas.label(c0 + 1, r0, room, &label, '─');
        }
    }

    let _ = writeln!(out, "## Wireframe  (1 char = {:.0}x{:.0} px, `╌` = fold of the first screen)", cell_w, cell_h);
    out.push_str("```\n");
    out.push_str(&canvas.to_string());
    out.push_str("```\n");
    if full_rows > rows {
        let _ = writeln!(out, "(page continues: {} more rows, {:.0} px)", full_rows - rows, page.doc_height - rows as f64 * cell_h);
    }
    out.push('\n');
}

// ---------- element list ----------

fn element_list(page: &Page, out: &mut String) {
    out.push_str("## Elements  (x y w×h, then type and colour; contrast is text on its effective background)\n```\n");
    for b in &page.boxes {
        if !(b.drawn() || b.has_text()) {
            continue;
        }
        let indent = "  ".repeat(b.depth.min(12));
        let mut line = format!("{indent}{:<24}", b.name());
        let _ = write!(line, " {:>5} {:>5} {:>5}×{:<5}", b.x, b.y, b.w, b.h);
        if b.is_block() {
            let _ = write!(line, " {}", short_display(&b.display));
        }
        if b.position != "static" {
            let _ = write!(line, " {}", b.position);
        }
        if !b.bg.is_empty() {
            let _ = write!(line, " bg {}", hex(&b.bg));
        }
        if b.has_text() {
            let _ = write!(line, " fs{} ", b.font_size);
            if b.font_weight >= 600 {
                let _ = write!(line, "bold ");
            }
            let _ = write!(line, "{}", hex(&b.color));
            if let Some(c) = contrast(&b.color, &b.eff_bg) {
                let _ = write!(line, " on {} ({:.1}:1)", hex(&b.eff_bg), c);
            }
            let _ = write!(line, " {:?}", b.text);
        } else if let Some(alt) = &b.alt {
            let _ = write!(line, " alt={alt:?}");
        }
        if b.clipped {
            line.push_str(" CLIPPED");
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("```\n\n");
}

fn short_display(d: &str) -> &str {
    match d {
        "block" => "block",
        "flex" | "inline-flex" => "flex",
        "grid" | "inline-grid" => "grid",
        "list-item" => "li",
        "inline-block" => "inline-block",
        other => other,
    }
}

// ---------- findings ----------

fn findings(page: &Page, out: &mut String) {
    let mut notes: Vec<String> = Vec::new();
    let boxes = &page.boxes;

    if page.doc_width > page.width + 1.0 {
        notes.push(format!(
            "Horizontal scroll: document is {:.0} px wide in a {:.0} px viewport.",
            page.doc_width, page.width
        ));
    }

    // Past the right edge of the viewport.
    let mut past: Vec<&Box_> = boxes.iter().filter(|b| b.is_block() && b.w > 0.0 && b.right() > page.width + 1.0).collect();
    past.sort_by(|a, b| b.right().partial_cmp(&a.right()).unwrap());
    for b in past.iter().take(6) {
        notes.push(format!("`{}` reaches x={:.0}, past the viewport edge at {:.0}.", b.name(), b.right(), page.width));
    }

    // Children sticking out of a parent that does not clip.
    let mut spill = 0;
    for b in boxes.iter().filter(|b| b.parent >= 0 && b.is_block() && b.w > 0.0 && b.h > 0.0 && b.position != "absolute" && b.position != "fixed") {
        let p = &boxes[b.parent as usize];
        if p.overflow != "visible" || p.w <= 0.0 {
            continue;
        }
        let dx = (b.right() - p.right()).max(p.x - b.x);
        let dy = (b.bottom() - p.bottom()).max(p.y - b.y);
        if dx > 1.0 || dy > 1.0 {
            spill += 1;
            if spill <= 6 {
                let axis = if dx > dy { format!("{dx:.0} px horizontally") } else { format!("{dy:.0} px vertically") };
                notes.push(format!("`{}` spills out of `{}` by {}.", b.name(), p.name(), axis));
            }
        }
    }
    if spill > 6 {
        notes.push(format!("(and {} more spills)", spill - 6));
    }

    // Siblings that overlap.
    let mut overlaps = 0;
    for (k, a) in boxes.iter().enumerate() {
        if !a.drawn() || a.position == "absolute" || a.position == "fixed" {
            continue;
        }
        for b in boxes[k + 1..].iter() {
            if b.parent != a.parent || !b.drawn() || b.position == "absolute" || b.position == "fixed" {
                continue;
            }
            let ox = a.right().min(b.right()) - a.x.max(b.x);
            let oy = a.bottom().min(b.bottom()) - a.y.max(b.y);
            if ox > 2.0 && oy > 2.0 {
                overlaps += 1;
                if overlaps <= 6 {
                    notes.push(format!("`{}` and `{}` overlap by {:.0}×{:.0} px.", a.name(), b.name(), ox, oy));
                }
            }
        }
    }
    if overlaps > 6 {
        notes.push(format!("(and {} more overlaps)", overlaps - 6));
    }

    // Clipped content.
    for b in boxes.iter().filter(|b| b.clipped).take(6) {
        notes.push(format!("`{}` clips its content (overflow: {}).", b.name(), b.overflow));
    }

    // Contrast, WCAG AA: 4.5 for body text, 3 for large text.
    let mut low: Vec<(f64, &Box_)> = boxes
        .iter()
        .filter(|b| b.has_text())
        .filter_map(|b| contrast(&b.color, &b.eff_bg).map(|c| (c, b)))
        .filter(|(c, b)| {
            let large = b.font_size >= 24.0 || (b.font_size >= 19.0 && b.font_weight >= 700);
            *c < if large { 3.0 } else { 4.5 }
        })
        .collect();
    low.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (c, b) in low.iter().take(8) {
        notes.push(format!(
            "Low contrast {:.1}:1 on `{}` ({} on {}) {:?}.",
            c, b.name(), hex(&b.color), hex(&b.eff_bg), b.text
        ));
    }
    if low.len() > 8 {
        notes.push(format!("(and {} more low-contrast texts)", low.len() - 8));
    }

    // Small text.
    let tiny: Vec<&Box_> = boxes.iter().filter(|b| b.has_text() && b.font_size > 0.0 && b.font_size < 12.0).collect();
    if !tiny.is_empty() {
        let names: Vec<String> = tiny.iter().take(5).map(|b| format!("`{}` ({}px)", b.name(), b.font_size)).collect();
        notes.push(format!("Text under 12 px: {}{}.", names.join(", "), if tiny.len() > 5 { ", ..." } else { "" }));
    }

    // Images without alt.
    let noalt = boxes.iter().filter(|b| b.tag == "img" && b.alt.is_none()).count();
    if noalt > 0 {
        notes.push(format!("{noalt} image(s) without an alt attribute."));
    }

    // Small tap targets.
    let small: Vec<&Box_> = boxes
        .iter()
        .filter(|b| matches!(b.tag.as_str(), "button" | "a" | "input" | "select") && b.w > 0.0 && b.h > 0.0 && (b.w < 24.0 || b.h < 24.0))
        .collect();
    if !small.is_empty() {
        let names: Vec<String> = small.iter().take(5).map(|b| format!("`{}` {:.0}×{:.0}", b.name(), b.w, b.h)).collect();
        notes.push(format!("Targets under 24 px: {}{}.", names.join(", "), if small.len() > 5 { ", ..." } else { "" }));
    }

    out.push_str("## Findings\n");
    if notes.is_empty() {
        out.push_str("Nothing measurable stood out. (That is not the same as looking good.)\n");
    } else {
        for n in notes {
            let _ = writeln!(out, "- {n}");
        }
    }
    out.push('\n');
}

// ---------- palettes ----------

fn palettes(page: &Page, out: &mut String) {
    let mut sizes: BTreeMap<u64, usize> = BTreeMap::new();
    let mut colors: BTreeMap<String, usize> = BTreeMap::new();
    let mut bgs: BTreeMap<String, usize> = BTreeMap::new();
    let mut spacing: BTreeMap<u64, usize> = BTreeMap::new();
    for b in &page.boxes {
        if b.has_text() {
            *sizes.entry(b.font_size as u64).or_default() += 1;
            *colors.entry(hex(&b.color)).or_default() += 1;
        }
        if !b.bg.is_empty() {
            *bgs.entry(hex(&b.bg)).or_default() += 1;
        }
        if b.is_block() && b.w > 0.0 {
            for v in b.pad.iter().chain(b.margin.iter()).chain(std::iter::once(&b.gap)) {
                if *v > 0.0 {
                    *spacing.entry(*v as u64).or_default() += 1;
                }
            }
        }
    }
    let fmt = |m: &BTreeMap<u64, usize>| m.iter().map(|(k, n)| format!("{k}({n})")).collect::<Vec<_>>().join(" ");
    let fmt_s = |m: &BTreeMap<String, usize>| {
        let mut v: Vec<_> = m.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        v.iter().take(12).map(|(k, n)| format!("{k}({n})")).collect::<Vec<_>>().join(" ")
    };
    out.push_str("## Palette  (value(count))\n");
    let _ = writeln!(out, "- font sizes: {}", fmt(&sizes));
    let _ = writeln!(out, "- text colours: {}", fmt_s(&colors));
    let _ = writeln!(out, "- backgrounds: {}", fmt_s(&bgs));
    let _ = writeln!(out, "- spacing (padding/margin/gap): {}", fmt(&spacing));
    let off_grid: Vec<String> = spacing.keys().filter(|v| **v % 4 != 0).map(|v| v.to_string()).collect();
    if !off_grid.is_empty() {
        let _ = writeln!(out, "- off the 4 px grid: {}", off_grid.join(" "));
    }
}

pub fn render(page: &Page) -> String {
    render_with(page, &Options::default())
}

pub fn render_with(page: &Page, opts: &Options) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}", if page.title.is_empty() { "(untitled)" } else { &page.title });
    let _ = writeln!(
        out,
        "viewport {:.0}×{:.0}, document {:.0}×{:.0}, {} elements{}\n",
        page.width, page.height, page.doc_width, page.doc_height, page.boxes.len(),
        if page.lang.is_empty() { String::new() } else { format!(", lang={}", page.lang) }
    );
    wireframe(page, opts, &mut out);
    findings(page, &mut out);
    element_list(page, &mut out);
    palettes(page, &mut out);
    out
}
