use rum_runtime::SessionCompletenessStatus;

/// Request to parse source without evaluating it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostParseRequest {
    /// Source text to parse.
    pub source: String,
}

impl HostParseRequest {
    /// Creates a parse request from source text.
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

/// Response from a parse request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostParseResponse {
    /// Source text that was parsed.
    pub source: String,
    /// Debug rendering of the parsed syntax tree.
    pub syntax: String,
}

/// Request to evaluate source text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostEvalRequest {
    /// Source text to evaluate.
    pub source: String,
}

impl HostEvalRequest {
    /// Creates an evaluation request from source text.
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

/// Request completion candidates at a cursor position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCompletionRequest {
    /// Source text containing the cursor.
    pub source: String,
    /// Byte offset of the cursor within `source`.
    pub cursor_offset: usize,
}

impl HostCompletionRequest {
    /// Creates a completion request for a source string and cursor byte offset.
    pub fn new(source: impl Into<String>, cursor_offset: usize) -> Self {
        Self {
            source: source.into(),
            cursor_offset,
        }
    }
}

/// Completion candidates and replacement range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCompletionResponse {
    /// Prefix used to query completions.
    pub prefix: String,
    /// Matching completion candidates.
    pub matches: Vec<String>,
    /// Start byte offset of the text replacement.
    pub replacement_start: usize,
    /// End byte offset of the text replacement.
    pub replacement_end: usize,
}

/// Request symbol inspection at a cursor position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInspectRequest {
    /// Source text containing the cursor.
    pub source: String,
    /// Byte offset of the cursor within `source`.
    pub cursor_offset: usize,
}

impl HostInspectRequest {
    /// Creates an inspection request for a source string and cursor byte offset.
    pub fn new(source: impl Into<String>, cursor_offset: usize) -> Self {
        Self {
            source: source.into(),
            cursor_offset,
        }
    }
}

/// Request to classify source completeness for interactive submission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostIsCompleteRequest {
    /// Source text to classify.
    pub source: String,
}

impl HostIsCompleteRequest {
    /// Creates a source-completeness request.
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

/// Response from a source-completeness request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostIsCompleteResponse {
    /// Completeness status for the source text.
    pub status: HostCompletenessStatus,
    /// Optional indentation hint for an incomplete source form.
    pub indent: Option<String>,
    /// Optional diagnostic message when the source is invalid.
    pub diagnostic: Option<String>,
}

/// Source completeness classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostCompletenessStatus {
    /// The source is complete and can be evaluated.
    Complete,
    /// The source is syntactically incomplete and may accept more input.
    Incomplete,
    /// The source is invalid rather than merely incomplete.
    Invalid,
}

pub(crate) fn completeness_status(status: SessionCompletenessStatus) -> HostCompletenessStatus {
    match status {
        SessionCompletenessStatus::Complete => HostCompletenessStatus::Complete,
        SessionCompletenessStatus::Incomplete => HostCompletenessStatus::Incomplete,
        SessionCompletenessStatus::Invalid => HostCompletenessStatus::Invalid,
    }
}
