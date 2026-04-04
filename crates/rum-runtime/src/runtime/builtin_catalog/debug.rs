use super::*;

pub(super) const DEBUG_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "Trace",
        "Evaluates an expression and returns a first-pass flat trace of current evaluation events.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Trace,
    ),
    BuiltinSpec::ordinary_with_hold(
        "TracePrint",
        "Evaluates an expression, emits its current first-pass flat trace through the print stream, and returns the result.",
        HoldAttribute::HoldAll,
        BuiltinHandler::TracePrint,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Timing",
        "Evaluates an expression and returns {elapsed, value} using the current wall-clock exact-rational subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Timing,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AbsoluteTiming",
        "Evaluates an expression and returns {elapsed, value} using the current wall-clock exact-rational subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::AbsoluteTiming,
    ),
    BuiltinSpec::ordinary(
        "Echo",
        "Prints a value through the runtime print stream and returns it.",
        BuiltinHandler::Echo,
    ),
    BuiltinSpec::ordinary(
        "EchoFunction",
        "Returns a pure function that echoes its argument through the runtime print stream.",
        BuiltinHandler::EchoFunction,
    ),
    BuiltinSpec::ordinary(
        "EchoLabel",
        "Returns a pure function that echoes its argument with a label through the runtime print stream.",
        BuiltinHandler::EchoLabel,
    ),
    BuiltinSpec::ordinary(
        "ByteCount",
        "Returns the current canonical-text byte count for an expression in the first-pass resource subset.",
        BuiltinHandler::ByteCount,
    ),
];
