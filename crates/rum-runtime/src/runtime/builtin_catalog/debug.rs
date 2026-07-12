use super::*;

pub(super) const DEBUG_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "Trace",
        "Evaluates an expression and returns a first-pass structured trace of current evaluation events.",
        HoldAttribute::HoldAll,
        builtin_trace,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "TracePrint",
        "Evaluates an expression, emits its current first-pass structured trace through the print stream, and returns the result.",
        HoldAttribute::HoldAll,
        builtin_trace_print,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Timing",
        "Evaluates an expression and returns {elapsed, value} using the current wall-clock exact-rational subset.",
        HoldAttribute::HoldAll,
        builtin_timing,
    ).protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary(
        "AbsoluteTime",
        "Returns the current exact rational Unix timestamp (seconds since 1970-01-01T00:00:00Z); requires Capability::Clock.",
        Session::eval_absolute_time,
    ).protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary_with_hold(
        "AbsoluteTiming",
        "Evaluates an expression and returns {elapsed, value} using the current wall-clock exact-rational subset.",
        HoldAttribute::HoldAll,
        builtin_absolute_timing,
    ).protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary_with_attributes(
        "RepeatedTiming",
        "Evaluates an expression multiple times and returns {elapsed, value} in the current fixed-sample wall-clock subset.",
        AttributeSet::empty()
            .with(Attribute::HoldFirst)
            .with(Attribute::SequenceHold),
        builtin_repeated_timing,
    ).protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary(
        "Short",
        "Returns the current canonical-text shortened display subset for an expression.",
        Session::eval_short,
    ).protected(),
    BuiltinSpec::ordinary(
        "Echo",
        "Prints a value through the runtime print stream and returns it.",
        Session::eval_echo,
    ).protected(),
    BuiltinSpec::ordinary(
        "EchoFunction",
        "Returns a pure function that echoes its argument through the runtime print stream.",
        Session::eval_echo_function,
    ).protected(),
    BuiltinSpec::ordinary(
        "EchoLabel",
        "Returns a pure function that echoes its argument with a label through the runtime print stream.",
        Session::eval_echo_label,
    ).protected(),
    BuiltinSpec::ordinary(
        "ByteCount",
        "Returns the current canonical-text byte count for an expression in the first-pass resource subset.",
        Session::eval_byte_count,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "On",
        "Enables session-level assertion checking or emission of matching messages in the current selector subset.",
        HoldAttribute::HoldAll,
        builtin_on,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Off",
        "Disables session-level assertion checking or emission of matching messages in the current selector subset.",
        HoldAttribute::HoldAll,
        builtin_off,
    ).protected(),
    BuiltinSpec::ordinary(
        "Sow",
        "Collects a value for enclosing Reap in the current first-pass tag-and-group subset.",
        Session::eval_sow,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Reap",
        "Evaluates an expression and collects values emitted by Sow in the current first-pass grouping subset.",
        HoldAttribute::HoldFirst,
        builtin_reap,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "TimeConstrained",
        "Evaluates an expression under the current cooperative exact-numeric timeout subset.",
        HoldAttribute::HoldAll,
        builtin_time_constrained,
    ).protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary_with_hold(
        "MemoryConstrained",
        "Evaluates an expression under the current cooperative memory-proxy limit subset.",
        HoldAttribute::HoldAll,
        builtin_memory_constrained,
    ).protected(),
    BuiltinSpec::ordinary(
        "MemoryInUse",
        "Returns the current session memory-proxy byte count subset.",
        Session::eval_memory_in_use,
    ).protected()
    .with_capability(Capability::Diagnostic),
    BuiltinSpec::ordinary(
        "MaxMemoryUsed",
        "Returns the peak session memory-proxy byte count high-water mark subset.",
        Session::eval_max_memory_used,
    ).protected()
    .with_capability(Capability::Diagnostic),
    BuiltinSpec::ordinary_with_hold(
        "Monitor",
        "Evaluates an expression while exposing the current first-pass monitor display and callback subset.",
        HoldAttribute::HoldAll,
        builtin_monitor,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "EvaluationData",
        "Evaluates an expression and returns a first-pass association of result and diagnostics.",
        HoldAttribute::HoldFirst,
        builtin_evaluation_data,
    ).protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary(
        "RumProfileReset",
        "Internal diagnostic hook: resets rum runtime profile counters when profiling is enabled.",
        builtin_rum_profile_reset,
    ).protected()
    .with_capability(Capability::Diagnostic),
    BuiltinSpec::ordinary(
        "RumProfileSnapshot",
        "Internal diagnostic hook: prints rum runtime profile counters when profiling is enabled.",
        builtin_rum_profile_snapshot,
    ).protected()
    .with_capability(Capability::Diagnostic),
    BuiltinSpec::ordinary_with_hold(
        "TestCreate",
        "Builds a first-pass symbolic test specification.",
        HoldAttribute::HoldAll,
        Session::eval_test_create,
    ).protected(),
    BuiltinSpec::ordinary(
        "TestReport",
        "Runs one or more first-pass symbolic tests and returns an association summary.",
        Session::eval_test_report,
    ).protected()
    .with_capability(Capability::Clock),
    BuiltinSpec::ordinary_with_hold(
        "Stack",
        "Returns the current evaluation stack in the first-pass held-expression subset.",
        HoldAttribute::HoldAll,
        builtin_stack,
    ).protected(),
];
