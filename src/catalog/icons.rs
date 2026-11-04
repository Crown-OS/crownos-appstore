use super::mirrors;
use crate::util::{
    cache,
    http::{Fetched, fetch},
    pixels::{Pixels, decode_png},
};

/// The published icon at `relative`, fitted into `edge` pixels: from the disk cache, else from the
/// first mirror that has it (and then cached).
pub fn load_icon(relative: &str, edge: u32) -> Option<Pixels> {
    let cached = cache::read(relative).or_else(|| download(relative))?;
    decode_png(&cached, edge)
}

fn download(relative: &str) -> Option<Vec<u8>> {
    let bytes = mirrors::urls(relative).find_map(|url| match fetch(&url, None) {
        Ok(Fetched::Body { bytes, .. }) => Some(bytes),
        _ => None,
    })?;
    let _ = cache::write(relative, &bytes);
    Some(bytes)
}
