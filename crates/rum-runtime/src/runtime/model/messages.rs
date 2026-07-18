use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    OwnValue,
    DelayedOwnValue,
    UpValue,
    DownValue,
    SubValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleApplicationSource {
    UpValue,
    DownValue,
    SubValue,
    Replace,
    ReplaceList,
    ReplaceAt,
    ReplaceAll,
    ReplaceAllUnheld,
    ReplaceRepeated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceEvent {
    EvaluationPass {
        before: String,
        after: String,
    },
    DefinitionStored {
        symbol: String,
        kind: DefinitionKind,
        value: String,
    },
    RuleApplied {
        source: RuleApplicationSource,
        input: String,
        rule: String,
        result: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMessage {
    pub symbol: String,
    pub tag: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeInputKind {
    InputString,
    Input,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeInputRequest {
    pub kind: RuntimeInputKind,
    pub prompt: String,
    pub password: bool,
}

impl RuntimeInputRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            kind: RuntimeInputKind::InputString,
            prompt: prompt.into(),
            password: false,
        }
    }

    pub fn input(mut self) -> Self {
        self.kind = RuntimeInputKind::Input;
        self
    }

    pub fn input_string(mut self) -> Self {
        self.kind = RuntimeInputKind::InputString;
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct MessageSelector {
    pub(in crate::runtime) symbol: String,
    pub(in crate::runtime) tag: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runtime) struct MessageNameKey {
    pub(in crate::runtime) symbol: Symbol,
    pub(in crate::runtime) tag: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::runtime) struct MessageDefinition {
    pub(in crate::runtime) immediate: Option<Expr>,
    pub(in crate::runtime) delayed: Option<Expr>,
}

impl fmt::Display for RuntimeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}: {}", self.symbol, self.tag, self.text)
    }
}

impl fmt::Display for RuntimeInputRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.prompt.is_empty() {
            f.write_str("input required")
        } else {
            write!(f, "input required: {}", self.prompt)
        }
    }
}

impl MessageSelector {
    pub(in crate::runtime) fn matches(&self, message: &RuntimeMessage) -> bool {
        self.symbol == message.symbol
            && self
                .tag
                .as_ref()
                .map_or(true, |tag| message.tag.as_str() == tag.as_str())
    }

    pub(in crate::runtime) fn covers(&self, other: &MessageSelector) -> bool {
        self.symbol == other.symbol
            && self.tag.as_ref().map_or(true, |tag| {
                other.tag.as_ref().is_some_and(|other| other == tag)
            })
    }
}
