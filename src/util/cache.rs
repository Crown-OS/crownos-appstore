use std::{
    fs, io,
    path::{Path, PathBuf},
};

const APP_DIRECTORY: &str = "crownos/appstore";

/// `$XDG_CACHE_HOME/crownos/appstore`, or `~/.cache/crownos/appstore`.
pub fn cache_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))?;
    Some(base.join(APP_DIRECTORY))
}

pub fn read(name: &str) -> Option<Vec<u8>> {
    fs::read(cache_dir()?.join(name)).ok()
}

/// Replaces `name` atomically, so a crash never leaves half a file behind.
pub fn write(name: &str, bytes: &[u8]) -> io::Result<()> {
    let path = cache_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no cache directory"))?
        .join(name);
    write_atomically(&path, bytes)
}

fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let partial = path.with_extension("partial");
    fs::write(&partial, bytes)?;
    fs::rename(partial, path)
}
