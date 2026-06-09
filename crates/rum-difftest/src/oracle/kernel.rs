use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::temp_dir::TempDir;
use super::version;
use super::{
    DEFAULT_ORACLE_OUTPUT_LIMIT_BYTES, DEFAULT_ORACLE_TIMEOUT, Oracle, OracleError,
    OracleEvaluation,
};

#[path = "kernel/evaluate.rs"]
mod evaluate;
#[path = "kernel/session.rs"]
mod session;

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

pub struct OracleKernel {
    executable: PathBuf,
    temp_dir: TempDir,
    wrapper_path: PathBuf,
    session_wrapper_path: PathBuf,
    next_case_id: usize,
    timeout: Duration,
    output_limit_bytes: usize,
}

impl OracleKernel {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, OracleError> {
        Self::with_timeout(executable, DEFAULT_ORACLE_TIMEOUT)
    }

    pub fn with_timeout(
        executable: impl Into<PathBuf>,
        timeout: Duration,
    ) -> Result<Self, OracleError> {
        Self::with_limits(executable, timeout, DEFAULT_ORACLE_OUTPUT_LIMIT_BYTES)
    }

    pub fn with_limits(
        executable: impl Into<PathBuf>,
        timeout: Duration,
        output_limit_bytes: usize,
    ) -> Result<Self, OracleError> {
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
            timeout,
            output_limit_bytes,
        })
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn output_limit_bytes(&self) -> usize {
        self.output_limit_bytes
    }

    pub fn version(&self) -> Result<String, OracleError> {
        version::query_kernel_version(
            &self.executable,
            self.temp_dir.path(),
            self.timeout,
            self.output_limit_bytes,
        )
    }
}

impl Oracle for OracleKernel {
    fn evaluate(&mut self, input: &str) -> Result<OracleEvaluation, OracleError> {
        evaluate::evaluate(self, input)
    }

    fn evaluate_session(
        &mut self,
        inputs: &[String],
    ) -> Result<Vec<OracleEvaluation>, OracleError> {
        session::evaluate_session(self, inputs)
    }
}
