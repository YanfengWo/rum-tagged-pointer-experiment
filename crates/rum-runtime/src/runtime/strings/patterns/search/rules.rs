use super::super::matcher::{
    best_pattern_match_at_with_bindings, best_pattern_match_state, pattern_match_text,
};
use super::*;

impl Session {
    pub(super) fn replace_string_value_with_rules(
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

    pub(super) fn collect_string_cases_with_rule_evaluation(
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
        bindings: &SymbolExprMap,
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
