use rum_core::{Atom, Expr};

#[path = "value_render/arithmetic.rs"]
mod arithmetic;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Precedence {
    Lowest = 0,
    Plus = 1,
    Times = 2,
    Power = 3,
    Atom = 4,
}

pub fn render_input_form_like(expr: &Expr) -> String {
    render_expr(expr, Precedence::Lowest)
}

pub(super) fn render_expr(expr: &Expr, parent_precedence: Precedence) -> String {
    let precedence = expr_precedence(expr);
    let rendered = match expr {
        Expr::Atom(atom) => atom.to_string(),
        Expr::Call { head, args } => render_call(head, args),
    };

    if precedence < parent_precedence {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn expr_precedence(expr: &Expr) -> Precedence {
    match expr {
        Expr::Atom(_) => Precedence::Atom,
        Expr::Call { head, .. } => match symbol_head_name(head) {
            Some("Plus") => Precedence::Plus,
            Some("Times") => Precedence::Times,
            Some("Power") => Precedence::Power,
            _ => Precedence::Atom,
        },
    }
}

fn render_call(head: &Expr, args: &[Expr]) -> String {
    match symbol_head_name(head) {
        Some("List") => render_wrapped("{", "}", args),
        Some("Association") => render_wrapped("<|", "|>", args),
        Some("Rule") if args.len() == 2 => render_infix(&args[0], " -> ", &args[1]),
        Some("RuleDelayed") if args.len() == 2 => render_rule_delayed(&args[0], &args[1]),
        Some("Set") if args.len() == 2 => render_infix(&args[0], " = ", &args[1]),
        Some("SetDelayed") if args.len() == 2 => render_infix(&args[0], " := ", &args[1]),
        Some("Plus") => arithmetic::render_plus(args),
        Some("Times") => arithmetic::render_times(args),
        Some("Power") if args.len() == 2 => arithmetic::render_power(&args[0], &args[1]),
        Some("MessageName") if args.len() == 2 => render_message_name(&args[0], &args[1]),
        Some("Blank") => render_blank("_", args),
        Some("BlankSequence") => render_blank("__", args),
        Some("BlankNullSequence") => render_blank("___", args),
        Some("Pattern") if args.len() == 2 => render_pattern(&args[0], &args[1]),
        _ => {
            let rendered_args = args
                .iter()
                .map(|arg| render_expr(arg, Precedence::Lowest))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}[{rendered_args}]", render_expr(head, Precedence::Lowest))
        }
    }
}

fn render_blank(prefix: &str, args: &[Expr]) -> String {
    match args {
        [] => prefix.to_string(),
        [head] => format!("{prefix}{}", render_expr(head, Precedence::Atom)),
        _ => render_head_form(prefix_to_blank_head(prefix), args),
    }
}

fn render_pattern(name: &Expr, pattern: &Expr) -> String {
    if let (Some(name), Some(blank)) = (symbol_head_name(name), render_blank_like(pattern)) {
        format!("{name}{blank}")
    } else {
        render_head_form("Pattern", &[name.clone(), pattern.clone()])
    }
}

fn render_blank_like(expr: &Expr) -> Option<String> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    match symbol_head_name(head) {
        Some("Blank") => Some(render_blank("_", args)),
        Some("BlankSequence") => Some(render_blank("__", args)),
        Some("BlankNullSequence") => Some(render_blank("___", args)),
        _ => None,
    }
}

fn prefix_to_blank_head(prefix: &str) -> &str {
    match prefix {
        "_" => "Blank",
        "__" => "BlankSequence",
        "___" => "BlankNullSequence",
        _ => unreachable!("only known blank prefixes are rendered"),
    }
}

fn render_head_form(head: &str, args: &[Expr]) -> String {
    let rendered_args = args
        .iter()
        .map(|arg| render_expr(arg, Precedence::Lowest))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{head}[{rendered_args}]")
}

fn render_message_name(symbol: &Expr, tag: &Expr) -> String {
    let Some(tag) = message_tag_text(tag) else {
        let rendered_args = [symbol, tag]
            .iter()
            .map(|arg| render_expr(arg, Precedence::Lowest))
            .collect::<Vec<_>>()
            .join(", ");
        return format!("MessageName[{rendered_args}]");
    };

    format!("{}::{tag}", render_expr(symbol, Precedence::Atom))
}

fn message_tag_text(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Atom(Atom::String(tag)) => Some(tag.clone()),
        Expr::Atom(Atom::Symbol(tag)) => Some(tag.name().to_string()),
        _ => None,
    }
}

fn render_wrapped(open: &str, close: &str, args: &[Expr]) -> String {
    let body = args
        .iter()
        .map(|arg| render_expr(arg, Precedence::Lowest))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{open}{body}{close}")
}

fn render_infix(lhs: &Expr, op: &str, rhs: &Expr) -> String {
    format!(
        "{}{op}{}",
        render_expr(lhs, Precedence::Lowest),
        render_expr(rhs, Precedence::Lowest)
    )
}

fn render_rule_delayed(lhs: &Expr, rhs: &Expr) -> String {
    let rhs = if is_assignment_like(rhs) {
        format!("({})", render_expr(rhs, Precedence::Lowest))
    } else {
        render_expr(rhs, Precedence::Lowest)
    };
    format!("{} :> {rhs}", render_expr(lhs, Precedence::Lowest))
}

fn is_assignment_like(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, .. } => matches!(symbol_head_name(head), Some("Set" | "SetDelayed")),
        Expr::Atom(_) => false,
    }
}

pub(super) fn symbol_head_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name()),
        _ => None,
    }
}

#[cfg(test)]
#[path = "value_render/tests.rs"]
mod tests;
