use super::*;

pub(super) const RULE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Part",
        "Extracts structural parts using integer positions, All, Span, and exact nested part specifications.",
        Session::eval_part,
    ),
    BuiltinSpec::ordinary(
        "Extract",
        "Extracts parts at structural positions, optionally wrapping each result before evaluation.",
        Session::eval_extract,
    ),
    BuiltinSpec::ordinary(
        "ReplacePart",
        "Replaces structural parts via immediate or delayed rules over exact, Span, and pattern-based position specifications, with Heads-aware matching.",
        Session::eval_replace_part,
    ),
    BuiltinSpec::special(
        "ReplaceAt",
        "Applies rules at exact structural positions, with the current operator-form and exact-position first-pass subset.",
        Session::eval_replace_at,
    ),
    BuiltinSpec::special(
        "Set",
        "Assigns an immediate value to a symbol or pattern.",
        builtin_set,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AddTo",
        "Adds an evaluated increment to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_add_to,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SubtractFrom",
        "Subtracts an evaluated decrement from a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_subtract_from,
    ),
    BuiltinSpec::ordinary_with_hold(
        "TimesBy",
        "Multiplies a held symbol or exact Part target by an evaluated factor and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_times_by,
    ),
    BuiltinSpec::ordinary_with_hold(
        "DivideBy",
        "Divides a held symbol or exact Part target by an evaluated divisor and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_divide_by,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ApplyTo",
        "Applies an evaluated function to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_apply_to,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AppendTo",
        "Appends an evaluated item to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_append_to,
    ),
    BuiltinSpec::ordinary_with_hold(
        "PrependTo",
        "Prepends an evaluated item to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_prepend_to,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Increment",
        "Increments a held symbol or exact Part target and returns its previous value.",
        HoldAttribute::HoldFirst,
        builtin_increment,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Decrement",
        "Decrements a held symbol or exact Part target and returns its previous value.",
        HoldAttribute::HoldFirst,
        builtin_decrement,
    ),
    BuiltinSpec::ordinary_with_hold(
        "PreIncrement",
        "Increments a held symbol or exact Part target and returns the updated value.",
        HoldAttribute::HoldFirst,
        builtin_pre_increment,
    ),
    BuiltinSpec::ordinary_with_hold(
        "PreDecrement",
        "Decrements a held symbol or exact Part target and returns the updated value.",
        HoldAttribute::HoldFirst,
        builtin_pre_decrement,
    ),
    BuiltinSpec::special(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
        builtin_set_delayed,
    ),
    BuiltinSpec::special(
        "UpSet",
        "Stores an immediate upvalue definition on one or more tag symbols inferred from the lhs arguments.",
        builtin_up_set,
    ),
    BuiltinSpec::special(
        "UpSetDelayed",
        "Stores a delayed upvalue definition on one or more tag symbols inferred from the lhs arguments.",
        builtin_up_set_delayed,
    ),
    BuiltinSpec::special(
        "TagSet",
        "Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.",
        builtin_tag_set,
    ),
    BuiltinSpec::special(
        "TagSetDelayed",
        "Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.",
        builtin_tag_set_delayed,
    ),
    BuiltinSpec::special(
        "TagUnset",
        "Removes a tagged definition using an explicit tag and lhs.",
        builtin_tag_unset,
    ),
    BuiltinSpec::special(
        "Rule",
        "Represents an immediate rewrite rule.",
        builtin_rule,
    ),
    BuiltinSpec::special(
        "RuleDelayed",
        "Represents a delayed rewrite rule.",
        builtin_rule_delayed,
    ),
    BuiltinSpec::special(
        "MatchQ",
        "Tests whether an expression matches a pattern.",
        builtin_match_q,
    ),
    BuiltinSpec::special(
        "Cases",
        "Collects subexpressions matching a pattern or rule.",
        builtin_cases,
    ),
    BuiltinSpec::special(
        "DeleteCases",
        "Deletes subexpressions matching a pattern on levels specified by a standard levelspec.",
        builtin_delete_cases,
    ),
    BuiltinSpec::special(
        "Count",
        "Counts subexpressions matching a pattern on levels specified by a standard levelspec.",
        builtin_count,
    ),
    BuiltinSpec::special(
        "MemberQ",
        "Tests whether any subexpression matches a pattern on levels specified by a standard levelspec.",
        builtin_member_q,
    ),
    BuiltinSpec::special(
        "FreeQ",
        "Tests whether no subexpression matches a pattern on levels specified by a standard levelspec.",
        builtin_free_q,
    ),
    BuiltinSpec::ordinary(
        "Level",
        "Collects subexpressions found on levels specified by a standard levelspec.",
        Session::eval_level,
    ),
    BuiltinSpec::special(
        "Position",
        "Finds positions of subexpressions matching a pattern on levels specified by a standard levelspec.",
        builtin_position,
    ),
    BuiltinSpec::special(
        "Replace",
        "Applies rules at the current first-pass Replace levelspec subset.",
        builtin_replace,
    ),
    BuiltinSpec::special(
        "ReplaceList",
        "Returns the current first-pass root replacement results across a rule list, with optional max-result limiting.",
        builtin_replace_list,
    ),
    BuiltinSpec::special(
        "ReplaceAll",
        "Applies rules everywhere in an expression.",
        builtin_replace_all,
    ),
    BuiltinSpec::special(
        "ReplaceRepeated",
        "Repeatedly applies rules until an expression reaches a fixed point.",
        builtin_replace_repeated,
    ),
    BuiltinSpec::ordinary_with_hold(
        "NormalizeRules",
        "Normalizes a rule or rule list into an explicit canonical list form for debugging.",
        HoldAttribute::HoldAll,
        builtin_normalize_rules,
    ),
];
