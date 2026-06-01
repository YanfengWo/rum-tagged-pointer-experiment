use super::candidates::orderless_candidate_choices;
use super::*;

impl Session {
    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_arg_choices_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.match_compiled_orderless_optional_choices(
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, false);
                self.match_compiled_orderless_choices_owned(
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn match_compiled_orderless_flat_arg_choices_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<CompiledMatchOutcome, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.match_compiled_orderless_flat_optional_choices(
                flat_head,
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, true);
                self.match_compiled_orderless_flat_choices_owned(
                    flat_head,
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_arg_choice_matches_owned(
        &mut self,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self.collect_compiled_orderless_optional_matches_owned(
                spec,
                rest_patterns,
                value_args,
                bindings,
            ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, false);
                self.collect_compiled_orderless_choices_matches_owned(
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                    Vec::new(),
                )
            }
        }
    }

    pub(in crate::runtime::pattern_matching::sequence) fn collect_compiled_orderless_flat_arg_choice_matches_owned(
        &mut self,
        flat_head: Expr,
        pattern: MatcherArg,
        rest_patterns: Vec<MatcherArg>,
        value_args: Vec<Expr>,
        bindings: BTreeMap<Symbol, Expr>,
    ) -> Result<Vec<BTreeMap<Symbol, Expr>>, EvalError> {
        match pattern {
            MatcherArg::Optional(spec) => self
                .collect_compiled_orderless_flat_optional_matches_owned(
                    flat_head,
                    spec,
                    rest_patterns,
                    value_args,
                    bindings,
                ),
            other => {
                let choices =
                    orderless_candidate_choices(&other, &rest_patterns, &value_args, true);
                self.collect_compiled_orderless_flat_choices_matches_owned(
                    flat_head,
                    other,
                    rest_patterns,
                    choices,
                    bindings,
                    0,
                    Vec::new(),
                )
            }
        }
    }
}
