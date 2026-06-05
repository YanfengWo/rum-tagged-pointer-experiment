use rum_core::{Atom, Expr};

pub fn render_input_form_like(expr: &Expr) -> String {
    match expr {
        Expr::Atom(atom) => atom.to_string(),
        Expr::Call { head, args } => render_call(head, args),
    }
}

fn render_call(head: &Expr, args: &[Expr]) -> String {
    match symbol_head_name(head) {
        Some("List") => render_wrapped("{", "}", args),
        Some("Association") => render_wrapped("<|", "|>", args),
        Some("Rule") if args.len() == 2 => render_infix(&args[0], " -> ", &args[1]),
        Some("RuleDelayed") if args.len() == 2 => render_infix(&args[0], " :> ", &args[1]),
        _ => {
            let rendered_args = args
                .iter()
                .map(render_input_form_like)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}[{rendered_args}]", render_input_form_like(head))
        }
    }
}

fn render_wrapped(open: &str, close: &str, args: &[Expr]) -> String {
    let body = args
        .iter()
        .map(render_input_form_like)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{open}{body}{close}")
}

fn render_infix(lhs: &Expr, op: &str, rhs: &Expr) -> String {
    format!(
        "{}{op}{}",
        render_input_form_like(lhs),
        render_input_form_like(rhs)
    )
}

fn symbol_head_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_lists_and_nested_lists_like_input_form() {
        let expr = Expr::call(
            Expr::system_symbol("List"),
            vec![
                Expr::symbol("a"),
                Expr::call(
                    Expr::system_symbol("List"),
                    vec![Expr::symbol("b"), Expr::symbol("c")],
                ),
            ],
        );

        assert_eq!(render_input_form_like(&expr), "{a, {b, c}}");
    }

    #[test]
    fn renders_rules_and_associations_like_input_form() {
        let expr = Expr::call(
            Expr::system_symbol("Association"),
            vec![
                Expr::call(
                    Expr::system_symbol("Rule"),
                    vec![Expr::symbol("a"), Expr::symbol("lhs")],
                ),
                Expr::call(
                    Expr::system_symbol("RuleDelayed"),
                    vec![Expr::symbol("b"), Expr::symbol("rhs")],
                ),
            ],
        );

        assert_eq!(render_input_form_like(&expr), "<|a -> lhs, b :> rhs|>");
    }
}
