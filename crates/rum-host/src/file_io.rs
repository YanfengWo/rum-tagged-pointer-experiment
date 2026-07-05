use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rum_runtime::kernel_api::{FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode};

/// Host filesystem provider backed by `std::fs`.
///
/// Trusted host sessions install this provider by default. Sandboxed sessions
/// omit it so file builtins fail at construction level if capability policy
/// has explicitly allowed them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StdFileIo;

impl StdFileIo {
    /// Creates a provider value.
    pub fn new() -> Self {
        Self
    }

    /// Returns a shared cloneable provider handle for host configuration.
    pub fn provider() -> FileIoProvider {
        static PROVIDER: OnceLock<FileIoProvider> = OnceLock::new();
        PROVIDER
            .get_or_init(|| FileIoProvider::new(Self::new()))
            .clone()
    }
}

impl FileIo for StdFileIo {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        fs::read_to_string(path).map_err(file_io_error)
    }

    fn probe(&self, path: &Path) -> FileProbe {
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => FileProbe::file(normalize_path(path)),
            Ok(_) => FileProbe::non_file(),
            Err(_) => FileProbe::missing(),
        }
    }

    fn write_string(
        &self,
        path: &Path,
        contents: &str,
        mode: FileWriteMode,
    ) -> Result<(), FileIoError> {
        let result = match mode {
            FileWriteMode::Replace => fs::write(path, contents),
            FileWriteMode::Append => fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .and_then(|mut file| file.write_all(contents.as_bytes())),
        };

        result.map_err(file_io_error)
    }
}

fn file_io_error(error: std::io::Error) -> FileIoError {
    FileIoError::new(error.to_string())
}

fn normalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    })
}
