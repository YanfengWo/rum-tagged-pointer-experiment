use std::fs;
use std::path::{Path, PathBuf};

pub fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn seed_root() -> PathBuf {
    crate_root().join("seed/ham-import")
}

pub fn golden_dir() -> PathBuf {
    seed_root().join("golden")
}

pub fn curated_wolfram_core_dir() -> PathBuf {
    crate_root().join("cases/wolfram/core")
}

pub fn stateful_dir() -> PathBuf {
    crate_root().join("cases/stateful")
}

pub fn lane_a_default_fixture_paths() -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = text_files_in(golden_dir())?;
    files.extend(text_files_in(curated_wolfram_core_dir())?);
    files.sort();
    Ok(files)
}

pub fn lane_b_default_fixture_paths() -> Result<Vec<PathBuf>, std::io::Error> {
    text_files_in(stateful_dir())
}

pub fn load_text(path: impl AsRef<Path>) -> std::io::Result<String> {
    fs::read_to_string(path)
}

pub(super) fn text_files_in(dir: PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .filter(|entry| {
            entry.as_ref().map_or(true, |path| {
                path.extension().and_then(|ext| ext.to_str()) == Some("txt")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    files.sort();
    Ok(files)
}
