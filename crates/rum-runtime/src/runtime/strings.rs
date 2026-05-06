use super::*;
mod encoding;
mod patterns;
mod similarity;
mod template;
mod text;

pub(in crate::runtime) fn render_input_like_expr(expr: &Expr) -> String {
    text::render_input_like_expr(expr)
}

fn bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

fn string_slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}

fn string_unary_operator_form(symbol: &str, operator: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![operator.clone(), string_slot_expr()],
        )],
    )
}

fn string_unary_operator_form_slot_first(symbol: &str, tail: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![string_slot_expr(), tail.clone()],
        )],
    )
}

fn string_apply_value(
    session: &mut Session,
    operator: &Expr,
    value: &str,
    fallback: Expr,
) -> Result<Expr, EvalError> {
    let applied = Expr::call(
        operator.clone(),
        vec![list_expr(
            value
                .chars()
                .map(|ch| Expr::string(ch.to_string()))
                .collect(),
        )],
    );
    let evaluated = session.evaluate(applied)?;
    Ok(string_apply_output_expr(evaluated).unwrap_or(fallback))
}

fn string_apply_output_expr(expr: Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::String(_)) => Some(expr),
        Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
            if args
                .iter()
                .all(|item| matches!(item, Expr::Atom(Atom::String(_))))
            {
                let joined = args
                    .into_iter()
                    .filter_map(|item| match item {
                        Expr::Atom(Atom::String(value)) => Some(value),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .concat();
                return Some(Expr::string(joined));
            }

            let converted = args
                .into_iter()
                .map(string_apply_output_expr)
                .collect::<Option<Vec<_>>>()?;
            Some(list_expr(converted))
        }
        _ => None,
    }
}

fn is_string_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r')
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn string_list_expr(items: Vec<String>) -> Expr {
    list_expr(items.into_iter().map(Expr::string).collect())
}
