use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TruthAggregateKind {
    All,
    Any,
    None,
}

impl Session {
    pub(super) fn eval_true_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("TrueQ", args, 1));
        };

        Ok(predicate_bool_expr(is_system_symbol_name(
            expr,
            heads::structure::TRUE,
        )))
    }

    pub(super) fn eval_boolean_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("BooleanQ", args, 1));
        };

        Ok(predicate_bool_expr(truthiness_symbol(expr).is_some()))
    }

    pub(super) fn eval_atom_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AtomQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(expr, Expr::Atom(_))))
    }

    pub(super) fn eval_list_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("ListQ", args, 1));
        };

        Ok(predicate_bool_expr(
            matches!(expr, Expr::Call { head, .. } if is_system_symbol_name(head, heads::structure::LIST)),
        ))
    }

    pub(super) fn eval_string_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("StringQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(
            expr,
            Expr::Atom(Atom::String(_))
        )))
    }

    pub(super) fn eval_integer_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("IntegerQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(
            expr,
            Expr::Atom(Atom::Integer(_))
        )))
    }

    pub(super) fn eval_number_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("NumberQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(
            expr,
            Expr::Atom(Atom::Integer(_) | Atom::Rational(_))
        )))
    }

    pub(super) fn eval_numeric_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("NumericQ", args, 1));
        };

        Ok(predicate_bool_expr(exact_numeric_value(expr).is_some()))
    }

    pub(super) fn eval_even_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("EvenQ", args, 1));
        };

        let is_even = match expr {
            Expr::Atom(Atom::Integer(value)) => value % BigInt::from(2_u8) == BigInt::from(0_u8),
            _ => false,
        };

        Ok(predicate_bool_expr(is_even))
    }

    pub(super) fn eval_odd_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("OddQ", args, 1));
        };

        let is_odd = match expr {
            Expr::Atom(Atom::Integer(value)) => value % BigInt::from(2_u8) != BigInt::from(0_u8),
            _ => false,
        };

        Ok(predicate_bool_expr(is_odd))
    }

    pub(super) fn eval_positive(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate("Positive", args, |value| {
            value.numerator() > &BigInt::from(0_u8)
        })
    }

    pub(super) fn eval_negative(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate("Negative", args, |value| {
            value.numerator() < &BigInt::from(0_u8)
        })
    }

    pub(super) fn eval_non_positive(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate("NonPositive", args, |value| {
            value.numerator() <= &BigInt::from(0_u8)
        })
    }

    pub(super) fn eval_non_negative(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate("NonNegative", args, |value| {
            value.numerator() >= &BigInt::from(0_u8)
        })
    }

    pub(super) fn eval_possible_zero_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate("PossibleZeroQ", args, |value| value.is_zero())
    }

    pub(super) fn eval_machine_number_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("MachineNumberQ", args, 1));
        };

        Ok(predicate_bool_expr(false))
    }

    pub(super) fn eval_real_valued_number_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("RealValuedNumberQ", args, 1));
        };

        Ok(predicate_bool_expr(exact_numeric_value(expr).is_some()))
    }

    pub(super) fn eval_real_valued_numeric_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("RealValuedNumericQ", args, 1));
        };

        Ok(predicate_bool_expr(exact_numeric_value(expr).is_some()))
    }

    pub(super) fn eval_prime_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("PrimeQ", args, 1));
        };

        Ok(predicate_bool_expr(
            exact_integer_value(expr).is_some_and(|value| is_prime_bigint(&value)),
        ))
    }

    pub(super) fn eval_algebraic_integer_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AlgebraicIntegerQ", args, 1));
        };

        Ok(predicate_bool_expr(exact_integer_value(expr).is_some()))
    }

    pub(super) fn eval_duplicate_free_q(
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

    pub(super) fn eval_intersecting_q(
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

    pub(super) fn eval_disjoint_q(
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

    pub(super) fn eval_subset_q(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_order(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [lhs, rhs] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Order", args, 2));
        };

        let ordering = compare_expr_canonical(lhs, rhs);
        let value = match ordering {
            std::cmp::Ordering::Less => -1_i64,
            std::cmp::Ordering::Equal => 0_i64,
            std::cmp::Ordering::Greater => 1_i64,
        };

        Ok(Expr::integer(BigInt::from(value)))
    }

    pub(super) fn eval_ordered_q(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("OrderedQ", args, 1));
        };

        let Some(items) = list_items(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(predicate_bool_expr(items.windows(2).all(|pair| {
            compare_expr_canonical(&pair[0], &pair[1]) != std::cmp::Ordering::Greater
        })))
    }

    pub(super) fn eval_contains_all(
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

    pub(super) fn eval_contains_any(
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

    pub(super) fn eval_contains_exactly(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_list_containment_relation(head, "ContainsExactly", args, |lhs, rhs| {
            expr_counts(lhs) == expr_counts(rhs)
        })
    }

    pub(super) fn eval_contains_none(
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

    pub(super) fn eval_contains_only(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_list_containment_relation(head, "ContainsOnly", args, |lhs, rhs| {
            let rhs_set = expr_set(rhs);
            lhs.iter().all(|item| rhs_set.contains(item))
        })
    }

    pub(super) fn eval_between(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [value_expr, interval_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Between", args, 2));
        };

        let Some(value) = exact_numeric_value(value_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(interval_items) = list_items(interval_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let [lower_expr, upper_expr] = interval_items else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let (Some(lower), Some(upper)) = (
            exact_numeric_value(lower_expr),
            exact_numeric_value(upper_expr),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(predicate_bool_expr(
            compare_exact_numeric(&lower, &value) != std::cmp::Ordering::Greater
                && compare_exact_numeric(&value, &upper) != std::cmp::Ordering::Greater,
        ))
    }

    pub(super) fn eval_vector_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("VectorQ", args, 1));
        };

        Ok(predicate_bool_expr(
            array_shape(expr).is_some_and(|shape| shape.len() == 1),
        ))
    }

    pub(super) fn eval_matrix_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("MatrixQ", args, 1));
        };

        Ok(predicate_bool_expr(
            array_shape(expr).is_some_and(|shape| shape.len() == 2),
        ))
    }

    pub(super) fn eval_array_q(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ArrayQ", args, 1, 2));
        }

        let Some(shape) = array_shape(&args[0]) else {
            return Ok(predicate_bool_expr(false));
        };

        if args.len() == 1 {
            return Ok(predicate_bool_expr(true));
        }

        let Some(rank) = expr_as_non_negative_usize(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(predicate_bool_expr(shape.len() == rank))
    }

    pub(super) fn eval_all_true(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_truth_aggregate(head, args, TruthAggregateKind::All)
    }

    pub(super) fn eval_any_true(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_truth_aggregate(head, args, TruthAggregateKind::Any)
    }

    pub(super) fn eval_none_true(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_truth_aggregate(head, args, TruthAggregateKind::None)
    }

    pub(super) fn eval_coprime_q(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let Some(values) = args
            .iter()
            .map(exact_integer_value)
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let one = BigInt::from(1_u8);
        for (index, lhs) in values.iter().enumerate() {
            for rhs in &values[index + 1..] {
                if bigint_gcd(bigint_abs(lhs), bigint_abs(rhs)) != one {
                    return Ok(predicate_bool_expr(false));
                }
            }
        }

        Ok(predicate_bool_expr(true))
    }

    pub(super) fn eval_same_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        Ok(predicate_bool_expr(
            args.windows(2).all(|pair| pair[0] == pair[1]),
        ))
    }

    pub(super) fn eval_unsame_q(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        for (index, lhs) in args.iter().enumerate() {
            if args[index + 1..].iter().any(|rhs| lhs == rhs) {
                return Ok(predicate_bool_expr(false));
            }
        }

        Ok(predicate_bool_expr(true))
    }

    fn eval_exact_numeric_sign_predicate<F>(
        &mut self,
        name: &str,
        args: &[Expr],
        predicate: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&Rational) -> bool,
    {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count(name, args, 1));
        };

        Ok(predicate_bool_expr(
            exact_numeric_value(expr).is_some_and(|value| predicate(&value)),
        ))
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

    fn eval_truth_aggregate(
        &mut self,
        head: &Expr,
        args: &[Expr],
        kind: TruthAggregateKind,
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            let symbol = truth_aggregate_name(kind);
            return Ok(self.invalid_call_with_arg_range(symbol, args, 2, 3));
        }

        let levels = if let Some(levelspec) = args.get(2) {
            let Some(levels) = parse_standard_level_spec(levelspec) else {
                return Ok(self.invalid_call_with_arg_form(
                    truth_aggregate_name(kind),
                    args,
                    "expected a standard levelspec as the third argument",
                ));
            };
            levels
        } else {
            CasesLevelSpec::default_cases()
        };

        let parts = collect_level_parts_owned(
            &args[0],
            CasesTraversalConfig {
                levels,
                max_matches: None,
                include_heads: false,
            },
            0,
        );

        self.resume_truth_aggregate_from_index(
            kind,
            head.clone(),
            args.to_vec(),
            parts,
            args[1].clone(),
            0,
            false,
        )
    }

    fn resume_truth_aggregate_from_index(
        &mut self,
        kind: TruthAggregateKind,
        head: Expr,
        call_args: Vec<Expr>,
        parts: Vec<Expr>,
        criterion: Expr,
        index: usize,
        saw_unknown: bool,
    ) -> Result<Expr, EvalError> {
        let Some(part) = parts.get(index).cloned() else {
            return Ok(finalize_truth_aggregate(kind, head, call_args, saw_unknown));
        };

        let call = Expr::call(criterion.clone(), vec![part]);
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_parts = parts.clone();
        let suspended_criterion = criterion.clone();
        let value_result = self.evaluate(call);
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_truth_aggregate_with_value(
                kind,
                suspended_head,
                suspended_call_args,
                suspended_parts,
                suspended_criterion,
                index,
                saw_unknown,
                value,
            )
        })?;

        self.resume_truth_aggregate_with_value(
            kind,
            head,
            call_args,
            parts,
            criterion,
            index,
            saw_unknown,
            value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_truth_aggregate_with_value(
        &mut self,
        kind: TruthAggregateKind,
        head: Expr,
        call_args: Vec<Expr>,
        parts: Vec<Expr>,
        criterion: Expr,
        index: usize,
        saw_unknown: bool,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        match truthiness_symbol(&value) {
            Some(true) => match kind {
                TruthAggregateKind::All => self.resume_truth_aggregate_from_index(
                    kind,
                    head,
                    call_args,
                    parts,
                    criterion,
                    index + 1,
                    saw_unknown,
                ),
                TruthAggregateKind::Any => Ok(system_expr(heads::structure::TRUE)),
                TruthAggregateKind::None => Ok(system_expr(heads::structure::FALSE)),
            },
            Some(false) => match kind {
                TruthAggregateKind::All => Ok(system_expr(heads::structure::FALSE)),
                TruthAggregateKind::Any | TruthAggregateKind::None => self
                    .resume_truth_aggregate_from_index(
                        kind,
                        head,
                        call_args,
                        parts,
                        criterion,
                        index + 1,
                        saw_unknown,
                    ),
            },
            None => self.resume_truth_aggregate_from_index(
                kind,
                head,
                call_args,
                parts,
                criterion,
                index + 1,
                true,
            ),
        }
    }
}

