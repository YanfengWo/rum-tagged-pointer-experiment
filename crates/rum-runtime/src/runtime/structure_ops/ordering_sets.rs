use std::cmp::Ordering;

use super::*;

#[derive(Clone, Debug)]
struct SortByEntry {
    item: Expr,
    key: Expr,
    index: usize,
}

impl Session {
    pub(in crate::runtime) fn eval_sort(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => self.sort_normal_expression("Sort", args, expr),
            [expr, _ordering] => {
                if matches!(expr, Expr::Atom(_)) {
                    Ok(self.invalid_ordering_normal_call("Sort", args, 1))
                } else {
                    Ok(Expr::call(head.clone(), args.to_vec()))
                }
            }
            _ => Ok(self.invalid_call_with_arg_range("Sort", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_sort_by(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(sort_by_operator_form(function)),
            [expr, function] => {
                let Expr::Call {
                    head: expr_head,
                    args: expr_args,
                } = expr
                else {
                    return Ok(self.invalid_ordering_normal_call("SortBy", args, 1));
                };

                self.resume_sort_by_from_index(
                    args.to_vec(),
                    expr_head.as_ref().clone(),
                    function.clone(),
                    expr_args.to_vec(),
                    0,
                    Vec::new(),
                )
            }
            [expr, _function, _ordering] => {
                if matches!(expr, Expr::Atom(_)) {
                    Ok(self.invalid_ordering_normal_call("SortBy", args, 1))
                } else {
                    Ok(Expr::call(head.clone(), args.to_vec()))
                }
            }
            _ => Ok(self.invalid_call_with_arg_range("SortBy", args, 1, 3)),
        }
    }

    pub(in crate::runtime) fn eval_union(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(Expr::call(system_expr("List"), Vec::new()));
        }

        let (expr_head, groups) = match self.same_head_argument_groups("Union", args) {
            Ok(groups) => groups,
            Err(value) => return Ok(value),
        };
        let items = groups.into_iter().flatten().collect::<Vec<_>>();
        Ok(Expr::call(expr_head, sorted_unique_exprs(items)))
    }

    pub(in crate::runtime) fn eval_complement(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(self.invalid_call_with_message(
                "Complement",
                args,
                "argm",
                "Complement called with 0 arguments; 1 or more arguments are expected.",
            ));
        }

        let (expr_head, mut groups) = match self.same_head_argument_groups("Complement", args) {
            Ok(groups) => groups,
            Err(value) => return Ok(value),
        };
        let first = groups.remove(0);
        let excluded = groups.into_iter().flatten().collect::<Vec<_>>();
        let kept = first
            .into_iter()
            .filter(|item| !excluded.iter().any(|excluded| excluded == item))
            .collect::<Vec<_>>();
        Ok(Expr::call(expr_head, sorted_unique_exprs(kept)))
    }

    fn sort_normal_expression(
        &mut self,
        symbol: &str,
        args: &[Expr],
        expr: &Expr,
    ) -> Result<Expr, EvalError> {
        let Expr::Call {
            head: expr_head,
            args: expr_args,
        } = expr
        else {
            return Ok(self.invalid_ordering_normal_call(symbol, args, 1));
        };

        let mut sorted = expr_args.to_vec();
        sorted.sort_by(compare_structural_order);
        Ok(Expr::call(expr_head.as_ref().clone(), sorted))
    }

    fn same_head_argument_groups(
        &mut self,
        symbol: &str,
        args: &[Expr],
    ) -> Result<(Expr, Vec<Vec<Expr>>), Expr> {
        let Some((first_head, first_args)) = call_head_and_args(&args[0]) else {
            return Err(self.invalid_ordering_normal_call(symbol, args, 1));
        };

        let mut groups = vec![first_args.to_vec()];
        for (index, arg) in args.iter().enumerate().skip(1) {
            let Some((head, items)) = call_head_and_args(arg) else {
                return Err(self.invalid_ordering_normal_call(symbol, args, index + 1));
            };
            if head != first_head {
                return Err(self.invalid_ordering_heads_call(
                    symbol,
                    args,
                    index + 1,
                    head,
                    first_head,
                ));
            }
            groups.push(items.to_vec());
        }

        Ok((first_head.clone(), groups))
    }

