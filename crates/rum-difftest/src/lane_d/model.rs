use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LaneDSuite {
    #[default]
    All,
    Timing,
    Memory,
    CoreLanguage,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LaneDProfile {
    #[default]
    Smoke,
    Full,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LaneDThresholdProfile {
    #[default]
    LocalDebug,
    CiDebug,
    CiRelease,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaneDOptions {
    pub suite: LaneDSuite,
    pub profile: LaneDProfile,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDSummary {
    pub total_seen: usize,
    pub executed: usize,
    pub skipped_by_profile: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub total_elapsed_micros: u128,
    pub cases: Vec<LaneDCaseReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneDCaseReport {
    pub suite: String,
    pub source: PathBuf,
    pub index: usize,
    pub category: Option<String>,
    pub threshold: Option<String>,
    pub input: String,
    pub status: LaneDCaseStatus,
    pub elapsed_micros: u128,
    pub value_nodes: Option<usize>,
    pub message_count: usize,
    pub print_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaneDCaseStatus {
    Success,
    Failure { message: String },
}
