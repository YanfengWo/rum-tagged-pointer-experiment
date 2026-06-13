use super::*;

mod pair;
mod plain;
mod sequence_fold;
mod while_loop;

use pair::apply_fold_pair;
use plain::apply_fold;
use sequence_fold::apply_sequence_fold;
use while_loop::apply_fold_while;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FoldWhileConfig {
    pub(super) test_arity: NestWhileTestArity,
    pub(super) tail_adjustment: isize,
}

impl Session {
    pub(in crate::runtime) fn eval_fold(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(variadic_operator_form("Fold", vec![operator.clone()])),
            [operator, list_expr] => apply_fold(self, operator, None, list_expr, false),
            [operator, initial, list_expr] => {
                apply_fold(self, operator, Some(initial.clone()), list_expr, false)
            }
            _ => Ok(self.invalid_call_with_arg_range("Fold", args, 1, 3)),
        }
    }

    pub(in crate::runtime) fn eval_fold_list(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(variadic_operator_form("FoldList", vec![operator.clone()])),
            [operator, list_expr] => apply_fold(self, operator, None, list_expr, true),
            [operator, initial, list_expr] => {
                apply_fold(self, operator, Some(initial.clone()), list_expr, true)
            }
            _ => Ok(self.invalid_call_with_arg_range("FoldList", args, 1, 3)),
        }
    }

    pub(in crate::runtime) fn eval_fold_pair(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, list_expr] => {
                apply_fold_pair(self, head, operator, None, list_expr, None, args)
            }
            [operator, initial, list_expr] => apply_fold_pair(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                None,
                args,
            ),
            [operator, initial, list_expr, projector] => apply_fold_pair(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                Some(projector.clone()),
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("FoldPair", args, 2, 4)),
        }
    }

    pub(in crate::runtime) fn eval_fold_while(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, test] => Ok(variadic_operator_form_with_suffix(
                "FoldWhile",
                vec![operator.clone()],
                vec![test.clone()],
            )),
            [operator, list_expr, test] => apply_fold_while(
                self, head, operator, None, list_expr, test, None, None, args,
            ),
            [operator, initial, list_expr, test] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                None,
                None,
                args,
            ),
            [operator, initial, list_expr, test, m_expr] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                Some(m_expr),
                None,
                args,
            ),
            [operator, initial, list_expr, test, m_expr, n_expr] => apply_fold_while(
                self,
                head,
                operator,
                Some(initial.clone()),
                list_expr,
                test,
                Some(m_expr),
                Some(n_expr),
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("FoldWhile", args, 2, 6)),
        }
    }

    pub(in crate::runtime) fn eval_sequence_fold(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, seeds, items] => {
                apply_sequence_fold(self, head, operator, seeds, items, None, false, args)
            }
            [operator, seeds, items, arity_expr] => apply_sequence_fold(
                self,
                head,
                operator,
                seeds,
                items,
                Some(arity_expr),
                false,
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("SequenceFold", args, 3, 4)),
        }
    }

    pub(in crate::runtime) fn eval_sequence_fold_list(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator, seeds, items] => {
                apply_sequence_fold(self, head, operator, seeds, items, None, true, args)
            }
            [operator, seeds, items, arity_expr] => apply_sequence_fold(
                self,
                head,
                operator,
                seeds,
                items,
                Some(arity_expr),
                true,
                args,
            ),
            _ => Ok(self.invalid_call_with_arg_range("SequenceFoldList", args, 3, 4)),
        }
    }
}

pub(super) fn iterable_items(expr: &Expr) -> Option<Vec<Expr>> {
    match expr {
        Expr::Call { head, args: _ } if is_system_symbol_name(head, "Association") => Some(
            association_entries_from_expr(expr)?
                .into_iter()
                .map(|(_, value)| value)
                .collect(),
        ),
        Expr::Call { args, .. } => Some(args.to_vec()),
        Expr::Atom(_) => None,
    }
}

pub(super) fn parse_fold_while_config(
    m_expr: Option<&Expr>,
    n_expr: Option<&Expr>,
) -> Option<FoldWhileConfig> {
    let test_arity = match m_expr {
        Some(expr) if is_system_symbol_name(expr, "All") => NestWhileTestArity::All,
        Some(expr) => {
            let count = expr_as_non_negative_usize(expr)?;
            if count == 0 {
                return None;
            }
            NestWhileTestArity::Count(count)
        }
        None => NestWhileTestArity::Count(1),
    };

    let tail_adjustment = match n_expr {
        Some(expr) => expr_as_signed_isize(expr)?,
        None => 0,
    };

    Some(FoldWhileConfig {
        test_arity,
        tail_adjustment,
    })
}
