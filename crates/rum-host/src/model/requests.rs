use rum_runtime::SessionCompletenessStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostParseRequest {
    pub source: String,
}

impl HostParseRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

impl From<&str> for HostParseRequest {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

impl From<String> for HostParseRequest {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostParseResponse {
    pub source: String,
    pub syntax: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostEvalRequest {
    pub source: String,
}

impl HostEvalRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

impl From<&str> for HostEvalRequest {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

impl From<String> for HostEvalRequest {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCompletionRequest {
    pub source: String,
    pub cursor_offset: usize,
}

impl HostCompletionRequest {
    pub fn new(source: impl Into<String>, cursor_offset: usize) -> Self {
        Self {
            source: source.into(),
            cursor_offset,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCompletionResponse {
    pub prefix: String,
    pub matches: Vec<String>,
    pub replacement_start: usize,
    pub replacement_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInspectRequest {
    pub source: String,
    pub cursor_offset: usize,
}

impl HostInspectRequest {
    pub fn new(source: impl Into<String>, cursor_offset: usize) -> Self {
        Self {
            source: source.into(),
            cursor_offset,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostIsCompleteRequest {
    pub source: String,
}

impl HostIsCompleteRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

impl From<&str> for HostIsCompleteRequest {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

impl From<String> for HostIsCompleteRequest {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostIsCompleteResponse {
    pub status: HostCompletenessStatus,
    pub indent: Option<String>,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostCompletenessStatus {
    Complete,
    Incomplete,
    Invalid,
}

pub(crate) fn completeness_status(status: SessionCompletenessStatus) -> HostCompletenessStatus {
    match status {
        SessionCompletenessStatus::Complete => HostCompletenessStatus::Complete,
        SessionCompletenessStatus::Incomplete => HostCompletenessStatus::Incomplete,
        SessionCompletenessStatus::Invalid => HostCompletenessStatus::Invalid,
    }
}
