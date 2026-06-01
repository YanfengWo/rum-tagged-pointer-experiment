use super::*;

pub(super) const RULE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Part",
        "Extracts structural parts using integer positions, All, Span, and exact nested part specifications.",
        BuiltinHandler::Part,
    ),
    BuiltinSpec::ordinary(
        "Extract",
        "Extracts parts at structural positions, optionally wrapping each result before evaluation.",
        BuiltinHandler::Extract,
    ),
    BuiltinSpec::ordinary(
        "ReplacePart",
        "Replaces structural parts via immediate or delayed rules over exact, Span, and pattern-based position specifications, with Heads-aware matching.",
        BuiltinHandler::ReplacePart,
    ),
    BuiltinSpec::special(
        "ReplaceAt",
        "Applies rules at exact structural positions, with the current operator-form and exact-position first-pass subset.",
        BuiltinHandler::ReplaceAt,
    ),
    BuiltinSpec::special(
        "Set",
        "Assigns an immediate value to a symbol or pattern.",
        BuiltinHandler::Set,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AddTo",
        "Adds an evaluated increment to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::AddTo,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SubtractFrom",
        "Subtracts an evaluated decrement from a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SubtractFrom,
    ),
    BuiltinSpec::ordinary_with_hold(
        "TimesBy",
        "Multiplies a held symbol or exact Part target by an evaluated factor and stores the result.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::TimesBy,
    ),
    BuiltinSpec::ordinary_with_hold(
        "DivideBy",
        "Divides a held symbol or exact Part target by an evaluated divisor and stores the result.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::DivideBy,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ApplyTo",
        "Applies an evaluated function to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::ApplyTo,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AppendTo",
        "Appends an evaluated item to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::AppendTo,
    ),
    BuiltinSpec::ordinary_with_hold(
        "PrependTo",
        "Prepends an evaluated item to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::PrependTo,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Increment",
        "Increments a held symbol or exact Part target and returns its previous value.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Increment,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Decrement",
        "Decrements a held symbol or exact Part target and returns its previous value.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Decrement,
    ),
    BuiltinSpec::ordinary_with_hold(
        "PreIncrement",
        "Increments a held symbol or exact Part target and returns the updated value.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::PreIncrement,
    ),
    BuiltinSpec::ordinary_with_hold(
        "PreDecrement",
        "Decrements a held symbol or exact Part target and returns the updated value.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::PreDecrement,
    ),
    BuiltinSpec::special(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
        BuiltinHandler::SetDelayed,
    ),
    BuiltinSpec::special(
        "UpSet",
        "Stores an immediate upvalue definition on one or more tag symbols inferred from the lhs arguments.",
        BuiltinHandler::UpSet,
    ),
    BuiltinSpec::special(
        "UpSetDelayed",
        "Stores a delayed upvalue definition on one or more tag symbols inferred from the lhs arguments.",
        BuiltinHandler::UpSetDelayed,
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
        "DeleteCases",
        "Deletes subexpressions matching a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::DeleteCases,
    ),
    BuiltinSpec::special(
        "Count",
        "Counts subexpressions matching a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::Count,
    ),
    BuiltinSpec::special(
        "MemberQ",
        "Tests whether any subexpression matches a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::MemberQ,
    ),
    BuiltinSpec::special(
        "FreeQ",
        "Tests whether no subexpression matches a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::FreeQ,
    ),
    BuiltinSpec::ordinary(
        "Level",
        "Collects subexpressions found on levels specified by a standard levelspec.",
        BuiltinHandler::Level,
    ),
    BuiltinSpec::special(
        "Position",
        "Finds positions of subexpressions matching a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::Position,
    ),
    BuiltinSpec::special(
        "Replace",
        "Applies rules at the current first-pass Replace levelspec subset.",
        BuiltinHandler::Replace,
    ),
    BuiltinSpec::special(
        "ReplaceList",
        "Returns the current first-pass root replacement results across a rule list, with optional max-result limiting.",
        BuiltinHandler::ReplaceList,
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
];