    fn resume_sort_by_from_index(
        &mut self,
        call_args: Vec<Expr>,
        expr_head: Expr,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        decorated: Vec<SortByEntry>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            let mut sorted = decorated;
            sorted.sort_by(|lhs, rhs| {
                compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))
            });
            return Ok(Expr::call(
                expr_head,
                sorted.into_iter().map(|entry| entry.item).collect(),
            ));
        };

        let key_result = self.evaluate(Expr::call(function.clone(), vec![item.clone()]));
        let suspended_call_args = call_args.clone();
        let suspended_expr_head = expr_head.clone();
        let suspended_function = function.clone();
        let suspended_items = items.clone();
        let suspended_decorated = decorated.clone();
        let suspended_item = item.clone();
        let key = self.suspend_with_frame(key_result, move |session, key| {
            session.resume_sort_by_with_key(
                suspended_call_args,
                suspended_expr_head,
                suspended_function,
                suspended_items,
                index,
                suspended_decorated,
                suspended_item,
                key,
            )
        })?;

        self.resume_sort_by_with_key(
            call_args, expr_head, function, items, index, decorated, item, key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_sort_by_with_key(
        &mut self,
        call_args: Vec<Expr>,
        expr_head: Expr,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        mut decorated: Vec<SortByEntry>,
        item: Expr,
        key: Expr,
    ) -> Result<Expr, EvalError> {
        decorated.push(SortByEntry { item, key, index });
        self.resume_sort_by_from_index(call_args, expr_head, function, items, index + 1, decorated)
    }

    fn invalid_ordering_normal_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        position: usize,
    ) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "normal",
            format!(
                "Nonatomic expression expected at position {position} in {}.",
                Expr::call(system_expr(symbol), args.to_vec())
            ),
        )
    }

    fn invalid_ordering_heads_call(
        &mut self,
        symbol: &str,
        args: &[Expr],
        position: usize,
        actual_head: &Expr,
        expected_head: &Expr,
    ) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "heads",
            format!(
                "Heads {actual_head} and {expected_head} at positions {position} and 1 are expected to be the same."
            ),
        )
    }
}

fn sort_by_operator_form(function: &Expr) -> Expr {
    let arg = Expr::symbol("rum$sortArg");
    Expr::call(
        system_expr("Function"),
        vec![
            arg.clone(),
            Expr::call(system_expr("SortBy"), vec![arg, function.clone()]),
        ],
    )
}

fn call_head_and_args(expr: &Expr) -> Option<(&Expr, &[Expr])> {
    match expr {
        Expr::Call { head, args } => Some((head.as_ref(), args)),
        Expr::Atom(_) => None,
    }
}

fn sorted_unique_exprs(mut items: Vec<Expr>) -> Vec<Expr> {
    items.sort_by(compare_structural_order);
    let mut unique = Vec::new();
    for item in items {
        if unique.iter().any(|existing| existing == &item) {
            continue;
        }
        unique.push(item);
    }
    unique
}

fn compare_structural_order(lhs: &Expr, rhs: &Expr) -> Ordering {
    match (lhs, rhs) {
        (Expr::Atom(lhs), Expr::Atom(rhs)) => compare_atom_order(lhs, rhs),
        (Expr::Atom(_), Expr::Call { .. }) => Ordering::Less,
        (Expr::Call { .. }, Expr::Atom(_)) => Ordering::Greater,
        (
            Expr::Call {
                head: lhs_head,
                args: lhs_args,
            },
            Expr::Call {
                head: rhs_head,
                args: rhs_args,
            },
        ) => compare_structural_order(lhs_head, rhs_head)
            .then_with(|| compare_expr_slices(lhs_args, rhs_args)),
    }
}

fn compare_atom_order(lhs: &Atom, rhs: &Atom) -> Ordering {
    atom_order_rank(lhs)
        .cmp(&atom_order_rank(rhs))
        .then_with(|| match (lhs, rhs) {
            (Atom::Integer(lhs), Atom::Integer(rhs)) => lhs.cmp(rhs),
            (Atom::String(lhs), Atom::String(rhs)) => lhs.cmp(rhs),
            (Atom::Symbol(lhs), Atom::Symbol(rhs)) => lhs.to_string().cmp(&rhs.to_string()),
            (Atom::Rational(lhs), Atom::Rational(rhs)) => lhs.cmp(rhs),
            _ => lhs.to_string().cmp(&rhs.to_string()),
        })
}

fn atom_order_rank(atom: &Atom) -> u8 {
    match atom {
        Atom::Integer(_) | Atom::Rational(_) => 0,
        Atom::String(_) => 1,
        Atom::Symbol(_) => 2,
    }
}

fn compare_expr_slices(lhs: &[Expr], rhs: &[Expr]) -> Ordering {
    for (lhs, rhs) in lhs.iter().zip(rhs) {
        let ordering = compare_structural_order(lhs, rhs);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    lhs.len().cmp(&rhs.len())
}
