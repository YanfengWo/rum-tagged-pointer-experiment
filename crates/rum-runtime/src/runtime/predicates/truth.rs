use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TruthAggregateKind {
    All,
    Any,
    None,
}

impl Session {
    pub(in crate::runtime) fn eval_true_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("TrueQ", args, 1));
        };

        Ok(predicate_bool_expr(is_system_symbol_name(
            expr,
            heads::structure::TRUE,
        )))
    }

    pub(in crate::runtime) fn eval_boolean_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("BooleanQ", args, 1));
        };

        Ok(predicate_bool_expr(truthiness_symbol(expr).is_some()))
    }

    pub(in crate::runtime) fn eval_boole(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Boole", args, 1));
        };

        Ok(match truthiness_symbol(expr) {
            Some(true) => Expr::integer(BigInt::from(1_i64)),
            Some(false) => Expr::integer(BigInt::from(0_i64)),
            None => Expr::call(head.clone(), args.to_vec()),
        })
    }

    pub(in crate::runtime) fn eval_all_true(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_truth_aggregate(head, args, TruthAggregateKind::All)
    }

    pub(in crate::runtime) fn eval_any_true(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_truth_aggregate(head, args, TruthAggregateKind::Any)
    }

    pub(in crate::runtime) fn eval_none_true(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_truth_aggregate(head, args, TruthAggregateKind::None)
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
