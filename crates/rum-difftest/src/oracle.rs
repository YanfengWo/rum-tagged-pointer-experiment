use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

#[path = "oracle/kernel.rs"]
mod kernel;
#[path = "oracle/process.rs"]
mod process;
#[path = "oracle/temp_dir.rs"]
mod temp_dir;
#[path = "oracle/version.rs"]
mod version;

pub use kernel::OracleKernel;

pub const DEFAULT_ORACLE_TIMEOUT: Duration = Duration::from_secs(120);
pub const DEFAULT_ORACLE_OUTPUT_LIMIT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleEvaluation {
    pub result: String,
    pub printed_output: String,
    pub stderr: String,
}

pub trait Oracle {
    fn evaluate(&mut self, input: &str) -> Result<OracleEvaluation, OracleError>;

    fn evaluate_session(
        &mut self,
        inputs: &[String],
    ) -> Result<Vec<OracleEvaluation>, OracleError> {
        let mut outputs = Vec::with_capacity(inputs.len());
        for input in inputs {
            outputs.push(self.evaluate(input)?);
        }
        Ok(outputs)
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
    #[error("failed while waiting for oracle executable `{executable}`: {source}")]
    Wait {
        executable: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to terminate timed-out oracle executable `{executable}`: {source}")]
    Kill {
        executable: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to capture oracle executable `{executable}` {stream}: {source}")]
    OutputCapture {
        executable: PathBuf,
        stream: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("oracle executable `{executable}` {stream} reader panicked")]
    OutputReaderPanicked {
        executable: PathBuf,
        stream: &'static str,
    },
    #[error(
        "oracle executable `{executable}` timed out after {timeout_millis} ms\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    TimedOut {
        executable: PathBuf,
        timeout_millis: u128,
        stdout: String,
        stderr: String,
    },
    #[error(
        "oracle executable `{executable}` exceeded the {output_limit_bytes}-byte capture limit for {streams}\ntruncated stdout:\n{stdout}\ntruncated stderr:\n{stderr}"
    )]
    OutputLimitExceeded {
        executable: PathBuf,
        output_limit_bytes: usize,
        streams: &'static str,
        stdout: String,
        stderr: String,
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
    #[error(
        "oracle executable `{executable}` emitted an invalid session result marker `{line}`\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    InvalidSessionResultMarker {
        executable: PathBuf,
        line: String,
        stdout: String,
        stderr: String,
    },
    #[error(
        "oracle executable `{executable}` emitted {actual} session results for {expected} inputs\nstdout:\n{stdout}\nstderr:\n{stderr}"
    )]
    SessionResultCountMismatch {
        executable: PathBuf,
        expected: usize,
        actual: usize,
        stdout: String,
        stderr: String,
    },
}
