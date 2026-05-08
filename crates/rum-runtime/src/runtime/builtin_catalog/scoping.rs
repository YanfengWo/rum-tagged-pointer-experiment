use super::*;

pub(super) const SCOPING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "With",
        "Lexically substitutes local constants into a body.",
        HoldAttribute::HoldAll,
        BuiltinHandler::With,
    ),
    BuiltinSpec::special_with_hold(
        "Module",
        "Creates lexical local symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Module,
    ),
    BuiltinSpec::ordinary(
        "Unique",
        "Creates a fresh symbol using the current first-pass prefix subset.",
        BuiltinHandler::Unique,
    ),
    BuiltinSpec::special_with_hold(
        "Block",
        "Temporarily rebinds symbols dynamically.",
        HoldAttribute::HoldAll,
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
        "HoldComplete",
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
    BuiltinSpec::ordinary_with_hold(
        "Dialog",
        "Runs the current first-pass dialog subset with Return-sensitive evaluation and host-input suspension.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Dialog,
    ),
    BuiltinSpec::ordinary(
        "InputString",
        "Requests one line of host-provided text and returns it as a string.",
        BuiltinHandler::InputString,
    ),
    BuiltinSpec::ordinary(
        "Input",
        "Requests one line of host-provided text, parses it as Wolfram input, and evaluates the resulting expression in the current first-pass subset.",
        BuiltinHandler::Input,
    ),
    BuiltinSpec::ordinary(
        "Print",
        "Emits a line of plain print output through the runtime print stream.",
        BuiltinHandler::Print,
    ),
    BuiltinSpec::ordinary(
        "Pause",
        "Pauses for at least a nonnegative exact number of seconds in the current first-pass subset.",
        BuiltinHandler::Pause,
    ),
    BuiltinSpec::ordinary_with_hold(
        "BlockRandom",
        "Evaluates an expression with localized random state in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::BlockRandom,
    ),
];
