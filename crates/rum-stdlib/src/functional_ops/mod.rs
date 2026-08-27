use rum_core::{Atom, BigInt, Expr, Symbol};
use rum_runtime::kernel_api::{
    CasesLevelSpec, EvalError, FrameRootTrace, FrameRootVisitor, Session, ThreadArgSpec,
    ThreadingDispatch, association_entries_from_expr, call_items_with_head, exact_expr_at_path,
    exact_position_paths_from_expr, expr_as_exact_integer, expr_as_non_negative_usize,
    expr_as_signed_isize, expr_as_symbol, is_system_symbol_name, parse_heads_option,
    parse_heads_option_with_default, parse_standard_level_spec, render_association,
    replace_exact_path, system_expr, truthiness_symbol,
};

mod apply_map;
mod arrays;
mod callable;
mod iteration;
mod sequence;
mod targeted_map;

pub(crate) use self::{
    apply_map::{
        eval_apply, eval_comap, eval_comap_apply, eval_map, eval_map_all, eval_map_apply,
        eval_map_indexed, eval_scan,
    },
    arrays::{eval_array, eval_array_filter, eval_array_reduce, eval_dot, eval_inner, eval_outer},
    callable::*,
    iteration::{
        eval_fixed_point, eval_fixed_point_list, eval_nest, eval_nest_list, eval_nest_while,
        eval_nest_while_list,
    },
    sequence::{
        eval_accumulate, eval_block_map, eval_fold, eval_fold_list, eval_fold_pair,
        eval_fold_while, eval_length_while, eval_select, eval_select_first, eval_sequence_fold,
        eval_sequence_fold_list, eval_take_while,
    },
    targeted_map::{eval_map_at, eval_map_thread, eval_subset_map},
};

fn slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}

fn slot_sequence_expr() -> Expr {
    Expr::call(
        system_expr("SlotSequence"),
        vec![Expr::integer(BigInt::from(1_u8))],
    )
}

fn unary_operator_form(symbol: &str, operator: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![operator.clone(), slot_expr()],
        )],
    )
}

fn unary_operator_form_slot_first(symbol: &str, tail: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![slot_expr(), tail.clone()],
        )],
    )
}

fn unary_operator_form_with_tail(symbol: &str, operator: Expr, tail: Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![operator, slot_expr(), tail],
        )],
    )
}

fn unary_operator_form_with_tails(symbol: &str, operator: Expr, tails: Vec<Expr>) -> Expr {
    let mut args = vec![operator, slot_expr()];
    args.extend(tails);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), args)],
    )
}

fn variadic_operator_form(symbol: &str, mut fixed_args: Vec<Expr>) -> Expr {
    fixed_args.push(slot_sequence_expr());
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), fixed_args)],
    )
}

fn variadic_operator_form_with_suffix(
    symbol: &str,
    mut prefix_args: Vec<Expr>,
    suffix_args: Vec<Expr>,
) -> Expr {
    prefix_args.push(slot_sequence_expr());
    prefix_args.extend(suffix_args);
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(system_expr(symbol), prefix_args)],
    )
}

fn function_body_operator(body: Expr) -> Expr {
    Expr::call(system_expr("Function"), vec![body])
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    let value = expr_as_non_negative_usize(expr)?;
    (value > 0).then_some(value)
}

fn nest_operator_form(symbol: &str, operator: &Expr, count: usize) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![
                operator.clone(),
                slot_expr(),
                Expr::integer(BigInt::from(count)),
            ],
        )],
    )
}

fn extend_isize_path(path: &[isize], next: isize) -> Vec<isize> {
    let mut extended = path.to_vec();
    extended.push(next);
    extended
}

fn position_expr_from_path(path: &[isize]) -> Expr {
    Expr::call(
        system_expr("List"),
        path.iter()
            .map(|component| Expr::integer(BigInt::from(*component)))
            .collect(),
    )
}

fn generated_expr_nodes_memory_proxy_bytes(count: usize) -> usize {
    count.saturating_mul(std::mem::size_of::<Expr>())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
enum NestWhileTestArity {
    Count(usize),
    All,
}

fn nest_while_test_args(history: &[Expr], arity: NestWhileTestArity) -> Vec<Expr> {
    match arity {
        NestWhileTestArity::Count(count) => history
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        NestWhileTestArity::All => history.to_vec(),
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<NestWhileTestArity>();
};
