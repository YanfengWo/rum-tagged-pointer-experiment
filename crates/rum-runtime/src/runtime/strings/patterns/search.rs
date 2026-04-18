use super::matcher::{
    best_pattern_match_at_with_bindings, best_pattern_match_state, collect_string_cases,
    pattern_match_text, split_string_value, string_contains_q, string_count_matches,
    string_delete_matches, string_ends_q, string_free_q, string_match_q, string_position_expr,
    string_starts_q, trim_string_value,
};
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_string_contains_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_string_pattern_predicate(head, args, "StringContainsQ", string_contains_q)
    }

    pub(in crate::runtime) fn eval_string_starts_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_string_pattern_predicate(head, args, "StringStartsQ", string_starts_q)
    }

    pub(in crate::runtime) fn eval_string_ends_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_string_pattern_predicate(head, args, "StringEndsQ", string_ends_q)
    }

    pub(in crate::runtime) fn eval_string_position(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringPosition", args, 2, 4));
        }

        let Some(pattern) = parse_string_pattern(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let option_exprs =
            self.merged_option_exprs_for_system_symbol("StringPosition", &args[2..])?;
        let Some((max_results, config)) = parse_string_position_args(&option_exprs) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(string_position_expr(value, &pattern, max_results, config));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| string_position_expr(&value, &pattern, max_results, config))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_split(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringSplit", args, 1, 3));
        }

        let delimiter_spec = match args.get(1) {
            Some(expr) => parse_string_boundary_spec(expr),
            None => Some(StringBoundarySpec::Whitespace),
        };
        let Some(delimiter_spec) = delimiter_spec else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let limit = match args.get(2) {
            Some(expr) => parse_string_split_limit(expr),
            None => Some(StringSplitLimit::Unlimited),
        };
        let Some(limit) = limit else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(string_list_expr(split_string_value(
                value,
                &delimiter_spec,
                limit,
            )));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        string_list_expr(split_string_value(&value, &delimiter_spec, limit))
                    })
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_trim(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringTrim", args, 1, 2));
        }

        let trim_spec = match args.get(1) {
            Some(expr) => parse_string_boundary_spec(expr),
            None => Some(StringBoundarySpec::Whitespace),
        };
        let Some(trim_spec) = trim_spec else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(trim_string_value(value, &trim_spec)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values
                    .into_iter()
                    .map(|value| trim_string_value(&value, &trim_spec))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_replace(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringReplace", args, 2, 4));
        }

        let Some(rules) = parse_exact_string_replacement_rules(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let option_exprs =
            self.merged_option_exprs_for_system_symbol("StringReplace", &args[2..])?;
        let Some((max_replacements, config)) = parse_string_match_limit_and_options(&option_exprs)
        else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(self.replace_string_value_with_rules(
                value,
                &rules,
                max_replacements,
                config,
            )?));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            let mut rendered = Vec::with_capacity(values.len());
            for value in values {
                rendered.push(self.replace_string_value_with_rules(
                    &value,
                    &rules,
                    max_replacements,
                    config,
                )?);
            }
            return Ok(string_list_expr(rendered));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_cases(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("StringCases", args, 2, 4));
        }

        let Some(spec) = parse_exact_string_case_spec(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let option_exprs = self.merged_option_exprs_for_system_symbol("StringCases", &args[2..])?;
        let Some((max_matches, config)) = parse_string_match_limit_and_options(&option_exprs)
        else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(if matches!(&spec, ExactStringCaseSpec::Rules(_)) {
                list_expr(self.collect_string_cases_with_rule_evaluation(
                    value,
                    &spec,
                    max_matches,
                    config,
                )?)
            } else {
                list_expr(collect_string_cases(value, &spec, max_matches, config))
            });
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            let mut outputs = Vec::with_capacity(values.len());
            for value in values {
                let items = if matches!(&spec, ExactStringCaseSpec::Rules(_)) {
                    self.collect_string_cases_with_rule_evaluation(
                        &value,
                        &spec,
                        max_matches,
                        config,
                    )?
                } else {
                    collect_string_cases(&value, &spec, max_matches, config)
                };
                outputs.push(list_expr(items));
            }
            return Ok(list_expr(outputs));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_count(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_arg_range("StringCount", args, 2, 4));
        }

        let Some(patterns) = parse_string_patterns(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let option_exprs = self.merged_option_exprs_for_system_symbol("StringCount", &args[2..])?;
        let Some(config) = parse_string_count_options(&option_exprs) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::integer(BigInt::from(
                string_count_matches(value, &patterns, config) as u64,
            )));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        Expr::integer(BigInt::from(
                            string_count_matches(&value, &patterns, config) as u64,
                        ))
                    })
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_delete(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() == 1 {
            return Ok(string_unary_operator_form_slot_first(
                "StringDelete",
                &args[0],
            ));
        }
        if args.len() < 2 {
            return Ok(self.invalid_call_with_arg_range("StringDelete", args, 1, 3));
        }

        let Some(patterns) = parse_string_patterns(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let option_exprs =
            self.merged_option_exprs_for_system_symbol("StringDelete", &args[2..])?;
        let Some(config) = parse_string_delete_options(&option_exprs) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(string_delete_matches(
                value, &patterns, config,
            )));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values
                    .into_iter()
                    .map(|value| string_delete_matches(&value, &patterns, config))
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(in crate::runtime) fn eval_string_apply(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(string_unary_operator_form("StringApply", operator)),
            [operator, input] => {
                if let Some(value) = expr_as_string(input) {
                    return string_apply_value(
                        self,
                        operator,
                        value,
                        Expr::call(head.clone(), args.to_vec()),
                    );
                }

                if let Some(values) = expr_as_string_list(input) {
                    let mut outputs = Vec::with_capacity(values.len());
                    for value in values {
                        outputs.push(string_apply_value(
                            self,
                            operator,
                            &value,
                            Expr::call(
                                head.clone(),
                                vec![operator.clone(), Expr::string(value.clone())],
                            ),
                        )?);
                    }
                    return Ok(list_expr(outputs));
                }

                Ok(Expr::call(head.clone(), args.to_vec()))
            }
            _ => Ok(self.invalid_call_with_arg_range("StringApply", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_string_match_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_string_patterns_predicate(head, args, "StringMatchQ", string_match_q)
    }

    pub(in crate::runtime) fn eval_string_free_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_string_patterns_predicate(head, args, "StringFreeQ", string_free_q)
    }

    fn eval_string_pattern_predicate(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        predicate: fn(&[char], &StringPattern, StringPatternMatchConfig) -> bool,
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range(symbol, args, 2, 3));
        }

        let Some(pattern) = parse_string_pattern(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let option_exprs = self.merged_option_exprs_for_system_symbol(symbol, &args[2..])?;
        let Some(config) = parse_string_match_config(&option_exprs) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            let chars = value.chars().collect::<Vec<_>>();
            return Ok(bool_expr(predicate(&chars, &pattern, config)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        let chars = value.chars().collect::<Vec<_>>();
                        bool_expr(predicate(&chars, &pattern, config))
                    })
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    fn eval_string_patterns_predicate(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        predicate: fn(&[char], &[StringPattern], StringPatternMatchConfig) -> bool,
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range(symbol, args, 2, 3));
        }

        let Some(patterns) = parse_string_patterns(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let option_exprs = self.merged_option_exprs_for_system_symbol(symbol, &args[2..])?;
        let Some(config) = parse_string_match_config(&option_exprs) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if let Some(value) = expr_as_string(&args[0]) {
            let chars = value.chars().collect::<Vec<_>>();
            return Ok(bool_expr(predicate(&chars, &patterns, config)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(list_expr(
                values
                    .into_iter()
                    .map(|value| {
                        let chars = value.chars().collect::<Vec<_>>();
                        bool_expr(predicate(&chars, &patterns, config))
                    })
                    .collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    fn replace_string_value_with_rules(
        &mut self,
        value: &str,
        rules: &[ExactStringReplacementRule],
        max_replacements: usize,
        config: StringPatternMatchConfig,
    ) -> Result<String, EvalError> {
        if max_replacements == 0 {
            return Ok(value.to_string());
        }

        let chars = value.chars().collect::<Vec<_>>();
        let mut output = String::new();
        let mut index = 0usize;
        let mut replacements = 0usize;

        while index < chars.len() {
            if replacements >= max_replacements {
                output.extend(chars[index..].iter());
                break;
            }

            if let Some((rule, state)) = rules.iter().find_map(|rule| {
                best_pattern_match_state(&rule.pattern, &chars, index, config)
                    .map(|state| (rule, state))
            }) {
                let rendered_expr = self.evaluate_string_rule_output_expr(
                    match &rule.replacement {
                        ExactStringReplacementValue::Immediate(expr)
                        | ExactStringReplacementValue::Delayed(expr) => expr,
                    },
                    &state.bindings,
                )?;
                let rendered = expr_as_string(&rendered_expr)
                    .map(str::to_owned)
                    .unwrap_or_else(|| rendered_expr.to_string());

                output.push_str(&rendered);
                if state.end == index {
                    output.push(chars[index]);
                    index += 1;
                } else {
                    index = state.end;
                }
                replacements += 1;
            } else {
                output.push(chars[index]);
                index += 1;
            }
        }

        Ok(output)
    }

    fn collect_string_cases_with_rule_evaluation(
        &mut self,
        value: &str,
        spec: &ExactStringCaseSpec,
        max_matches: usize,
        config: StringPatternMatchConfig,
    ) -> Result<Vec<Expr>, EvalError> {
        if max_matches == 0 {
            return Ok(Vec::new());
        }

        let chars = value.chars().collect::<Vec<_>>();
        let mut index = 0usize;
        let mut matches = Vec::new();

        while index <= chars.len() && matches.len() < max_matches {
            match spec {
                ExactStringCaseSpec::Patterns(patterns) => {
                    if let Some((_pattern_index, state)) =
                        best_pattern_match_at_with_bindings(patterns, &chars, index, config)
                    {
                        matches.push(Expr::string(pattern_match_text(&chars[index..state.end])));
                        index = if state.end == index {
                            if index == chars.len() {
                                break;
                            }
                            index + 1
                        } else {
                            state.end
                        };
                    } else {
                        if index == chars.len() {
                            break;
                        }
                        index += 1;
                    }
                }
                ExactStringCaseSpec::Rules(rules) => {
                    if let Some((rule, state)) = rules.iter().find_map(|rule| {
                        best_pattern_match_state(&rule.pattern, &chars, index, config)
                            .map(|state| (rule, state))
                    }) {
                        let output = self.evaluate_string_rule_output_expr(
                            match &rule.output {
                                ExactStringCaseOutput::Immediate(expr)
                                | ExactStringCaseOutput::Delayed(expr) => expr,
                            },
                            &state.bindings,
                        )?;
                        matches.push(output);
                        index = if state.end == index {
                            if index == chars.len() {
                                break;
                            }
                            index + 1
                        } else {
                            state.end
                        };
                    } else {
                        if index == chars.len() {
                            break;
                        }
                        index += 1;
                    }
                }
            }
        }

        Ok(matches)
    }

    fn evaluate_string_rule_output_expr(
        &mut self,
        expr: &Expr,
        bindings: &BTreeMap<Symbol, Expr>,
    ) -> Result<Expr, EvalError> {
        let substituted = if bindings.is_empty() {
            expr.clone()
        } else {
            substitute_lexical_bindings(expr, bindings, &BTreeSet::new())
        };
        let evaluated = self.evaluate(substituted)?;
        self.suspend_with_frame(Ok(evaluated), |_, value| Ok(value))
    }
}
