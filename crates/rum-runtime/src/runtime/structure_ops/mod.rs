use super::*;

mod combinatorial_lists;
mod dedup;
mod exact_paths;
mod flatten;
mod join;
mod list_plumbing;
mod ordering_sets;
mod reordering;
mod replace;
mod selectors;
mod sequence;
mod surface;

use exact_paths::{
    collect_exact_paths_for_position_spec, exact_part_component, normalize_structural_arg_position,
    select_exact_part,
};
use replace::ReplacePartHeadsMode;
use selectors::{
    AtomicPartSpec, PartSelector, apply_part_selectors, invalid_part_selector_for_part,
    parse_part_selectors, parse_part_selectors_for_part, parse_single_position_spec,
    select_positioned_parts_for_span,
};

pub(super) fn exact_position_paths_from_expr(
    expr: &Expr,
    spec_expr: &Expr,
) -> Option<Vec<Vec<isize>>> {
    exact_paths::exact_position_paths_from_expr(expr, spec_expr)
}

pub(super) fn exact_position_paths_from_part_selector_exprs(
    expr: &Expr,
    selector_exprs: &[Expr],
) -> Option<Vec<Vec<isize>>> {
    let selectors = parse_part_selectors(selector_exprs)?;
    Some(collect_exact_paths_for_position_spec(
        expr,
        &selectors,
        ReplacePartHeadsMode::Automatic,
    ))
}

pub(super) fn exact_position_paths_from_part_target_selector_exprs(
    expr: &Expr,
    selector_exprs: &[Expr],
) -> Option<Vec<Vec<isize>>> {
    let selectors = parse_part_selectors_for_part(selector_exprs)?;
    Some(collect_exact_paths_for_position_spec(
        expr,
        &selectors,
        ReplacePartHeadsMode::Automatic,
    ))
}

pub(super) fn invalid_part_target_selector_expr(selector_exprs: &[Expr]) -> Option<&Expr> {
    invalid_part_selector_for_part(selector_exprs)
}

pub(super) fn selected_expr_from_part_target_selector_exprs(
    expr: &Expr,
    selector_exprs: &[Expr],
) -> Option<Expr> {
    let selectors = parse_part_selectors_for_part(selector_exprs)?;
    apply_part_selectors(expr, &selectors)
}

pub(super) fn exact_expr_at_path(expr: &Expr, path: &[isize]) -> Option<Expr> {
    exact_paths::exact_expr_at_path(expr, path)
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
            if let Some(entries) = association_entries_from_expr(expr) {
                let head_depth = include_heads
                    .then(|| structural_depth(head, true))
                    .unwrap_or(0);
                let value_depth = entries
                    .iter()
                    .map(|(_, value)| structural_depth(value, include_heads))
                    .max()
                    .unwrap_or(0);
                return 1 + 1.max(head_depth).max(value_depth);
            }

            let head_depth = include_heads
                .then(|| structural_depth(head, true))
                .unwrap_or(0);
            let arg_depth = args
                .iter()
                .map(|arg| structural_depth(arg, include_heads))
                .max()
                .unwrap_or(0);
            1 + 1.max(head_depth).max(arg_depth)
        }
    }
}

fn structural_leaf_count(expr: &Expr) -> usize {
    match expr {
        Expr::Atom(_) => 1,
        Expr::Call { head, args } => {
            structural_leaf_count(head) + args.iter().map(structural_leaf_count).sum::<usize>()
        }
    }
}