fn exact_numeric_value(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.clone())),
        Expr::Atom(Atom::Rational(value)) => Some(value.clone()),
        _ => None,
    }
}

fn exact_integer_value(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.clone()),
        _ => None,
    }
}

fn list_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

fn two_list_args(args: &[Expr]) -> Option<(&[Expr], &[Expr])> {
    let [lhs, rhs] = args else {
        return None;
    };

    Some((list_items(lhs)?, list_items(rhs)?))
}

fn expr_set(items: &[Expr]) -> std::collections::HashSet<Expr> {
    items.iter().cloned().collect()
}

fn expr_counts(items: &[Expr]) -> std::collections::HashMap<Expr, usize> {
    let mut counts = std::collections::HashMap::new();

    for item in items {
        *counts.entry(item.clone()).or_insert(0) += 1;
    }

    counts
}

fn array_shape(expr: &Expr) -> Option<Vec<usize>> {
    let items = list_items(expr)?;
    list_shape(items)
}

fn list_shape(items: &[Expr]) -> Option<Vec<usize>> {
    if items.is_empty() {
        return Some(vec![0]);
    }

    let child_shape = element_shape(&items[0])?;
    for item in &items[1..] {
        if element_shape(item)? != child_shape {
            return None;
        }
    }

    let mut shape = vec![items.len()];
    shape.extend(child_shape);
    Some(shape)
}

