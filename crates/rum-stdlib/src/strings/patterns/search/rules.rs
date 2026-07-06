use super::super::matcher::{
    best_pattern_match_at_with_bindings, best_pattern_match_state, pattern_match_text,
};
use super::*;

#[derive(Clone)]
struct StringReplaceRuleEvalState {
    chars: Vec<char>,
    rules: Vec<ExactStringReplacementRule>,
    max_replacements: usize,
    config: StringPatternMatchConfig,
}

#[derive(Clone)]
struct StringReplaceValuesEvalState {
    values: Vec<String>,
    rules: Vec<ExactStringReplacementRule>,
    max_replacements: usize,
    config: StringPatternMatchConfig,
}

#[derive(Clone)]
struct StringCasesRuleEvalState {
    chars: Vec<char>,
    spec: ExactStringCaseSpec,
    max_matches: usize,
    config: StringPatternMatchConfig,
}

#[derive(Clone)]
struct StringCasesValuesEvalState {
    values: Vec<String>,
    spec: ExactStringCaseSpec,
    max_matches: usize,
    config: StringPatternMatchConfig,
}

pub(super) fn replace_string_value_with_rules(
    session: &mut Session,
    value: &str,
    rules: &[ExactStringReplacementRule],
    max_replacements: usize,
    config: StringPatternMatchConfig,
) -> Result<String, EvalError> {
    if max_replacements == 0 {
        return Ok(value.to_string());
    }

    resume_replace_string_value_from_index(
        session,
        StringReplaceRuleEvalState {
            chars: value.chars().collect(),
            rules: rules.to_vec(),
            max_replacements,
            config,
        },
        0,
        0,
        String::new(),
    )
}

pub(super) fn replace_string_values_with_rules_expr(
    session: &mut Session,
    values: Vec<String>,
    rules: &[ExactStringReplacementRule],
    max_replacements: usize,
    config: StringPatternMatchConfig,
) -> Result<Expr, EvalError> {
    let capacity = values.len();
    resume_replace_string_values_from_index(
        session,
        StringReplaceValuesEvalState {
            values,
            rules: rules.to_vec(),
            max_replacements,
            config,
        },
        0,
        Vec::with_capacity(capacity),
    )
}

pub(super) fn collect_string_cases_with_rule_evaluation(
    session: &mut Session,
    value: &str,
    spec: &ExactStringCaseSpec,
    max_matches: usize,
    config: StringPatternMatchConfig,
) -> Result<Vec<Expr>, EvalError> {
    if max_matches == 0 {
        return Ok(Vec::new());
    }

    resume_collect_string_cases_from_index(
        session,
        StringCasesRuleEvalState {
            chars: value.chars().collect(),
            spec: spec.clone(),
            max_matches,
            config,
        },
        0,
        Vec::new(),
    )
}

pub(super) fn collect_string_cases_values_with_rule_evaluation_expr(
    session: &mut Session,
    values: Vec<String>,
    spec: &ExactStringCaseSpec,
    max_matches: usize,
    config: StringPatternMatchConfig,
) -> Result<Expr, EvalError> {
    let capacity = values.len();
    resume_collect_string_cases_values_from_index(
        session,
        StringCasesValuesEvalState {
            values,
            spec: spec.clone(),
            max_matches,
            config,
        },
        0,
        Vec::with_capacity(capacity),
    )
}

fn resume_replace_string_value_from_index(
    session: &mut Session,
    state: StringReplaceRuleEvalState,
    mut index: usize,
    replacements: usize,
    mut output: String,
) -> Result<String, EvalError> {
    while index <= state.chars.len() {
        if replacements >= state.max_replacements {
            output.extend(state.chars[index..].iter());
            return Ok(output);
        }

        if let Some((output_expr, end, bindings)) = state.rules.iter().find_map(|rule| {
            best_pattern_match_state(&rule.pattern, &state.chars, index, state.config).map(
                |match_state| {
                    let output_expr = match &rule.replacement {
                        ExactStringReplacementValue::Immediate(expr)
                        | ExactStringReplacementValue::Delayed(expr) => expr.clone(),
                    };
                    (output_expr, match_state.end, match_state.bindings)
                },
            )
        }) {
            let suspended_state = state.clone();
            let suspended_output = output.clone();
            let rendered_result =
                evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);
            return session.drive_evaluated_value::<Expr, String, _>(
                rendered_result,
                move |session, rendered_expr| {
                    resume_replace_string_value_with_output(
                        session,
                        suspended_state,
                        index,
                        end,
                        replacements,
                        suspended_output,
                        rendered_expr,
                    )
                },
            );
        }

        if index == state.chars.len() {
            break;
        }
        output.push(state.chars[index]);
        index += 1;
    }

    Ok(output)
}

