use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

#[path = "oracle/temp_dir.rs"]
mod temp_dir;
#[path = "oracle/version.rs"]
mod version;

use temp_dir::TempDir;

const RESULT_MARKER: &str = "__RUM_DIFFTEST_RESULT__=";
const SESSION_RESULT_MARKER: &str = "__RUM_DIFFTEST_SESSION_RESULT__=";
const ORACLE_WRAPPER: &str = r#"inputPath = Last[$CommandLine];
exprText = Import[inputPath, "Text"];
result = Block[{$MessageList = {}}, ToExpression[exprText, InputForm]];
WriteString[$Output, "__RUM_DIFFTEST_RESULT__=", ToString[result, InputForm], "\n"];
Exit[];
"#;
const ORACLE_SESSION_WRAPPER: &str = r#"manifestPath = Last[$CommandLine];
inputPaths = Import[manifestPath, "Lines"];
sessionDir = DirectoryName[manifestPath];
Do[
  exprText = Import[inputPaths[[i]], "Text"];
  suffix = IntegerString[i, 10, 4];
  messagesPath = FileNameJoin[{sessionDir, "messages-" <> suffix <> ".txt"}];
  printsPath = FileNameJoin[{sessionDir, "prints-" <> suffix <> ".txt"}];
  messagesStream = OpenWrite[messagesPath, PageWidth -> Infinity];
  printsStream = OpenWrite[printsPath, PageWidth -> Infinity];
  result = Block[
    {$MessageList = {}, $Messages = {messagesStream}, $Output = {printsStream}},
    ToExpression[exprText, InputForm]
  ];
  Close[messagesStream];
  Close[printsStream];
  WriteString[$Output, "__RUM_DIFFTEST_SESSION_RESULT__=", ToString[i, InputForm], "\t", ToString[result, InputForm], "\n"],
  {i, Length[inputPaths]}
];
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

pub struct OracleKernel {
    executable: PathBuf,
    temp_dir: TempDir,
    wrapper_path: PathBuf,
    session_wrapper_path: PathBuf,
    next_case_id: usize,
}

impl OracleKernel {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, OracleError> {
        let executable = executable.into();
        let temp_dir = TempDir::new("rum-difftest")?;
        let wrapper_path = temp_dir.path().join("oracle_wrapper.wls");
        let session_wrapper_path = temp_dir.path().join("oracle_session_wrapper.wls");
        fs::write(&wrapper_path, ORACLE_WRAPPER).map_err(|source| OracleError::Io {
            path: wrapper_path.clone(),
            source,
        })?;
        fs::write(&session_wrapper_path, ORACLE_SESSION_WRAPPER).map_err(|source| {
            OracleError::Io {
                path: session_wrapper_path.clone(),
                source,
            }
        })?;

        Ok(Self {
            executable,
            temp_dir,
            wrapper_path,
            session_wrapper_path,
            next_case_id: 0,
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn version(&self) -> Result<String, OracleError> {
        version::query_kernel_version(&self.executable, self.temp_dir.path())
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

    fn evaluate_session(
        &mut self,
        inputs: &[String],
    ) -> Result<Vec<OracleEvaluation>, OracleError> {
        if inputs.is_empty() {
            return Ok(Vec::new());
        }

        self.next_case_id += 1;
        let session_dir = self
            .temp_dir
            .path()
            .join(format!("session-{:04}", self.next_case_id));
        fs::create_dir(&session_dir).map_err(|source| OracleError::Io {
            path: session_dir.clone(),
            source,
        })?;

        let mut input_paths = Vec::with_capacity(inputs.len());
        for (index, input) in inputs.iter().enumerate() {
            let path = session_dir.join(format!("step-{:04}.wl", index + 1));
            fs::write(&path, input).map_err(|source| OracleError::Io {
                path: path.clone(),
                source,
            })?;
            input_paths.push(path);
        }

        let manifest_path = session_dir.join("manifest.txt");
        let manifest = input_paths
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&manifest_path, manifest).map_err(|source| OracleError::Io {
            path: manifest_path.clone(),
            source,
        })?;

        let output = Command::new(&self.executable)
            .arg("-noprompt")
            .arg("-script")
            .arg(&self.session_wrapper_path)
            .arg(&manifest_path)
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

        let mut results = vec![None; inputs.len()];
        let mut printed_lines = Vec::new();

        for line in stdout.lines() {
            if let Some(payload) = line.strip_prefix(SESSION_RESULT_MARKER) {
                let Some((index_text, value)) = payload.split_once('\t') else {
                    return Err(OracleError::InvalidSessionResultMarker {
                        executable: self.executable.clone(),
                        line: line.to_owned(),
                        stdout: stdout.clone(),
                        stderr: stderr.clone(),
                    });
                };
                let Ok(index) = index_text.parse::<usize>() else {
                    return Err(OracleError::InvalidSessionResultMarker {
                        executable: self.executable.clone(),
                        line: line.to_owned(),
                        stdout: stdout.clone(),
                        stderr: stderr.clone(),
                    });
                };
                if index == 0 || index > inputs.len() {
                    return Err(OracleError::InvalidSessionResultMarker {
                        executable: self.executable.clone(),
                        line: line.to_owned(),
                        stdout: stdout.clone(),
                        stderr: stderr.clone(),
                    });
                }
                results[index - 1] = Some(value.trim().to_owned());
            } else {
                printed_lines.push(line.to_owned());
            }
        }

        let actual = results.iter().filter(|result| result.is_some()).count();
        if actual != inputs.len() {
            return Err(OracleError::SessionResultCountMismatch {
                executable: self.executable.clone(),
                expected: inputs.len(),
                actual,
                stdout,
                stderr,
            });
        }

        let printed_output = printed_lines.join("\n").trim().to_owned();
        let process_stderr = stderr.trim().to_owned();
        let mut evaluations = Vec::with_capacity(results.len());
        for (index, result) in results.into_iter().enumerate() {
            let suffix = format!("{:04}", index + 1);
            let messages_path = session_dir.join(format!("messages-{suffix}.txt"));
            let prints_path = session_dir.join(format!("prints-{suffix}.txt"));
            let mut step_messages =
                read_trimmed_text(&messages_path).map_err(|source| OracleError::Io {
                    path: messages_path.clone(),
                    source,
                })?;
            if !process_stderr.is_empty() {
                step_messages = join_nonempty_lines(&[&step_messages, &process_stderr]);
            }
            let mut step_prints =
                read_trimmed_text(&prints_path).map_err(|source| OracleError::Io {
                    path: prints_path.clone(),
                    source,
                })?;
            if !printed_output.is_empty() {
                step_prints = join_nonempty_lines(&[&step_prints, &printed_output]);
            }

            evaluations.push(OracleEvaluation {
                result: result.expect("result count was already validated"),
                printed_output: step_prints,
                stderr: step_messages,
            });
        }

        Ok(evaluations)
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

fn read_trimmed_text(path: &Path) -> Result<String, std::io::Error> {
    Ok(fs::read_to_string(path)?.trim().to_owned())
}

fn join_nonempty_lines(parts: &[&str]) -> String {
    parts
        .iter()
        .copied()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
