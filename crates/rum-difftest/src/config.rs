use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffTestConfig {
    pub oracle_kernel: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid config line `{line}`")]
    InvalidLine { line: String },
    #[error("unknown config key `{key}`")]
    UnknownKey { key: String },
}

pub fn load_config(path: impl AsRef<Path>) -> Result<DiffTestConfig, ConfigError> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_config(&text)
}

pub fn parse_config(text: &str) -> Result<DiffTestConfig, ConfigError> {
    let mut config = DiffTestConfig::default();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(ConfigError::InvalidLine {
                line: line.to_owned(),
            });
        };

        let key = key.trim();
        let value = unquote(value.trim());

        match key {
            "oracle_kernel" | "oracle_command" => {
                if !value.is_empty() {
                    config.oracle_kernel = Some(PathBuf::from(value));
                }
            }
            _ => {
                return Err(ConfigError::UnknownKey {
                    key: key.to_owned(),
                });
            }
        }
    }

    Ok(config)
}

pub fn resolve_oracle_kernel(
    cli_oracle_kernel: Option<PathBuf>,
    config: &DiffTestConfig,
) -> Option<PathBuf> {
    cli_oracle_kernel
        .or_else(|| config.oracle_kernel.clone())
        .or_else(|| env::var_os("RUM_DIFFTEST_ORACLE_KERNEL").map(PathBuf::from))
        .or_else(find_kernel_in_path)
        .or_else(find_common_kernel_path)
}

pub fn default_config_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("rum-difftest.conf")];

    if let Some(home) = env::var_os("HOME") {
        paths.push(
            PathBuf::from(home)
                .join(".config")
                .join("rum")
                .join("difftest.conf"),
        );
    }

    paths
}

pub fn load_optional_config(explicit_path: Option<&Path>) -> Result<DiffTestConfig, ConfigError> {
    if let Some(path) = explicit_path {
        return load_config(path);
    }

    for path in default_config_paths() {
        if path.exists() {
            return load_config(path);
        }
    }

    Ok(DiffTestConfig::default())
}

fn find_kernel_in_path() -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;

    for directory in env::split_paths(&path_var) {
        for candidate in ["WolframKernel", "MathKernel"] {
            let path = directory.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    None
}

fn find_common_kernel_path() -> Option<PathBuf> {
    find_linux_wolfram_kernel().or_else(|| {
        common_macos_kernel_paths()
            .into_iter()
            .find(|path| path.is_file())
    })
}

fn find_linux_wolfram_kernel() -> Option<PathBuf> {
    let root = Path::new("/usr/local/Wolfram/WolframEngine");
    let mut candidates = fs::read_dir(root)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.reverse();

    for candidate in candidates {
        let executable = candidate.join("Executables").join("WolframKernel");
        if executable.is_file() {
            return Some(executable);
        }

        let binary = candidate
            .join("SystemFiles")
            .join("Kernel")
            .join("Binaries")
            .join("Linux-x86-64")
            .join("WolframKernel");
        if binary.is_file() {
            return Some(binary);
        }
    }

    None
}

fn common_macos_kernel_paths() -> Vec<PathBuf> {
    [
        "/Applications/Wolfram.app/Contents/MacOS/WolframKernel",
        "/Applications/Mathematica.app/Contents/MacOS/WolframKernel",
        "/Applications/Mathematica.app/Contents/MacOS/MathKernel",
        "/Applications/Wolfram Engine.app/Contents/MacOS/WolframKernel",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect()
}

fn unquote(value: &str) -> String {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return value[1..value.len() - 1].to_owned();
        }
    }

    value.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_oracle_kernel_from_config() {
        let config = parse_config(
            r#"
                # comment
                oracle_kernel = "/tmp/MathKernel"
            "#,
        )
        .unwrap();

        assert_eq!(config.oracle_kernel, Some(PathBuf::from("/tmp/MathKernel")));
    }

    #[test]
    fn parses_oracle_command_alias_from_config() {
        let config = parse_config("oracle_command = 'WolframKernel'").unwrap();
        assert_eq!(config.oracle_kernel, Some(PathBuf::from("WolframKernel")));
    }

    #[test]
    fn common_macos_kernel_paths_include_wolfram_app() {
        assert!(common_macos_kernel_paths().contains(&PathBuf::from(
            "/Applications/Wolfram.app/Contents/MacOS/WolframKernel"
        )));
    }

    #[test]
    fn rejects_unknown_config_keys() {
        let err = parse_config("unexpected = 1").unwrap_err();
        assert!(matches!(err, ConfigError::UnknownKey { .. }));
    }

    #[test]
    fn rejects_oracle_version_pin_config_keys() {
        for key in ["oracle_version", "required_oracle_version"] {
            let err = parse_config(&format!("{key} = '14.3'")).unwrap_err();
            assert!(
                matches!(err, ConfigError::UnknownKey { key: ref unknown } if unknown == key),
                "expected `{key}` to remain unsupported, got {err:?}"
            );
        }
    }
}
