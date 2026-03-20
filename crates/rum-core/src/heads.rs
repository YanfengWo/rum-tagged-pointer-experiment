pub mod structure {
    pub const NULL: &str = "Null";
    pub const LIST: &str = "List";
    pub const FILE: &str = "File";
    pub const TRUE: &str = "True";
    pub const FALSE: &str = "False";
    pub const ALL: &str = "All";
    pub const NONE: &str = "None";
    pub const INFINITY: &str = "Infinity";
    pub const FAILED: &str = "$Failed";
    pub const CONTEXT_VAR: &str = "$Context";
    pub const CONTEXT_PATH_VAR: &str = "$ContextPath";
    pub const PACKAGES_VAR: &str = "$Packages";
    pub const PATH_VAR: &str = "$Path";
    pub const MESSAGE_LIST_VAR: &str = "$MessageList";
    pub const INTEGER: &str = "Integer";
    pub const STRING: &str = "String";
    pub const SYMBOL: &str = "Symbol";
}

pub mod arithmetic {
    pub const PLUS: &str = "Plus";
    pub const TIMES: &str = "Times";
    pub const POWER: &str = "Power";
}

pub mod patterns {
    pub const PATTERN: &str = "Pattern";
    pub const BLANK: &str = "Blank";
    pub const BLANK_SEQUENCE: &str = "BlankSequence";
    pub const BLANK_NULL_SEQUENCE: &str = "BlankNullSequence";
    pub const CONDITION: &str = "Condition";
}

pub mod rules {
    pub const SET: &str = "Set";
    pub const SET_DELAYED: &str = "SetDelayed";
    pub const UNSET: &str = "Unset";
    pub const RULE: &str = "Rule";
    pub const RULE_DELAYED: &str = "RuleDelayed";
    pub const REPLACE_ALL: &str = "ReplaceAll";
    pub const REPLACE_REPEATED: &str = "ReplaceRepeated";
    pub const TAG_SET: &str = "TagSet";
    pub const TAG_SET_DELAYED: &str = "TagSetDelayed";
    pub const TAG_UNSET: &str = "TagUnset";
}

pub mod functions {
    pub const SLOT: &str = "Slot";
    pub const SLOT_SEQUENCE: &str = "SlotSequence";
    pub const FUNCTION: &str = "Function";
    pub const SEQUENCE: &str = "Sequence";
    pub const EVALUATE: &str = "Evaluate";
    pub const UNEVALUATED: &str = "Unevaluated";
    pub const HOLD_PATTERN: &str = "HoldPattern";
    pub const RELEASE_HOLD: &str = "ReleaseHold";
}

pub mod scoping {
    pub const IF: &str = "If";
    pub const WHICH: &str = "Which";
    pub const WITH: &str = "With";
    pub const MODULE: &str = "Module";
    pub const BLOCK: &str = "Block";
}

pub mod contexts {
    pub const CONTEXT: &str = "Context";
    pub const BEGIN: &str = "Begin";
    pub const END: &str = "End";
    pub const BEGIN_PACKAGE: &str = "BeginPackage";
    pub const END_PACKAGE: &str = "EndPackage";
    pub const NEEDS: &str = "Needs";
    pub const GET: &str = "Get";
    pub const FIND_FILE: &str = "FindFile";
}

pub mod messages {
    pub const MESSAGE_NAME: &str = "MessageName";
    pub const MESSAGE: &str = "Message";
    pub const MESSAGES: &str = "Messages";
    pub const MESSAGE_DEFINITIONS: &str = "MessageDefinitions";
    pub const MESSAGE_TAGS: &str = "MessageTags";
    pub const GENERAL: &str = "General";
}

pub mod inspection {
    pub const OWN_VALUES: &str = "OwnValues";
    pub const DOWN_VALUES: &str = "DownValues";
    pub const UP_VALUES: &str = "UpValues";
    pub const SUB_VALUES: &str = "SubValues";
    pub const NORMALIZE_RULES: &str = "NormalizeRules";
    pub const ATTRIBUTES: &str = "Attributes";
    pub const SET_ATTRIBUTES: &str = "SetAttributes";
    pub const CLEAR_ATTRIBUTES: &str = "ClearAttributes";
}

pub const CANONICAL_CORE_HEADS: &[&str] = &[
    structure::NULL,
    structure::LIST,
    structure::FILE,
    structure::TRUE,
    structure::FALSE,
    structure::ALL,
    structure::NONE,
    structure::INFINITY,
    structure::FAILED,
    structure::CONTEXT_VAR,
    structure::CONTEXT_PATH_VAR,
    structure::PACKAGES_VAR,
    structure::PATH_VAR,
    structure::MESSAGE_LIST_VAR,
    structure::INTEGER,
    structure::STRING,
    structure::SYMBOL,
    arithmetic::PLUS,
    arithmetic::TIMES,
    arithmetic::POWER,
    patterns::PATTERN,
    patterns::BLANK,
    patterns::BLANK_SEQUENCE,
    patterns::BLANK_NULL_SEQUENCE,
    patterns::CONDITION,
    rules::SET,
    rules::SET_DELAYED,
    rules::UNSET,
    rules::RULE,
    rules::RULE_DELAYED,
    rules::REPLACE_ALL,
    rules::REPLACE_REPEATED,
    rules::TAG_SET,
    rules::TAG_SET_DELAYED,
    rules::TAG_UNSET,
    functions::SLOT,
    functions::SLOT_SEQUENCE,
    functions::FUNCTION,
    functions::SEQUENCE,
    functions::EVALUATE,
    functions::UNEVALUATED,
    functions::HOLD_PATTERN,
    functions::RELEASE_HOLD,
    scoping::IF,
    scoping::WHICH,
    scoping::WITH,
    scoping::MODULE,
    scoping::BLOCK,
    contexts::CONTEXT,
    contexts::BEGIN,
    contexts::END,
    contexts::BEGIN_PACKAGE,
    contexts::END_PACKAGE,
    contexts::NEEDS,
    contexts::GET,
    contexts::FIND_FILE,
    messages::MESSAGE_NAME,
    messages::MESSAGE,
    messages::MESSAGES,
    messages::MESSAGE_DEFINITIONS,
    messages::MESSAGE_TAGS,
    messages::GENERAL,
    inspection::OWN_VALUES,
    inspection::DOWN_VALUES,
    inspection::UP_VALUES,
    inspection::SUB_VALUES,
    inspection::NORMALIZE_RULES,
    inspection::ATTRIBUTES,
    inspection::SET_ATTRIBUTES,
    inspection::CLEAR_ATTRIBUTES,
];

pub fn is_canonical_core_head_name(name: &str) -> bool {
    CANONICAL_CORE_HEADS.contains(&name)
}
