use rum_runtime::{BuiltinHandler, BuiltinSpec, HoldAttribute};

pub const CORE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Identity",
        "Returns its argument unchanged.",
        BuiltinHandler::Identity,
    ),
    BuiltinSpec::special(
        "Set",
        "Assigns an immediate value to a symbol or pattern.",
        BuiltinHandler::Set,
    ),
    BuiltinSpec::special(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
        BuiltinHandler::SetDelayed,
    ),
    BuiltinSpec::special(
        "TagSet",
        "Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.",
        BuiltinHandler::TagSet,
    ),
    BuiltinSpec::special(
        "TagSetDelayed",
        "Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.",
        BuiltinHandler::TagSetDelayed,
    ),
    BuiltinSpec::special(
        "TagUnset",
        "Removes a tagged definition using an explicit tag and lhs.",
        BuiltinHandler::TagUnset,
    ),
    BuiltinSpec::special(
        "Rule",
        "Represents an immediate rewrite rule.",
        BuiltinHandler::Rule,
    ),
    BuiltinSpec::special(
        "RuleDelayed",
        "Represents a delayed rewrite rule.",
        BuiltinHandler::RuleDelayed,
    ),
    BuiltinSpec::special(
        "MatchQ",
        "Tests whether an expression matches a pattern.",
        BuiltinHandler::MatchQ,
    ),
    BuiltinSpec::special(
        "Cases",
        "Collects subexpressions matching a pattern or rule.",
        BuiltinHandler::Cases,
    ),
    BuiltinSpec::special(
        "ReplaceAll",
        "Applies rules everywhere in an expression.",
        BuiltinHandler::ReplaceAll,
    ),
    BuiltinSpec::special(
        "ReplaceRepeated",
        "Repeatedly applies rules until an expression reaches a fixed point.",
        BuiltinHandler::ReplaceRepeated,
    ),
    BuiltinSpec::ordinary_with_hold(
        "NormalizeRules",
        "Normalizes a rule or rule list into an explicit canonical list form for debugging.",
        HoldAttribute::HoldAll,
        BuiltinHandler::NormalizeRules,
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
    BuiltinSpec::ordinary_with_hold(
        "Context",
        "Returns the context of the current session or of a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Context,
    ),
    BuiltinSpec::new("File", "Represents an explicit file specification."),
    BuiltinSpec::ordinary(
        "FindFile",
        "Finds the full path of a file or package using the current search path.",
        BuiltinHandler::FindFile,
    ),
    BuiltinSpec::ordinary(
        "Get",
        "Reads a file or package and evaluates the expressions it contains.",
        BuiltinHandler::Get,
    ),
    BuiltinSpec::ordinary(
        "Needs",
        "Loads a package only if its context has not already been loaded.",
        BuiltinHandler::Needs,
    ),
    BuiltinSpec::ordinary(
        "Begin",
        "Switches into a context and prepends it to the context path.",
        BuiltinHandler::Begin,
    ),
    BuiltinSpec::ordinary(
        "End",
        "Restores the previous context after a matching Begin.",
        BuiltinHandler::End,
    ),
    BuiltinSpec::ordinary(
        "BeginPackage",
        "Starts a package context with a package-oriented context path.",
        BuiltinHandler::BeginPackage,
    ),
    BuiltinSpec::ordinary(
        "EndPackage",
        "Ends a package context and leaves it on the restored context path.",
        BuiltinHandler::EndPackage,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Message",
        "Emits a message described by a MessageName expression.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Message,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageName",
        "Represents a symbolic message name such as f::tag.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageName,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Messages",
        "Lists message names defined for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Messages,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageDefinitions",
        "Lists stored message template definitions for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageDefinitions,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageTags",
        "Lists message tags defined globally, for a symbol, or for one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageTags,
    ),
    BuiltinSpec::ordinary_with_hold(
        "OwnValues",
        "Lists stored immediate or delayed ownvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::OwnValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "UpValues",
        "Lists stored tagged upvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::UpValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "DownValues",
        "Lists stored downvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::DownValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SubValues",
        "Lists stored subvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SubValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Attributes",
        "Lists the currently active attributes for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Attributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SetAttributes",
        "Adds attributes to one or more symbols.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SetAttributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ClearAttributes",
        "Removes attributes from one or more symbols.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::ClearAttributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Unset",
        "Removes a specific stored definition or message template.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Unset,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Clear",
        "Clears values and rules from one or more symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Clear,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ClearAll",
        "Clears values, rules, messages, and user-defined attributes from one or more symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ClearAll,
    ),
];
