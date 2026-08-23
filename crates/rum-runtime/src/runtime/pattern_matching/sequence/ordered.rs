use super::*;

impl Session {
    // This is a multi-argument shape specialization. Keep it out of the common
    // atomic/single-argument matcher layout; at its intended widths, the
    // iterator loop makes one out-of-line call immaterial.
    #[cold]
    #[inline(never)]
    pub(super) fn match_compiled_ordered_args_iter_owned(
        &mut self,
        mut pattern_args: std::vec::IntoIter<MatcherArg>,
        mut value_args: std::vec::IntoIter<Expr>,
        mut bindings: SymbolExprMap,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        loop {
            let Some(pattern_arg) = pattern_args.next() else {
                return Ok(match_outcome(value_args.len() == 0, bindings));
            };
            let MatcherArg::Single(pattern) = pattern_arg else {
                return self.match_compiled_arg_and_rest_owned(
                    pattern_arg,
                    pattern_args.collect(),
                    value_args.collect(),
                    bindings,
                );
            };
            let Some(value) = value_args.next() else {
                return Ok(match_outcome(false, bindings));
            };

            match self.match_compiled_owned(pattern, value, bindings) {
                Ok(outcome) if outcome.matched => bindings = outcome.bindings,
                Ok(outcome) => return Ok(outcome),
                Err(error) => {
                    return self.resume_match_compiled_ordered_args_iter_owned(
                        error,
                        pattern_args,
                        value_args,
                    );
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn resume_match_compiled_ordered_args_iter_owned(
        &mut self,
        error: EvalError,
        pattern_args: std::vec::IntoIter<MatcherArg>,
        value_args: std::vec::IntoIter<Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        self.drive_internal_value::<CompiledMatchOutcome, CompiledMatchOutcome, _>(
            Err(error),
            move |session, outcome| {
                if !outcome.matched {
                    return Ok(outcome);
                }

                session.match_compiled_ordered_args_iter_owned(
                    pattern_args,
                    value_args,
                    outcome.bindings,
                )
            },
        )
    }

    #[cold]
    #[inline(never)]
    pub(super) fn collect_compiled_ordered_args_iter_owned(
        &mut self,
        mut pattern_args: std::vec::IntoIter<MatcherArg>,
        mut value_args: std::vec::IntoIter<Expr>,
        mut bindings: SymbolExprMap,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        loop {
            let Some(pattern_arg) = pattern_args.next() else {
                return Ok(if value_args.len() == 0 {
                    vec![bindings]
                } else {
                    Vec::new()
                });
            };
            let MatcherArg::Single(pattern) = pattern_arg else {
                return self.collect_compiled_arg_and_rest_matches_owned(
                    pattern_arg,
                    pattern_args.collect(),
                    value_args.collect(),
                    bindings,
                );
            };
            let Some(value) = value_args.next() else {
                return Ok(Vec::new());
            };

            match self.match_compiled_owned(pattern, value, bindings) {
                Ok(outcome) if outcome.matched => bindings = outcome.bindings,
                Ok(_) => return Ok(Vec::new()),
                Err(error) => {
                    return self.resume_collect_compiled_ordered_args_iter_owned(
                        error,
                        pattern_args,
                        value_args,
                    );
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn resume_collect_compiled_ordered_args_iter_owned(
        &mut self,
        error: EvalError,
        pattern_args: std::vec::IntoIter<MatcherArg>,
        value_args: std::vec::IntoIter<Expr>,
    ) -> Result<Vec<SymbolExprMap>, EvalError> {
        self.drive_internal_value::<CompiledMatchOutcome, Vec<SymbolExprMap>, _>(
            Err(error),
            move |session, outcome| {
                if !outcome.matched {
                    return Ok(Vec::new());
                }

                session.collect_compiled_ordered_args_iter_owned(
                    pattern_args,
                    value_args,
                    outcome.bindings,
                )
            },
        )
    }
}
