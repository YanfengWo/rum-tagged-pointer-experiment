use super::*;

pub(super) const SCOPING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "With",
        "Lexically substitutes local constants into a body.",
        HoldAttribute::HoldAll,
        builtin_with,
    ),
    BuiltinSpec::special_with_hold(
        "Module",
        "Creates lexical local symbols.",
        HoldAttribute::HoldAll,
        builtin_module,
    ),
    BuiltinSpec::ordinary(
        "Unique",
        "Creates fresh symbols using prefix, list, and attribute forms.",
        builtin_unique,
    ),
    BuiltinSpec::special_with_hold(
        "Block",
        "Temporarily rebinds symbols dynamically.",
        HoldAttribute::HoldAll,
        builtin_block,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Hold",
        "Prevents evaluation of its arguments.",
        HoldAttribute::HoldAll,
        builtin_hold_literal,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldFirst",
        "Prevents evaluation of the first argument.",
        HoldAttribute::HoldFirst,
        builtin_hold_literal,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldRest",
        "Prevents evaluation of all but the first argument.",
        HoldAttribute::HoldRest,
        builtin_hold_literal,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldAll",
        "Prevents evaluation of all arguments.",
        HoldAttribute::HoldAll,
        builtin_hold_literal,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "HoldAllComplete",
        "Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        builtin_hold_literal,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "HoldComplete",
        "Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        builtin_hold_literal,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Unevaluated",
        "Temporarily leaves an argument unevaluated as a direct function argument.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
        builtin_unevaluated,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Evaluate",
        "Forces evaluation in held contexts.",
        HoldAttribute::HoldAll,
        builtin_evaluate,
    ),
    BuiltinSpec::ordinary_with_hold(
        "HoldPattern",
        "Marks a pattern expression so it is matched without prior evaluation.",
        HoldAttribute::HoldAll,
        builtin_hold_literal,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ReleaseHold",
        "Removes supported hold wrappers and resumes evaluation.",
        HoldAttribute::HoldAll,
        builtin_release_hold,
    ),
    BuiltinSpec::ordinary(
        "Sequence",
        "Splices a sequence of arguments into a call.",
        builtin_sequence,
    ),
    BuiltinSpec::ordinary(
        "Thread",
        "Threads an expression over arguments that share a common head.",
        Session::eval_thread,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Dialog",
        "Runs the current first-pass dialog subset with Return-sensitive evaluation and host-input suspension.",
        HoldAttribute::HoldAll,
        builtin_dialog,
    ),
    BuiltinSpec::ordinary(
        "InputString",
        "Requests one line of host-provided text and returns it as a string.",
        builtin_input_string,
    ),
    BuiltinSpec::ordinary(
        "Input",
        "Requests one line of host-provided text, parses it as rum source, and evaluates the resulting expression in the current first-pass subset.",
        builtin_input,
    ),
    BuiltinSpec::ordinary(
        "Print",
        "Emits a line of plain print output through the runtime print stream.",
        builtin_print,
    ),
    BuiltinSpec::ordinary(
        "Pause",
        "Pauses for at least a nonnegative exact number of seconds in the current first-pass subset.",
        builtin_pause,
    ),
    BuiltinSpec::ordinary_with_hold(
        "BlockRandom",
        "Evaluates an expression with localized random state in the current first-pass subset.",
        HoldAttribute::HoldAll,
        builtin_block_random,
    ),
];
