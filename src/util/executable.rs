use std::path::Path;

/// Whether `name` is an executable file somewhere on `$PATH`.
pub fn is_on_path(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|directory| is_executable(&directory.join(name)))
    })
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    const ANY_EXECUTE_BIT: u32 = 0o111;
    path.metadata().is_ok_and(|metadata| {
        metadata.is_file() && metadata.permissions().mode() & ANY_EXECUTE_BIT != 0
    })
}
