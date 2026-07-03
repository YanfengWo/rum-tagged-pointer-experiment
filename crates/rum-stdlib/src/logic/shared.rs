use rum_core::{Expr, heads};
use rum_runtime::kernel_api::{is_system_symbol_name, system_expr};

pub(super) fn negated_relation_head_name(head: &Expr) -> Option<&'static str> {
    [
        ("Equal", "Unequal"),
        ("Unequal", "Equal"),
        ("Less", "GreaterEqual"),
        ("Greater", "LessEqual"),
        ("LessEqual", "Greater"),
        ("GreaterEqual", "Less"),
    ]
    .into_iter()
    .find_map(|(name, opposite)| is_system_symbol_name(head, name).then_some(opposite))
}

pub(super) fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}
