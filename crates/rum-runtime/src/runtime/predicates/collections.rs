use super::*;

impl Session {
    pub(in crate::runtime) fn eval_duplicate_free_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("DuplicateFreeQ", args, 1));
        };

        let Some(items) = list_items(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut seen = std::collections::HashSet::new();
        for item in items {
            if !seen.insert(item.clone()) {
                return Ok(predicate_bool_expr(false));
            }
        }

        Ok(predicate_bool_expr(true))
    }

    pub(in crate::runtime) fn eval_intersecting_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let Some((lhs, rhs)) = two_list_args(args) else {
            return Ok(if args.len() == 2 {
                Expr::call(head.clone(), args.to_vec())
            } else {
                self.invalid_call_with_exact_arg_count("IntersectingQ", args, 2)
            });
        };

        let rhs_set = expr_set(rhs);
        Ok(predicate_bool_expr(
            lhs.iter().any(|item| rhs_set.contains(item)),
        ))
    }

    pub(in crate::runtime) fn eval_disjoint_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let Some((lhs, rhs)) = two_list_args(args) else {
            return Ok(if args.len() == 2 {
                Expr::call(head.clone(), args.to_vec())
            } else {
                self.invalid_call_with_exact_arg_count("DisjointQ", args, 2)
            });
        };

        let rhs_set = expr_set(rhs);
        Ok(predicate_bool_expr(
            !lhs.iter().any(|item| rhs_set.contains(item)),
        ))
    }

    pub(in crate::runtime) fn eval_subset_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let Some((lhs, rhs)) = two_list_args(args) else {
            return Ok(if args.len() == 2 {
                Expr::call(head.clone(), args.to_vec())
            } else {
                self.invalid_call_with_exact_arg_count("SubsetQ", args, 2)
            });
        };

        let rhs_set = expr_set(rhs);
        Ok(predicate_bool_expr(
            expr_set(lhs)
                .into_iter()
                .all(|item| rhs_set.contains(&item)),
        ))
    }

    pub(in crate::runtime) fn eval_contains_all(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_list_containment_relation(head, "ContainsAll", args, |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .all(|item| lhs_set.contains(&item))
        })
    }

    pub(in crate::runtime) fn eval_contains_any(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_list_containment_relation(head, "ContainsAny", args, |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .any(|item| lhs_set.contains(&item))
        })
    }

    pub(in crate::runtime) fn eval_contains_exactly(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_list_containment_relation(head, "ContainsExactly", args, |lhs, rhs| {
            expr_counts(lhs) == expr_counts(rhs)
        })
    }

    pub(in crate::runtime) fn eval_contains_none(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_list_containment_relation(head, "ContainsNone", args, |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .all(|item| !lhs_set.contains(&item))
        })
    }

    pub(in crate::runtime) fn eval_contains_only(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_list_containment_relation(head, "ContainsOnly", args, |lhs, rhs| {
            let rhs_set = expr_set(rhs);
            lhs.iter().all(|item| rhs_set.contains(item))
        })
    }

    fn eval_list_containment_relation<F>(
        &mut self,
        head: &Expr,
        name: &str,
        args: &[Expr],
        predicate: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&[Expr], &[Expr]) -> bool,
    {
        let Some((lhs, rhs)) = two_list_args(args) else {
            return Ok(if args.len() == 2 {
                Expr::call(head.clone(), args.to_vec())
            } else {
                self.invalid_call_with_exact_arg_count(name, args, 2)
            });
        };

        Ok(predicate_bool_expr(predicate(lhs, rhs)))
    }
}
