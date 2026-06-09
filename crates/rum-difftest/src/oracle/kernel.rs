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
#[path = "kernel/session_batches.rs"]
mod session_batches;

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
const ORACLE_SESSION_BATCH_WRAPPER: &str = r#"Begin["RumDifftestOracle`"];
userContext = "Global`";
userContextPath = {"System`"};
initialPackages = {};
initialContexts = {};
resetFixtureState[] := (
  Quiet[Remove["Global`*"], {Remove::rmnsm, Remove::rmvex}];
  Scan[
    Function[{ctx},
      If[
        StringQ[ctx] && !MemberQ[initialContexts, ctx] && !StringMatchQ[ctx, "System`*"],
        With[{pattern = ctx <> "*"},
          Quiet[Remove[pattern], {Remove::rmnsm, Remove::rmvex}]
        ]
      ]
    ],
    Contexts[]
  ];
  Unprotect[$Packages];
  $Packages = initialPackages;
  Protect[$Packages];
  $Context = userContext;
  $ContextPath = userContextPath;
);
End[];

RumDifftestOracle`batchManifestPath = Last[$CommandLine];
RumDifftestOracle`batchManifestPaths = Select[
  Import[RumDifftestOracle`batchManifestPath, "Lines"],
  StringLength[#] > 0 &
];
RumDifftestOracle`initialPackages = $Packages;
RumDifftestOracle`initialContexts = Contexts[];
Do[
  RumDifftestOracle`resetFixtureState[];
  RumDifftestOracle`inputPaths = Select[
    Import[RumDifftestOracle`batchManifestPaths[[RumDifftestOracle`b]], "Lines"],
    StringLength[#] > 0 &
  ];
  RumDifftestOracle`sessionDir = DirectoryName[
    RumDifftestOracle`batchManifestPaths[[RumDifftestOracle`b]]
  ];
  Do[
    RumDifftestOracle`exprText = Import[RumDifftestOracle`inputPaths[[RumDifftestOracle`i]], "Text"];
    RumDifftestOracle`suffix =
      IntegerString[RumDifftestOracle`b, 10, 4] <> "-" <>
      IntegerString[RumDifftestOracle`i, 10, 4];
    RumDifftestOracle`messagesPath =
      FileNameJoin[{RumDifftestOracle`sessionDir, "messages-" <> RumDifftestOracle`suffix <> ".txt"}];
    RumDifftestOracle`printsPath =
      FileNameJoin[{RumDifftestOracle`sessionDir, "prints-" <> RumDifftestOracle`suffix <> ".txt"}];
    RumDifftestOracle`messagesStream =
      OpenWrite[RumDifftestOracle`messagesPath, PageWidth -> Infinity];
    RumDifftestOracle`printsStream =
      OpenWrite[RumDifftestOracle`printsPath, PageWidth -> Infinity];
    RumDifftestOracle`result = Block[
      {$MessageList = {}, $Messages = {RumDifftestOracle`messagesStream}, $Output = {RumDifftestOracle`printsStream}},
      ToExpression[RumDifftestOracle`exprText, InputForm]
    ];
    Close[RumDifftestOracle`messagesStream];
    Close[RumDifftestOracle`printsStream];
    WriteString[
      $Output,
      "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=",
      ToString[RumDifftestOracle`b, InputForm],
      "\t",
      ToString[RumDifftestOracle`i, InputForm],
      "\t",
      ToString[RumDifftestOracle`result, InputForm],
      "\n"
    ],
    {RumDifftestOracle`i, Length[RumDifftestOracle`inputPaths]}
  ],
  {RumDifftestOracle`b, Length[RumDifftestOracle`batchManifestPaths]}
];
Exit[];
"#;

pub struct OracleKernel {
    executable: PathBuf,
    temp_dir: TempDir,
    wrapper_path: PathBuf,
    session_wrapper_path: PathBuf,
    session_batch_wrapper_path: PathBuf,
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
        let session_batch_wrapper_path = temp_dir.path().join("oracle_session_batch_wrapper.wls");
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
        fs::write(&session_batch_wrapper_path, ORACLE_SESSION_BATCH_WRAPPER).map_err(|source| {
            OracleError::Io {
                path: session_batch_wrapper_path.clone(),
                source,
            }
        })?;

        Ok(Self {
            executable,
            temp_dir,
            wrapper_path,
            session_wrapper_path,
            session_batch_wrapper_path,
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

    pub fn session_batch_transport_name(&self) -> &'static str {
        "kernel-single-process-batches"
    }

    pub fn session_batch_process_count(&self, batch_count: usize) -> usize {
        usize::from(batch_count > 0)
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

    fn evaluate_session_batches(
        &mut self,
        batches: &[Vec<String>],
    ) -> Result<Vec<Vec<OracleEvaluation>>, OracleError> {
        session_batches::evaluate_session_batches(self, batches)
    }
}
