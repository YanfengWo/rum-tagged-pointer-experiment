use super::*;

mod sequence;
mod exact_paths;
mod replace;
mod selectors;
mod surface;

use exact_paths::{
    collect_exact_paths_for_position_spec, exact_part_component, normalize_structural_arg_position,
    select_exact_part,
};
use replace::ReplacePartHeadsMode;
use selectors::{
    apply_part_selectors, parse_single_position_spec, select_positioned_parts_for_span,
    AtomicPartSpec, PartSelector, PartSpanSpec, SpanEndpoint,
};

pub(super) fn apply_keyed_part_spec(expr: &Expr, spec_expr: &Expr) -> Option<Expr> {
    selectors::apply_keyed_part_spec(expr, spec_expr)
}

pub(super) fn exact_position_paths_from_expr(
    expr: &Expr,
    spec_expr: &Expr,
) -> Option<Vec<Vec<isize>>> {
    exact_paths::exact_position_paths_from_expr(expr, spec_expr)
}

pub(super) fn replace_exact_path(expr: Expr, path: &[isize], rhs: Expr) -> Expr {
    exact_paths::replace_exact_path(expr, path, rhs)
}

impl Session {
    fn invalid_normal_expression_call(&mut self, symbol: &str, args: &[Expr]) -> Expr {
        self.push_message(
            heads::messages::GENERAL,
            "normal",
            format!(
                "Nonatomic expression expected at position 1 in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        );
        Expr::call(system_expr(symbol), args.to_vec())
    }
}

fn is_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST)
    )
}

fn is_empty_plain_list_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args }
            if is_system_symbol_name(head, heads::structure::LIST) && args.is_empty()
    )
}

fn structural_head_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Atom(Atom::Integer(_)) => system_expr("Integer"),
        Expr::Atom(Atom::Rational(_)) => system_expr("Rational"),
        Expr::Atom(Atom::String(_)) => system_expr("String"),
        Expr::Atom(Atom::Symbol(_)) => system_expr("Symbol"),
        Expr::Call { head, .. } => head.as_ref().clone(),
    }
}

fn structural_length(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 0,
        Expr::Call { args, .. } => args.len(),
    }
}

fn structural_depth(expr: &Expr, include_heads: bool) -> usize {
    match expr {
        Expr::Atom(_) => 1,
        Expr::Call { head, args } => {
            let head_depth = include_heads
                .then(|| structural_depth(head, true))
                .unwrap_or(0);
            let arg_depth = args
                .iter()
                .map(|arg| structural_depth(arg, include_heads))
                .max()
                .unwrap_or(0);
            1 + head_depth.max(arg_depth)
        }
    }
}
