use std::path::Path;

use anyhow::{Context, Result, bail};
use image::{RgbaImage, imageops::FilterType};
use resvg::{tiny_skia, usvg};

/// Published icons are square PNGs this many pixels wide.
const EDGE: u32 = 256;

/// Rasterizes `source` (SVG or PNG) to an `EDGE`-pixel PNG at `target`.
pub fn publish(source: &Path, target: &Path) -> Result<()> {
    let bytes = std::fs::read(source).with_context(|| format!("read {}", source.display()))?;
    let pixels = match source.extension().and_then(|extension| extension.to_str()) {
        Some("svg") => rasterize_svg(&bytes)?,
        Some("png") => fit_png(&bytes)?,
        _ => bail!("{} must be .svg or .png", source.display()),
    };
    pixels
        .save(target)
        .with_context(|| format!("write {}", target.display()))
}

fn rasterize_svg(bytes: &[u8]) -> Result<RgbaImage> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default()).context("parse svg")?;
    let size = tree.size();
    let scale = EDGE as f32 / size.width().max(size.height()).max(1.0);
    let offset_x = (EDGE as f32 - size.width() * scale) / 2.0;
    let offset_y = (EDGE as f32 - size.height() * scale) / 2.0;
    let mut pixmap = tiny_skia::Pixmap::new(EDGE, EDGE).context("allocate pixmap")?;
    let transform =
        tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let straight = pixel.demultiply();
            [
                straight.red(),
                straight.green(),
                straight.blue(),
                straight.alpha(),
            ]
        })
        .collect();
    RgbaImage::from_raw(EDGE, EDGE, rgba).context("pixmap size")
}

fn fit_png(bytes: &[u8]) -> Result<RgbaImage> {
    let source = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .context("decode png")?
        .into_rgba8();
    if source.dimensions() == (EDGE, EDGE) {
        return Ok(source);
    }
    let fitted = image::imageops::resize(&source, EDGE, EDGE, FilterType::Lanczos3);
    Ok(fitted)
}
