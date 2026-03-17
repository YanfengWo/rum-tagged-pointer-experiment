use rum_runtime::{Attribute, AttributeSet, BuiltinSpec, HoldAttribute, Session};

pub const CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new("Identity", "Returns its argument unchanged."),
    BuiltinSpec::new("Set", "Assigns an immediate value to a symbol or pattern."),
    BuiltinSpec::new(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
    ),
    BuiltinSpec::new(
        "TagSet",
        "Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.",
    ),
    BuiltinSpec::new(
        "TagSetDelayed",
        "Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.",
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
    BuiltinSpec::with_hold(
        "NormalizeRules",
        "Normalizes a rule or rule list into an explicit canonical list form for debugging.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::new(
        "$Context",
        "Evaluates to the current default context name as a string.",
    ),
    BuiltinSpec::new(
        "$ContextPath",
        "Evaluates to the active context search path as a list of strings.",
    ),
    BuiltinSpec::new("$Failed", "Represents a failed operation."),
    BuiltinSpec::new(
        "$MessageList",
        "Lists the message names emitted by the most recent top-level evaluation.",
    ),
    BuiltinSpec::new(
        "$Packages",
        "Lists package contexts loaded into the current session.",
    ),
    BuiltinSpec::new("$Path", "Lists search roots used by the package loader."),
    BuiltinSpec::with_hold(
        "Context",
        "Returns the context of the current session or of a symbol.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::new("File", "Represents an explicit file specification."),
    BuiltinSpec::new(
        "FindFile",
        "Finds the full path of a file or package using the current search path.",
    ),
    BuiltinSpec::new(
        "Get",
        "Reads a file or package and evaluates the expressions it contains.",
    ),
    BuiltinSpec::new(
        "Needs",
        "Loads a package only if its context has not already been loaded.",
    ),
    BuiltinSpec::new(
        "Begin",
        "Switches into a context and prepends it to the context path.",
    ),
    BuiltinSpec::new(
        "End",
        "Restores the previous context after a matching Begin.",
    ),
    BuiltinSpec::new(
        "BeginPackage",
        "Starts a package context with a package-oriented context path.",
    ),
    BuiltinSpec::new(
        "EndPackage",
        "Ends a package context and leaves it on the restored context path.",
    ),
    BuiltinSpec::with_hold(
        "Message",
        "Emits a message described by a MessageName expression.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "MessageName",
        "Represents a symbolic message name such as f::tag.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "Messages",
        "Lists message names defined for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "MessageDefinitions",
        "Lists stored message template definitions for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "MessageTags",
        "Lists message tags defined globally, for a symbol, or for one message query.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "OwnValues",
        "Lists stored immediate or delayed ownvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "UpValues",
        "Lists stored tagged upvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "DownValues",
        "Lists stored downvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "Attributes",
        "Lists the currently active attributes for a symbol.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "SetAttributes",
        "Adds attributes to one or more symbols.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "ClearAttributes",
        "Removes attributes from one or more symbols.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "Unset",
        "Removes a specific stored definition or message template.",
        HoldAttribute::HoldFirst,
    ),
    BuiltinSpec::with_hold(
        "Clear",
        "Clears values and rules from one or more symbols.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "ClearAll",
        "Clears values, rules, messages, and user-defined attributes from one or more symbols.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "Check",
        "Evaluates an expression and substitutes a fallback if matching messages are emitted.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "Quiet",
        "Evaluates an expression while suppressing matching runtime messages it emits.",
        HoldAttribute::HoldAll,
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
    BuiltinSpec::with_attributes(
        "HoldAllComplete",
        "Prevents evaluation of all arguments and disables Evaluate and Sequence escapes.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
    ),
    BuiltinSpec::with_attributes(
        "Unevaluated",
        "Temporarily leaves an argument unevaluated as a direct function argument.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
    ),
    BuiltinSpec::with_hold(
        "Evaluate",
        "Forces evaluation in held contexts.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "HoldPattern",
        "Marks a pattern expression so it is matched without prior evaluation.",
        HoldAttribute::HoldAll,
    ),
    BuiltinSpec::with_hold(
        "ReleaseHold",
        "Removes supported hold wrappers and resumes evaluation.",
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
