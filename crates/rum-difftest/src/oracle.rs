use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

const RESULT_MARKER: &str = "__RUM_DIFFTEST_RESULT__=";
const ORACLE_WRAPPER: &str = r#"inputPath = Last[$CommandLine];
exprText = Import[inputPath, "Text"];
held = ToExpression[exprText, InputForm, HoldComplete];
result = ReleaseHold[held];
WriteString[$Output, "__RUM_DIFFTEST_RESULT__=", ToString[result, InputForm], "\n"];
Exit[];
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleEvaluation {
    pub result: String,
    pub printed_output: String,
    pub stderr: String,
}

pub trait Oracle {
    fn evaluate(&mut self, input: &str) -> Result<OracleEvaluation, OracleError>;
}

pub struct OracleKernel {
    executable: PathBuf,
    temp_dir: TempDir,
    wrapper_path: PathBuf,
    next_case_id: usize,
}

impl OracleKernel {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, OracleError> {
        let executable = executable.into();
        let temp_dir = TempDir::new("rum-difftest")?;
        let wrapper_path = temp_dir.path().join("oracle_wrapper.wls");
        fs::write(&wrapper_path, ORACLE_WRAPPER).map_err(|source| OracleError::Io {
            path: wrapper_path.clone(),
            source,
        })?;

        Ok(Self {
            executable,
            temp_dir,
            wrapper_path,
            next_case_id: 0,
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }
}

impl Oracle for OracleKernel {
    fn evaluate(&mut self, input: &str) -> Result<OracleEvaluation, OracleError> {
        self.next_case_id += 1;
        let input_path = self
            .temp_dir
            .path()
            .join(format!("case-{:04}.wl", self.next_case_id));
        fs::write(&input_path, input).map_err(|source| OracleError::Io {
            path: input_path.clone(),
            source,
        })?;

        let output = Command::new(&self.executable)
            .arg("-noprompt")
            .arg("-script")
            .arg(&self.wrapper_path)
            .arg(&input_path)
            .output()
            .map_err(|source| OracleError::Spawn {
                executable: self.executable.clone(),
                source,
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        if !output.status.success() {
            return Err(OracleError::ExecutionFailed {
                executable: self.executable.clone(),
                code: output.status.code(),
                stdout,
                stderr,
            });
        }

        let mut result = None;
        let mut printed_lines = Vec::new();

        for line in stdout.lines() {
            if let Some(value) = line.strip_prefix(RESULT_MARKER) {
                result = Some(value.trim().to_owned());
            } else {
                printed_lines.push(line.to_owned());
            }
        }

        let Some(result) = result else {
            return Err(OracleError::MissingResultMarker {
                executable: self.executable.clone(),
                stdout,
                stderr,
            });
        };

        Ok(OracleEvaluation {
            result,
            printed_output: printed_lines.join("\n").trim().to_owned(),
            stderr: stderr.trim().to_owned(),
        })
    }
}

#[derive(Debug, Error)]
pub enum OracleError {
    #[error("failed to create temporary oracle workspace: {0}")]
    TempDir(#[from] std::io::Error),
    #[error("failed to write `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to spawn oracle executable `{executable}`: {source}")]
    Spawn {
        executable: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "oracle executable `{executable}` failed with status {code:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    ExecutionFailed {
        executable: PathBuf,
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    #[error(
        "oracle executable `{executable}` did not emit a result marker\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    MissingResultMarker {
        executable: PathBuf,
        stdout: String,
        stderr: String,
    },
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Result<Self, std::io::Error> {
        let base = std::env::temp_dir();
        let pid = std::process::id();

        for attempt in 0..100u32 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let candidate = base.join(format!("{prefix}-{pid}-{now}-{attempt}"));
            match fs::create_dir(&candidate) {
                Ok(()) => return Ok(Self { path: candidate }),
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(err),
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not allocate a unique temporary directory",
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
