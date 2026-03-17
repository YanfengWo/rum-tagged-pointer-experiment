use rum_runtime::{BuiltinSpec, HoldAttribute, Session};

pub const CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new("Identity", "Returns its argument unchanged."),
    BuiltinSpec::new("Set", "Assigns an immediate value to a symbol or pattern."),
    BuiltinSpec::new(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
    ),
    BuiltinSpec::new("Rule", "Represents an immediate rewrite rule."),
    BuiltinSpec::new("RuleDelayed", "Represents a delayed rewrite rule."),
    BuiltinSpec::new("MatchQ", "Tests whether an expression matches a pattern."),
    BuiltinSpec::new(
        "Cases",
        "Collects subexpressions matching a pattern or rule.",
    ),
    BuiltinSpec::new("ReplaceAll", "Applies rules everywhere in an expression."),
    BuiltinSpec::new(
        "ReplaceRepeated",
        "Repeatedly applies rules until an expression reaches a fixed point.",
    ),
    BuiltinSpec::new(
        "$Context",
        "Evaluates to the current default context name as a string.",
    ),
    BuiltinSpec::new(
        "$ContextPath",
        "Evaluates to the active context search path as a list of strings.",
    ),
    BuiltinSpec::with_hold(
        "If",
        "Evaluates one of several branches conditionally.",
        HoldAttribute::HoldRest,
    ),
    BuiltinSpec::with_hold(
        "Which",
        "Evaluates conditions in order and returns the first matching branch.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "Function",
        "Creates a pure function.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "With",
        "Lexically substitutes local constants into a body.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "Module",
        "Creates lexical local symbols.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "Block",
        "Temporarily rebinds symbols dynamically.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "Hold",
        "Prevents evaluation of its arguments.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "HoldFirst",
        "Prevents evaluation of the first argument.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "HoldRest",
        "Prevents evaluation of all but the first argument.",
        HoldAttribute::HoldRest,
    ),
    BuiltinSpec::with_hold(
        "HoldAll",
        "Prevents evaluation of all arguments.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "Evaluate",
        "Forces evaluation in held contexts.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::new("Sequence", "Splices a sequence of arguments into a call."),
];

pub fn install_core(session: &mut Session) {
    for spec in CORE_BUILTINS {
        session.register_builtin(*spec);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_identity_builtin() {
        let mut session = Session::new();
        install_core(&mut session);

        let info = session.inspect("Identity").unwrap();
        assert!(info.is_builtin);
    }
}
