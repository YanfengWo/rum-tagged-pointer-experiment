use rum_runtime::{BuiltinSpec, Session};

pub const CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec {
        name: "Identity",
        summary: "Returns its argument unchanged.",
    },
    BuiltinSpec {
        name: "Set",
        summary: "Assigns an immediate value to a symbol or pattern.",
    },
    BuiltinSpec {
        name: "SetDelayed",
        summary: "Assigns a delayed definition evaluated on use.",
    },
    BuiltinSpec {
        name: "Rule",
        summary: "Represents an immediate rewrite rule.",
    },
    BuiltinSpec {
        name: "RuleDelayed",
        summary: "Represents a delayed rewrite rule.",
    },
    BuiltinSpec {
        name: "MatchQ",
        summary: "Tests whether an expression matches a pattern.",
    },
    BuiltinSpec {
        name: "Cases",
        summary: "Collects subexpressions matching a pattern or rule.",
    },
    BuiltinSpec {
        name: "ReplaceAll",
        summary: "Applies rules everywhere in an expression.",
    },
    BuiltinSpec {
        name: "If",
        summary: "Evaluates one of several branches conditionally.",
    },
    BuiltinSpec {
        name: "Function",
        summary: "Creates a pure function.",
    },
    BuiltinSpec {
        name: "Module",
        summary: "Creates lexical local symbols.",
    },
    BuiltinSpec {
        name: "Block",
        summary: "Temporarily rebinds symbols dynamically.",
    },
    BuiltinSpec {
        name: "With",
        summary: "Substitutes local constants lexically.",
    },
    BuiltinSpec {
        name: "Hold",
        summary: "Prevents evaluation of its arguments.",
    },
    BuiltinSpec {
        name: "HoldFirst",
        summary: "Prevents evaluation of the first argument.",
    },
    BuiltinSpec {
        name: "HoldRest",
        summary: "Prevents evaluation of all but the first argument.",
    },
    BuiltinSpec {
        name: "HoldAll",
        summary: "Prevents evaluation of all arguments.",
    },
    BuiltinSpec {
        name: "Evaluate",
        summary: "Forces evaluation in held contexts.",
    },
    BuiltinSpec {
        name: "Sequence",
        summary: "Splices a sequence of arguments into a call.",
    },
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
