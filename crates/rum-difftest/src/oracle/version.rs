use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::OracleError;

const VERSION_MARKER: &str = "__RUM_DIFFTEST_VERSION__=";
const ORACLE_VERSION_WRAPPER: &str = r#"WriteString[$Output, "__RUM_DIFFTEST_VERSION__=", $Version, "\n"];
Exit[];
"#;

pub fn query_kernel_version(executable: &Path, temp_dir: &Path) -> Result<String, OracleError> {
    let wrapper_path = temp_dir.join("oracle_version.wls");
    fs::write(&wrapper_path, ORACLE_VERSION_WRAPPER).map_err(|source| OracleError::Io {
        path: wrapper_path.clone(),
        source,
    })?;

    let output = Command::new(executable)
        .arg("-noprompt")
        .arg("-script")
        .arg(&wrapper_path)
        .output()
        .map_err(|source| OracleError::Spawn {
            executable: executable.to_path_buf(),
            source,
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        return Err(OracleError::ExecutionFailed {
            executable: executable.to_path_buf(),
            code: output.status.code(),
            stdout,
            stderr,
        });
    }

    parse_version_stdout(executable, &stdout, &stderr)
}

fn parse_version_stdout(
    executable: &Path,
    stdout: &str,
    stderr: &str,
) -> Result<String, OracleError> {
    for line in stdout.lines() {
        if let Some(version) = line.strip_prefix(VERSION_MARKER) {
            return Ok(version.trim().to_owned());
        }
    }

    Err(OracleError::MissingResultMarker {
        executable: PathBuf::from(executable),
        stdout: stdout.to_owned(),
        stderr: stderr.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_marked_version_output() {
        let version = parse_version_stdout(
            Path::new("WolframKernel"),
            "__RUM_DIFFTEST_VERSION__=14.3.0 for Mac OS X ARM (64-bit)\n",
            "",
        )
        .unwrap();

        assert_eq!(version, "14.3.0 for Mac OS X ARM (64-bit)");
    }

    #[test]
    fn rejects_missing_version_marker() {
        let err = parse_version_stdout(Path::new("WolframKernel"), "banner\n", "").unwrap_err();

        assert!(matches!(err, OracleError::MissingResultMarker { .. }));
    }
}
