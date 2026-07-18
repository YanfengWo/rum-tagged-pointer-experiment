use super::*;

pub(in super::super) const NAMESPACE_VARIABLE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new(
        "$Context",
        "Evaluates to the current default context name as a string.",
    ).protected(),
    BuiltinSpec::new(
        "$ContextPath",
        "Evaluates to the active context search path as a list of strings.",
    ),
    BuiltinSpec::new(
        "$Aborted",
        "Represents a computation aborted by Abort or an external interrupt.",
    ).protected(),
    BuiltinSpec::new(
        "$Canceled",
        "Represents a canceled operation for confirmation-flow compatibility.",
    ).protected(),
    BuiltinSpec::new("$Failed", "Represents a failed operation.").protected(),
    BuiltinSpec::new(
        "$MessageList",
        "Lists the message names emitted by the most recent top-level evaluation.",
    ).protected(),
    BuiltinSpec::new(
        "$Packages",
        "Lists package contexts loaded into the current session.",
    ),
    BuiltinSpec::new(
        "$InputFileName",
        "Evaluates to the path of the file currently being loaded, or the empty string at top level.",
    ).protected(),
    BuiltinSpec::new("$Path", "Lists search roots used by the package loader."),
    BuiltinSpec::new(
        "Nothing",
        "Represents a list element that is removed from evaluated Lists.",
    ).protected(),
];
