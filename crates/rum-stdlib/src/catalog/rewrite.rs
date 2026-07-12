use rum_runtime::kernel_api::BuiltinSpec;

use crate::rewrite_builtins;

pub(crate) const REWRITE_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::special(
        "MatchQ",
        "Tests whether an expression matches a pattern.",
        rewrite_builtins::eval_match_q,
    ).protected(),
    BuiltinSpec::special(
        "Cases",
        "Collects subexpressions matching a pattern or rule.",
        rewrite_builtins::eval_cases,
    ).protected(),
    BuiltinSpec::special(
        "DeleteCases",
        "Deletes subexpressions matching a pattern on levels specified by a standard levelspec.",
        rewrite_builtins::eval_delete_cases,
    ).protected(),
    BuiltinSpec::special(
        "Replace",
        "Applies rules at the current first-pass Replace levelspec subset.",
        rewrite_builtins::eval_replace,
    ).protected(),
    BuiltinSpec::special(
        "ReplaceList",
        "Returns the current first-pass root replacement results across a rule list, with optional max-result limiting.",
        rewrite_builtins::eval_replace_list,
    ).protected(),
    BuiltinSpec::special(
        "ReplaceAll",
        "Applies rules everywhere in an expression.",
        rewrite_builtins::eval_replace_all,
    ).protected(),
    BuiltinSpec::special(
        "ReplaceRepeated",
        "Repeatedly applies rules until an expression reaches a fixed point.",
        rewrite_builtins::eval_replace_repeated,
    ).protected(),
];
