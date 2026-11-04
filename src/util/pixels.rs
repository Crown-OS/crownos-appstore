use image::{ImageFormat, imageops::FilterType};

/// Straight RGBA8, ready for `Cx::register_image`.
pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Decodes a PNG and fits it into an `edge`-pixel square, never upscaling.
pub fn decode_png(bytes: &[u8], edge: u32) -> Option<Pixels> {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Png).ok()?;
    let fitted = if image.width().max(image.height()) > edge {
        image.resize(edge, edge, FilterType::Triangle)
    } else {
        image
    };
    let rgba = fitted.into_rgba8();
    let (width, height) = rgba.dimensions();
    Some(Pixels {
        width,
        height,
        rgba: rgba.into_raw(),
    })
}
