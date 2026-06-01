use super::*;

pub(in crate::runtime::logic) fn negated_relation_head_name(head: &Expr) -> Option<&'static str> {
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

pub(in crate::runtime::logic) fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}