fn resume_replace_string_value_with_output(
    session: &mut Session,
    state: StringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    mut output: String,
    rendered_expr: Expr,
) -> Result<String, EvalError> {
    let rendered = expr_as_string(&rendered_expr)
        .map(str::to_owned)
        .unwrap_or_else(|| rendered_expr.to_string());
    output.push_str(&rendered);

    let next_index = if end == index {
        if index == state.chars.len() {
            return Ok(output);
        }
        output.push(state.chars[index]);
        index + 1
    } else {
        end
    };

    resume_replace_string_value_from_index(session, state, next_index, replacements + 1, output)
}

fn resume_replace_string_values_from_index(
    session: &mut Session,
    state: StringReplaceValuesEvalState,
    index: usize,
    rendered: Vec<String>,
) -> Result<Expr, EvalError> {
    let Some(value) = state.values.get(index) else {
        return Ok(string_list_expr(rendered));
    };

    let result = replace_string_value_with_rules(
        session,
        value,
        &state.rules,
        state.max_replacements,
        state.config,
    );
    let suspended_state = state.clone();
    let suspended_rendered = rendered.clone();
    session.drive_internal_value::<String, Expr, _>(result, move |session, value| {
        resume_replace_string_values_with_value(
            session,
            suspended_state,
            index,
            suspended_rendered,
            value,
        )
    })
}

fn resume_replace_string_values_with_value(
    session: &mut Session,
    state: StringReplaceValuesEvalState,
    index: usize,
    mut rendered: Vec<String>,
    value: String,
) -> Result<Expr, EvalError> {
    rendered.push(value);
    resume_replace_string_values_from_index(session, state, index + 1, rendered)
}

fn resume_collect_string_cases_from_index(
    session: &mut Session,
    state: StringCasesRuleEvalState,
    mut index: usize,
    mut matches: Vec<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    while index <= state.chars.len() && matches.len() < state.max_matches {
        match &state.spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, match_state)) =
                    best_pattern_match_at_with_bindings(patterns, &state.chars, index, state.config)
                {
                    matches.push(Expr::string(pattern_match_text(
                        &state.chars[index..match_state.end],
                    )));
                    index = if match_state.end == index {
                        if index == state.chars.len() {
                            break;
                        }
                        index + 1
                    } else {
                        match_state.end
                    };
                } else {
                    if index == state.chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((output_expr, end, bindings)) = rules.iter().find_map(|rule| {
                    best_pattern_match_state(&rule.pattern, &state.chars, index, state.config).map(
                        |match_state| {
                            let output_expr = match &rule.output {
                                ExactStringCaseOutput::Immediate(expr)
                                | ExactStringCaseOutput::Delayed(expr) => expr.clone(),
                            };
                            (output_expr, match_state.end, match_state.bindings)
                        },
                    )
                }) {
                    let suspended_state = state.clone();
                    let suspended_matches = matches.clone();
                    let output_result =
                        evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);
                    return session.drive_evaluated_value::<Expr, Vec<Expr>, _>(
                        output_result,
                        move |session, output| {
                            resume_collect_string_cases_with_output(
                                session,
                                suspended_state,
                                index,
                                end,
                                suspended_matches,
                                output,
                            )
                        },
                    );
                }

                if index == state.chars.len() {
                    break;
                }
                index += 1;
            }
        }
    }

    Ok(matches)
}

fn resume_collect_string_cases_with_output(
    session: &mut Session,
    state: StringCasesRuleEvalState,
    index: usize,
    end: usize,
    mut matches: Vec<Expr>,
    output: Expr,
) -> Result<Vec<Expr>, EvalError> {
    matches.push(output);
    let next_index = if end == index {
        if index == state.chars.len() {
            return Ok(matches);
        }
        index + 1
    } else {
        end
    };

    resume_collect_string_cases_from_index(session, state, next_index, matches)
}

fn resume_collect_string_cases_values_from_index(
    session: &mut Session,
    state: StringCasesValuesEvalState,
    index: usize,
    outputs: Vec<Expr>,
) -> Result<Expr, EvalError> {
    let Some(value) = state.values.get(index) else {
        return Ok(list_expr(outputs));
    };

    let result = collect_string_cases_with_rule_evaluation(
        session,
        value,
        &state.spec,
        state.max_matches,
        state.config,
    );
    let suspended_state = state.clone();
    let suspended_outputs = outputs.clone();
    session.drive_internal_value::<Vec<Expr>, Expr, _>(result, move |session, items| {
        resume_collect_string_cases_values_with_value(
            session,
            suspended_state,
            index,
            suspended_outputs,
            items,
        )
    })
}

fn resume_collect_string_cases_values_with_value(
    session: &mut Session,
    state: StringCasesValuesEvalState,
    index: usize,
    mut outputs: Vec<Expr>,
    items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    outputs.push(list_expr(items));
    resume_collect_string_cases_values_from_index(session, state, index + 1, outputs)
}

fn evaluate_string_rule_output_expr_result(
    session: &mut Session,
    expr: &Expr,
    bindings: &SymbolExprMap,
) -> Result<Expr, EvalError> {
    let substituted = if bindings.is_empty() {
        expr.clone()
    } else {
        substitute_lexical_bindings(expr, bindings, &BTreeSet::new())
    };
    session.evaluate(substituted)
}
