use rum_runtime::{Attribute, AttributeSet, BuiltinHandler, BuiltinSpec, HoldAttribute};

pub const CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "Check",
        "Evaluates an expression and substitutes a fallback if matching messages are emitted.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Check,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Quiet",
        "Evaluates an expression while suppressing matching runtime messages it emits.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Quiet,
    ),
    BuiltinSpec::ordinary_with_hold(
        "If",
        "Evaluates one of several branches conditionally.",
        HoldAttribute::HoldRest,
        BuiltinHandler::If,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Which",
        "Evaluates conditions in order and returns the first matching branch.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Which,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Function",
        "Creates a pure function.",
        HoldAttribute::HoldAll,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "With",
        "Lexically substitutes local constants into a body.",
        HoldAttribute::HoldAll,
        BuiltinHandler::With,
    ),
    BuiltinSpec::special(
        "Module",
        "Creates lexical local symbols.",
        BuiltinHandler::Module,
    ),
    BuiltinSpec::special(
        "Block",
        "Temporarily rebinds symbols dynamically.",
        BuiltinHandler::Block,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Hold",
        "Prevents evaluation of its arguments.",
        HoldAttribute::HoldAll,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldFirst",
        "Prevents evaluation of the first argument.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldRest",
        "Prevents evaluation of all but the first argument.",
        HoldAttribute::HoldRest,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldAll",
        "Prevents evaluation of all arguments.",
        HoldAttribute::HoldAll,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "HoldAllComplete",
        "Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Unevaluated",
        "Temporarily leaves an argument unevaluated as a direct function argument.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        BuiltinHandler::Unevaluated,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Evaluate",
        "Forces evaluation in held contexts.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Evaluate,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldPattern",
        "Marks a pattern expression so it is matched without prior evaluation.",
        HoldAttribute::HoldAll,
        BuiltinHandler::HoldLiteral,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ReleaseHold",
        "Removes supported hold wrappers and resumes evaluation.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ReleaseHold,
    ),
    BuiltinSpec::ordinary(
        "Sequence",
        "Splices a sequence of arguments into a call.",
        BuiltinHandler::Sequence,
    ),
    BuiltinSpec::ordinary(
        "Thread",
        "Threads an expression over arguments that share a common head.",
        BuiltinHandler::Thread,
    ),
    BuiltinSpec::ordinary(
        "InputString",
        "Requests one line of host-provided text and returns it as a string.",
        BuiltinHandler::InputString,
    ),
];
