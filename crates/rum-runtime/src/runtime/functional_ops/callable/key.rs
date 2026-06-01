use super::*;

pub(super) fn parse_key_wrapper(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") => match args.as_slice() {
            [key] => Some(key),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn apply_key_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let [expr] = args else {
        return None;
    };

    let key = parse_key_wrapper(head)?;
    let entries = association_entries_from_expr(expr)?;
    Some(
        entries
            .iter()
            .find(|(entry_key, _)| entry_key == key)
            .map(|(_, value)| value.clone())
            .unwrap_or_else(|| missing_key_absent_expr(key.clone())),
    )
}
