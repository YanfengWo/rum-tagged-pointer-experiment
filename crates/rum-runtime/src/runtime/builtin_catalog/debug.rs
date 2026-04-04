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
    BuiltinSpec::ordinary_with_attributes(
        "RepeatedTiming",
        "Evaluates an expression multiple times and returns {elapsed, value} in the current fixed-sample wall-clock subset.",
        AttributeSet::empty()
            .with(Attribute::HoldFirst)
            .with(Attribute::SequenceHold),
        BuiltinHandler::RepeatedTiming,
    ),
    BuiltinSpec::ordinary(
        "Short",
        "Returns the current canonical-text shortened display subset for an expression.",
        BuiltinHandler::Short,
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
    BuiltinSpec::ordinary_with_hold(
        "On",
        "Enables session-level emission of matching messages in the current selector subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::On,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Off",
        "Disables session-level emission of matching messages in the current selector subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Off,
    ),
    BuiltinSpec::ordinary(
        "Sow",
        "Collects a value for enclosing Reap in the current first-pass tag-and-group subset.",
        BuiltinHandler::Sow,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Reap",
        "Evaluates an expression and collects values emitted by Sow in the current first-pass grouping subset.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Reap,
    ),
    BuiltinSpec::ordinary_with_hold(
        "TimeConstrained",
        "Evaluates an expression under the current cooperative exact-numeric timeout subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::TimeConstrained,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MemoryConstrained",
        "Evaluates an expression under the current cooperative memory-proxy limit subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MemoryConstrained,
    ),
    BuiltinSpec::ordinary(
        "MemoryInUse",
        "Returns the current session memory-proxy byte count subset.",
        BuiltinHandler::MemoryInUse,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Monitor",
        "Evaluates an expression while exposing the current first-pass monitor display and callback subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Monitor,
    ),
    BuiltinSpec::ordinary_with_hold(
        "EvaluationData",
        "Evaluates an expression and returns a first-pass association of result and diagnostics.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::EvaluationData,
    ),
    BuiltinSpec::ordinary_with_hold(
        "TestCreate",
        "Builds a first-pass symbolic test specification.",
        HoldAttribute::HoldAll,
        BuiltinHandler::TestCreate,
    ),
    BuiltinSpec::ordinary(
        "TestReport",
        "Runs one or more first-pass symbolic tests and returns an association summary.",
        BuiltinHandler::TestReport,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Stack",
        "Returns the current evaluation stack in the first-pass held-expression subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Stack,
    ),
];
