use super::*;

impl Session {
    pub(in crate::runtime) fn match_compiled_args_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_arg_and_rest_owned(first, rest_patterns, value_args, bindings)
    }

    pub(in crate::runtime) fn match_compiled_args_with_flat_head_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        flat_head: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_flat_arg_and_rest_owned(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn match_compiled_orderless_args_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_orderless_arg_choices_owned(first, rest_patterns, value_args, bindings)
    }

    pub(in crate::runtime) fn match_compiled_orderless_args_with_flat_head_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        flat_head: Expr,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        if pattern_args.is_empty() {
            return Ok(match_outcome(value_args.is_empty(), bindings));
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.match_compiled_orderless_flat_arg_choices_owned(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn collect_compiled_args_matches_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        if pattern_args.is_empty() {
            return Ok(if value_args.is_empty() {
                vec![bindings]
            } else {
                Vec::new()
            });
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.collect_compiled_arg_and_rest_matches_owned(first, rest_patterns, value_args, bindings)
    }

    pub(in crate::runtime) fn collect_compiled_args_matches_with_flat_head_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        flat_head: Expr,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        if pattern_args.is_empty() {
            return Ok(if value_args.is_empty() {
                vec![bindings]
            } else {
                Vec::new()
            });
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.collect_compiled_flat_arg_and_rest_matches_owned(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn collect_compiled_orderless_arg_matches_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        if pattern_args.is_empty() {
            return Ok(if value_args.is_empty() {
                vec![bindings]
            } else {
                Vec::new()
            });
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.collect_compiled_orderless_arg_choice_matches_owned(
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }

    pub(in crate::runtime) fn collect_compiled_orderless_arg_matches_with_flat_head_owned(
        &mut self,
        pattern_args: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
        flat_head: Expr,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        if pattern_args.is_empty() {
            return Ok(if value_args.is_empty() {
                vec![bindings]
            } else {
                Vec::new()
            });
        }

        let mut pattern_iter = pattern_args.into_iter();
        let first = pattern_iter
            .next()
            .expect("non-empty pattern args always have a first item");
        let rest_patterns = pattern_iter.collect::<Vec<_>>();

        self.collect_compiled_orderless_flat_arg_choice_matches_owned(
            flat_head,
            first,
            rest_patterns,
            value_args,
            bindings,
        )
    }
}
