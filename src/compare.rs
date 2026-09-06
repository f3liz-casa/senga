//! Servo's painting against a picture from another browser, band by band.
//! Anti-aliasing differs everywhere and stays under a few percent; a band
//! that jumps is where the layouts disagree.

use std::fmt::Write;

use image::RgbaImage;

const BAND: u32 = 100;
/// Sum of the three channel differences above which a pixel counts as different.
const THRESHOLD: u32 = 60;

pub struct Comparison {
    pub width: u32,
    pub height: u32,
    pub differing: f64,
    pub bands: Vec<(u32, f64)>,
}

pub fn compare(ours: &RgbaImage, theirs: &RgbaImage) -> Comparison {
    let width = ours.width().min(theirs.width());
    let height = ours.height().min(theirs.height());
    let mut total = 0u64;
    let mut bands = Vec::new();
    let mut y0 = 0;
    while y0 < height {
        let y1 = (y0 + BAND).min(height);
        let mut count = 0u64;
        for y in y0..y1 {
            for x in 0..width {
                let a = ours.get_pixel(x, y);
                let b = theirs.get_pixel(x, y);
                let d: u32 = (0..3).map(|i| a[i].abs_diff(b[i]) as u32).sum();
                if d > THRESHOLD {
                    count += 1;
                }
            }
        }
        total += count;
        bands.push((y0, count as f64 * 100.0 / (width * (y1 - y0)) as f64));
        y0 = y1;
    }
    Comparison { width, height, differing: total as f64 * 100.0 / (width * height) as f64, bands }
}

pub fn report(c: &Comparison, ours: &RgbaImage, theirs: &RgbaImage) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## Compare  (Servo against the reference, pixels differing per {BAND} px band)");
    if ours.dimensions() != theirs.dimensions() {
        let _ = writeln!(
            out,
            "sizes differ: Servo {}×{}, reference {}×{}; compared the overlap {}×{}",
            ours.width(), ours.height(), theirs.width(), theirs.height(), c.width, c.height
        );
    }
    let _ = writeln!(out, "overall {:.1}% differing", c.differing);
    let worst = c.bands.iter().map(|b| b.1).fold(0.0, f64::max);
    out.push_str("```\n");
    for (y, pct) in &c.bands {
        let bar = "#".repeat((pct / 2.0).round() as usize);
        let mark = if *pct == worst && worst > 0.0 { "  <- worst" } else { "" };
        let _ = writeln!(out, "y {y:>5}  {pct:5.1}%  {bar}{mark}");
    }
    out.push_str("```\n");
    out.push_str("Under about 5% everywhere is font rendering. A band well above its neighbours is a layout difference; look there.\n");
    out
}

/// Servo on the left, the reference on the right, a red seam between.
pub fn side_by_side(ours: &RgbaImage, theirs: &RgbaImage) -> RgbaImage {
    let seam = 10;
    let width = ours.width() + seam + theirs.width();
    let height = ours.height().max(theirs.height());
    let mut out = RgbaImage::from_pixel(width, height, image::Rgba([255, 0, 0, 255]));
    image::imageops::overlay(&mut out, ours, 0, 0);
    image::imageops::overlay(&mut out, theirs, (ours.width() + seam) as i64, 0);
    out
}