fn element_shape(expr: &Expr) -> Option<Vec<usize>> {
    match list_items(expr) {
        Some(items) => list_shape(items),
        None => Some(Vec::new()),
    }
}

fn is_prime_bigint(value: &BigInt) -> bool {
    let zero = BigInt::from(0_u8);
    let one = BigInt::from(1_u8);
    let two = BigInt::from(2_u8);

    if value <= &one {
        return false;
    }

    if value == &two {
        return true;
    }

    if value < &zero || value % &two == zero {
        return false;
    }

    let mut divisor = BigInt::from(3_u8);
    while &divisor * &divisor <= *value {
        if value % &divisor == zero {
            return false;
        }

        divisor += &two;
    }

    true
}

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_u8) {
        -value.clone()
    } else {
        value.clone()
    }
}

fn bigint_gcd(mut lhs: BigInt, mut rhs: BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);

    while rhs != zero {
        let remainder = lhs % rhs.clone();
        lhs = rhs;
        rhs = remainder;
    }

    lhs
}

fn truth_aggregate_name(kind: TruthAggregateKind) -> &'static str {
    match kind {
        TruthAggregateKind::All => "AllTrue",
        TruthAggregateKind::Any => "AnyTrue",
        TruthAggregateKind::None => "NoneTrue",
    }
}

fn finalize_truth_aggregate(
    kind: TruthAggregateKind,
    head: Expr,
    call_args: Vec<Expr>,
    saw_unknown: bool,
) -> Expr {
    if saw_unknown {
        Expr::call(head, call_args)
    } else {
        match kind {
            TruthAggregateKind::All | TruthAggregateKind::None => {
                system_expr(heads::structure::TRUE)
            }
            TruthAggregateKind::Any => system_expr(heads::structure::FALSE),
        }
    }
}

fn compare_exact_numeric(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}
