use super::*;

pub(super) const RULE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Part",
        "Extracts structural parts using integer positions, All, Span, and exact nested part specifications.",
        Session::eval_part,
    ).protected(),
    BuiltinSpec::ordinary(
        "Extract",
        "Extracts parts at structural positions, optionally wrapping each result before evaluation.",
        Session::eval_extract,
    ).protected(),
    BuiltinSpec::ordinary(
        "ReplacePart",
        "Replaces structural parts via immediate or delayed rules over exact, Span, and pattern-based position specifications, with Heads-aware matching.",
        Session::eval_replace_part,
    ).protected(),
    BuiltinSpec::special(
        "ReplaceAt",
        "Applies rules at exact structural positions, with the current operator-form and exact-position first-pass subset.",
        Session::eval_replace_at,
    ).protected(),
    BuiltinSpec::special(
        "Set",
        "Assigns an immediate value to a symbol or pattern.",
        builtin_set,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected exactly `1` argument(s), got `2`",
        },
        BuiltinMessageSpec {
            tag: "kval",
            template: "The value for the key `1` does not exist.",
        },
        BuiltinMessageSpec {
            tag: "pkspec1",
            template: "The expression `1` cannot be used as a part specification.",
        },
        BuiltinMessageSpec {
            tag: "shape",
            template: "Lists `1` and `2` are not the same shape.",
        },
        BuiltinMessageSpec {
            tag: "write",
            template: "Tag `1` in `2` is Protected.",
        },
        BuiltinMessageSpec {
            tag: "wrsym",
            template: "Symbol `1` is Protected.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "AddTo",
        "Adds an evaluated increment to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_add_to,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "SubtractFrom",
        "Subtracts an evaluated decrement from a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_subtract_from,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "TimesBy",
        "Multiplies a held symbol or exact Part target by an evaluated factor and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_times_by,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "DivideBy",
        "Divides a held symbol or exact Part target by an evaluated divisor and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_divide_by,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ApplyTo",
        "Applies an evaluated function to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_apply_to,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "AppendTo",
        "Appends an evaluated item to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_append_to,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "PrependTo",
        "Prepends an evaluated item to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_prepend_to,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Increment",
        "Increments a held symbol or exact Part target and returns its previous value.",
        HoldAttribute::HoldFirst,
        builtin_increment,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Decrement",
        "Decrements a held symbol or exact Part target and returns its previous value.",
        HoldAttribute::HoldFirst,
        builtin_decrement,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "PreIncrement",
        "Increments a held symbol or exact Part target and returns the updated value.",
        HoldAttribute::HoldFirst,
        builtin_pre_increment,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "PreDecrement",
        "Decrements a held symbol or exact Part target and returns the updated value.",
        HoldAttribute::HoldFirst,
        builtin_pre_decrement,
    ).protected(),
    BuiltinSpec::special(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
        builtin_set_delayed,
    ).protected(),
    BuiltinSpec::special(
        "UpSet",
        "Stores an immediate upvalue definition on one or more tag symbols inferred from the lhs arguments.",
        builtin_up_set,
    ).protected(),
    BuiltinSpec::special(
        "UpSetDelayed",
        "Stores a delayed upvalue definition on one or more tag symbols inferred from the lhs arguments.",
        builtin_up_set_delayed,
    ).protected(),
    BuiltinSpec::special(
        "TagSet",
        "Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.",
        builtin_tag_set,
    ).protected(),
    BuiltinSpec::special(
        "TagSetDelayed",
        "Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.",
        builtin_tag_set_delayed,
    ).protected(),
    BuiltinSpec::special(
        "TagUnset",
        "Removes a tagged definition using an explicit tag and lhs.",
        builtin_tag_unset,
    ).protected(),
    BuiltinSpec::special(
        "Rule",
        "Represents an immediate rewrite rule.",
        builtin_rule,
    ).protected(),
    BuiltinSpec::special(
        "RuleDelayed",
        "Represents a delayed rewrite rule.",
        builtin_rule_delayed,
    ).protected(),
    BuiltinSpec::special(
        "Count",
        "Counts subexpressions matching a pattern on levels specified by a standard levelspec.",
        builtin_count,
    ).protected(),
    BuiltinSpec::special(
        "MemberQ",
        "Tests whether any subexpression matches a pattern on levels specified by a standard levelspec.",
        builtin_member_q,
    ).protected(),
    BuiltinSpec::special(
        "FreeQ",
        "Tests whether no subexpression matches a pattern on levels specified by a standard levelspec.",
        builtin_free_q,
    ).protected(),
    BuiltinSpec::ordinary(
        "Level",
        "Collects subexpressions found on levels specified by a standard levelspec.",
        Session::eval_level,
    ).protected(),
    BuiltinSpec::special(
        "Position",
        "Finds positions of subexpressions matching a pattern on levels specified by a standard levelspec.",
        builtin_position,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "NormalizeRules",
        "Normalizes a rule or rule list into an explicit canonical list form for debugging.",
        HoldAttribute::HoldAll,
        builtin_normalize_rules,
    ).protected(),
];
