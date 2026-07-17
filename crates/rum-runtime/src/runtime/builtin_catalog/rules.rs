use super::message_templates::*;
use super::*;

pub(super) const RULE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Part",
        "Extracts structural parts using integer positions, All, Span, and exact nested part specifications.",
        Session::eval_part,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argm",
            template: "Part called with 0 arguments; 1 or more arguments are expected.",
        },
        PARTD,
        PARTW,
        PKSPEC1,
        BuiltinMessageSpec {
            tag: "pmix",
            template: "Integer and Key specifications cannot be mixed in the same part specification.",
        },
        PSPEC1,
        BuiltinMessageSpec {
            tag: "span",
            template: "Reports an invalid Span part specification.",
        },
        BuiltinMessageSpec {
            tag: "take",
            template: "Reports positions that cannot be taken from an expression.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Extract",
        "Extracts parts at structural positions, optionally wrapping each result before evaluation.",
        Session::eval_extract,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Extract called with `1` arguments; 2 or 3 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "keyw",
            template: "Key `1` does not exist in `2`.",
        },
        PARTD,
        PARTW,
        BuiltinMessageSpec {
            tag: "psl1",
            template: "Position specification `1` in `2` is not applicable.",
        },
        PSPEC1,
        BuiltinMessageSpec {
            tag: "span",
            template: "Reports an invalid Span part specification.",
        },
        BuiltinMessageSpec {
            tag: "take",
            template: "Reports positions that cannot be taken from an expression.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "ReplacePart",
        "Replaces structural parts via immediate or delayed rules over exact, Span, and pattern-based position specifications, with Heads-aware matching.",
        Session::eval_replace_part,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a Heads -> True/False option as the third argument",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "span",
            template: "`1` is not a valid Span specification for ReplacePart; expected a valid Part Span with integer, All, or UpTo endpoints and a nonzero integer step.",
        },
    ])
    .protected(),
    BuiltinSpec::special(
        "ReplaceAt",
        "Applies rules at exact structural positions, with the current operator-form and exact-position first-pass subset.",
        Session::eval_replace_at,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid rule list or exact part specification.",
        },
        ARGTU,
    ])
    .protected(),
    BuiltinSpec::special(
        "Set",
        "Assigns an immediate value to a symbol or pattern.",
        builtin_set,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid assignment target or assigned metadata value.",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "kval",
            template: "The value for the key `1` does not exist.",
        },
        PKSPEC1,
        BuiltinMessageSpec {
            tag: "shape",
            template: "Lists `1` and `2` are not the same shape.",
        },
        BuiltinMessageSpec {
            tag: "write",
            template: "Tag `1` in `2` is Protected.",
        },
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "AddTo",
        "Adds an evaluated increment to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_add_to,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "SubtractFrom",
        "Subtracts an evaluated decrement from a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_subtract_from,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "TimesBy",
        "Multiplies a held symbol or exact Part target by an evaluated factor and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_times_by,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "DivideBy",
        "Divides a held symbol or exact Part target by an evaluated divisor and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_divide_by,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "ApplyTo",
        "Applies an evaluated function to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_apply_to,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "AppendTo",
        "Appends an evaluated item to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_append_to,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "PrependTo",
        "Prepends an evaluated item to a held symbol or exact Part target and stores the result.",
        HoldAttribute::HoldFirst,
        builtin_prepend_to,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "normal",
            template: "Nonatomic expression expected at position 1 in `1`.",
        },
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Increment",
        "Increments a held symbol or exact Part target and returns its previous value.",
        HoldAttribute::HoldFirst,
        builtin_increment,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "Decrement",
        "Decrements a held symbol or exact Part target and returns its previous value.",
        HoldAttribute::HoldFirst,
        builtin_decrement,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "PreIncrement",
        "Increments a held symbol or exact Part target and returns the updated value.",
        HoldAttribute::HoldFirst,
        builtin_pre_increment,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "PreDecrement",
        "Decrements a held symbol or exact Part target and returns the updated value.",
        HoldAttribute::HoldFirst,
        builtin_pre_decrement,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an exact Part target as the first argument",
        },
        ARGX_EXACT,
        RVALUE,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::special(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
        builtin_set_delayed,
    )
    .with_messages(&[
        ARGX_EXACT,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::special(
        "UpSet",
        "Stores an immediate upvalue definition on one or more tag symbols inferred from the lhs arguments.",
        builtin_up_set,
    )
    .with_messages(&[
        ARGX_EXACT,
        NOSYM,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::special(
        "UpSetDelayed",
        "Stores a delayed upvalue definition on one or more tag symbols inferred from the lhs arguments.",
        builtin_up_set_delayed,
    )
    .with_messages(&[
        ARGX_EXACT,
        NOSYM,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::special(
        "TagSet",
        "Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.",
        builtin_tag_set,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string tag as the first argument",
        },
        ARGX_EXACT,
        TAGNF,
        TAGPOS,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::special(
        "TagSetDelayed",
        "Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.",
        builtin_tag_set_delayed,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string tag as the first argument",
        },
        ARGX_EXACT,
        TAGNF,
        TAGPOS,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::special(
        "TagUnset",
        "Removes a tagged definition using an explicit tag and lhs.",
        builtin_tag_unset,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a symbol or symbol-name string tag as the first argument",
        },
        ARGX_EXACT,
        TAGNF,
        TAGPOS,
        WRSYM,
    ])
    .protected(),
    BuiltinSpec::special(
        "Rule",
        "Represents an immediate rewrite rule.",
        builtin_rule,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::special(
        "RuleDelayed",
        "Represents a delayed rewrite rule.",
        builtin_rule_delayed,
    )
    .with_messages(&[
        ARGX_EXACT,
    ])
    .protected(),
    BuiltinSpec::special(
        "Count",
        "Counts subexpressions matching a pattern on levels specified by a standard levelspec.",
        builtin_count,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid traversal argument or Heads option.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "argx",
            template: "`1` called with `2` arguments; 1 argument is expected.",
        },
        LEVEL,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected(),
    BuiltinSpec::special(
        "MemberQ",
        "Tests whether any subexpression matches a pattern on levels specified by a standard levelspec.",
        builtin_member_q,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid traversal argument or Heads option.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "argx",
            template: "`1` called with `2` arguments; 1 argument is expected.",
        },
        LEVEL,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected(),
    BuiltinSpec::special(
        "FreeQ",
        "Tests whether no subexpression matches a pattern on levels specified by a standard levelspec.",
        builtin_free_q,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid traversal argument or Heads option.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "argx",
            template: "`1` called with `2` arguments; 1 argument is expected.",
        },
        LEVEL,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "Level",
        "Collects subexpressions found on levels specified by a standard levelspec.",
        Session::eval_level,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid Heads option.",
        },
        ARGTU,
        LEVEL,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected(),
    BuiltinSpec::special(
        "Position",
        "Finds positions of subexpressions matching a pattern on levels specified by a standard levelspec.",
        builtin_position,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "Reports an invalid traversal argument or Heads option.",
        },
        ARGTU,
        BuiltinMessageSpec {
            tag: "argx",
            template: "`1` called with `2` arguments; 1 argument is expected.",
        },
        INNF,
        LEVEL,
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        OPTTF,
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "NormalizeRules",
        "Normalizes a rule or rule list into an explicit canonical list form for debugging.",
        HoldAttribute::HoldAll,
        builtin_normalize_rules,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a rule, a rule list, or a symbol that evaluates to rules",
        },
        ARGX_EXACT,
    ])
    .protected(),
];
