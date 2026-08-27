use super::super::matcher::{
    best_pattern_match_at_with_bindings, best_pattern_match_state, linear_literal_match_ranges,
    pattern_match_text,
};
use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct StringReplaceRuleEvalState {
    chars: Vec<char>,
    rules: Vec<ExactStringReplacementRule>,
    max_replacements: usize,
    config: StringPatternMatchConfig,
    literal_matches: Option<Vec<(usize, usize)>>,
    literal_match_index: usize,
}

#[derive(FrameRootTrace)]
struct StringReplaceValuesEvalState {
    values: Vec<String>,
    rules: Vec<ExactStringReplacementRule>,
    max_replacements: usize,
    config: StringPatternMatchConfig,
}

#[derive(FrameRootTrace)]
struct StringCasesRuleEvalState {
    chars: Vec<char>,
    spec: ExactStringCaseSpec,
    max_matches: usize,
    config: StringCasesConfig,
    literal_matches: Option<Vec<(usize, usize)>>,
    literal_match_index: usize,
}

#[derive(FrameRootTrace)]
struct StringCasesValuesEvalState {
    values: Vec<String>,
    spec: ExactStringCaseSpec,
    max_matches: usize,
    config: StringCasesConfig,
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

    let chars = value.chars().collect::<Vec<_>>();
    let literal_matches = match rules {
        [rule] => {
            linear_literal_match_ranges(&chars, &rule.pattern, config, false, max_replacements)
        }
        _ => None,
    };

    resume_replace_string_value_from_index(
        session,
        StringReplaceRuleEvalState {
            chars,
            rules: rules.to_vec(),
            max_replacements,
            config,
            literal_matches,
            literal_match_index: 0,
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
    config: StringCasesConfig,
) -> Result<Vec<Expr>, EvalError> {
    if max_matches == 0 {
        return Ok(Vec::new());
    }

    let chars = value.chars().collect::<Vec<_>>();
    let literal_matches = match spec {
        ExactStringCaseSpec::Rules(rules) => match rules.as_slice() {
            [rule] => linear_literal_match_ranges(
                &chars,
                &rule.pattern,
                config.match_config,
                matches!(config.overlaps, StringSearchOverlaps::Overlapping),
                max_matches,
            ),
            _ => None,
        },
        ExactStringCaseSpec::Patterns(_) => None,
    };

    resume_collect_string_cases_from_index(
        session,
        StringCasesRuleEvalState {
            chars,
            spec: spec.clone(),
            max_matches,
            config,
            literal_matches,
            literal_match_index: 0,
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
    config: StringCasesConfig,
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
    if state.literal_matches.is_some() {
        return resume_replace_literal_string_from_index(
            session,
            state,
            index,
            replacements,
            output,
        );
    }

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
            let rendered_result =
                evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);
            return drive_replace_string_output_result(
                session,
                state,
                index,
                end,
                replacements,
                output,
                rendered_result,
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

fn resume_replace_literal_string_from_index(
    session: &mut Session,
    mut state: StringReplaceRuleEvalState,
    index: usize,
    replacements: usize,
    mut output: String,
) -> Result<String, EvalError> {
    if replacements >= state.max_replacements {
        output.extend(state.chars[index..].iter());
        return Ok(output);
    }

    let next_match = state
        .literal_matches
        .as_ref()
        .and_then(|matches| matches.get(state.literal_match_index))
        .copied();
    let Some((start, end)) = next_match else {
        output.extend(state.chars[index..].iter());
        return Ok(output);
    };
    debug_assert!(start >= index);
    output.extend(state.chars[index..start].iter());
    state.literal_match_index += 1;

    let rule = state
        .rules
        .first()
        .expect("literal replacement matching requires one rule");
    let output_expr = match &rule.replacement {
        ExactStringReplacementValue::Immediate(expr)
        | ExactStringReplacementValue::Delayed(expr) => expr.clone(),
    };
    let rendered_result =
        evaluate_string_rule_output_expr_result(session, &output_expr, &SymbolExprMap::default());
    drive_replace_string_output_result(
        session,
        state,
        start,
        end,
        replacements,
        output,
        rendered_result,
    )
}

fn drive_replace_string_output_result(
    session: &mut Session,
    state: StringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    output: String,
    rendered_result: Result<Expr, EvalError>,
) -> Result<String, EvalError> {
    match rendered_result {
        Ok(rendered_expr) => resume_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            output,
            rendered_expr,
        ),
        Err(error @ EvalError::InputRequired { .. }) => {
            suspend_replace_string_output(session, error, state, index, end, replacements, output)
        }
        Err(error) => Err(error),
    }
}

#[cold]
#[inline(never)]
fn suspend_replace_string_output(
    session: &mut Session,
    error: EvalError,
    state: StringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    output: String,
) -> Result<String, EvalError> {
    let literal_match_index = state.literal_match_index;
    let state = FrozenStringValue::freeze(state);
    let output = PersistentStringText::freeze(output);
    session.drive_evaluated_value::<Expr, String, _>(Err(error), move |session, rendered_expr| {
        resume_suspended_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            literal_match_index,
            output,
            rendered_expr,
        )
    })
}

fn resume_suspended_replace_string_value_from_index(
    session: &mut Session,
    state: FrozenStringValue<StringReplaceRuleEvalState>,
    mut index: usize,
    replacements: usize,
    mut literal_match_index: usize,
    mut output: PersistentStringText,
) -> Result<String, EvalError> {
    let fixed = state.as_ref();
    if fixed.literal_matches.is_some() {
        if replacements >= fixed.max_replacements {
            return Ok(output
                .appended(fixed.chars[index..].iter().collect::<String>())
                .to_string());
        }

        let next_match = fixed
            .literal_matches
            .as_ref()
            .and_then(|matches| matches.get(literal_match_index))
            .copied();
        let Some((start, end)) = next_match else {
            return Ok(output
                .appended(fixed.chars[index..].iter().collect::<String>())
                .to_string());
        };
        debug_assert!(start >= index);
        output = output.appended(fixed.chars[index..start].iter().collect::<String>());
        literal_match_index += 1;

        let rule = fixed
            .rules
            .first()
            .expect("literal replacement matching requires one rule");
        let output_expr = match &rule.replacement {
            ExactStringReplacementValue::Immediate(expr)
            | ExactStringReplacementValue::Delayed(expr) => expr.clone(),
        };
        let rendered_result = evaluate_string_rule_output_expr_result(
            session,
            &output_expr,
            &SymbolExprMap::default(),
        );
        return drive_suspended_replace_string_output_result(
            session,
            state,
            start,
            end,
            replacements,
            literal_match_index,
            output,
            rendered_result,
        );
    }

    while index <= fixed.chars.len() {
        if replacements >= fixed.max_replacements {
            return Ok(output
                .appended(fixed.chars[index..].iter().collect::<String>())
                .to_string());
        }

        if let Some((output_expr, end, bindings)) = fixed.rules.iter().find_map(|rule| {
            best_pattern_match_state(&rule.pattern, &fixed.chars, index, fixed.config).map(
                |match_state| {
                    let output_expr = match &rule.replacement {
                        ExactStringReplacementValue::Immediate(expr)
                        | ExactStringReplacementValue::Delayed(expr) => expr.clone(),
                    };
                    (output_expr, match_state.end, match_state.bindings)
                },
            )
        }) {
            let rendered_result =
                evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);
            return drive_suspended_replace_string_output_result(
                session,
                state,
                index,
                end,
                replacements,
                literal_match_index,
                output,
                rendered_result,
            );
        }

        if index == fixed.chars.len() {
            break;
        }
        output = output.appended(fixed.chars[index].to_string());
        index += 1;
    }

    Ok(output.to_string())
}

#[allow(clippy::too_many_arguments)]
fn drive_suspended_replace_string_output_result(
    session: &mut Session,
    state: FrozenStringValue<StringReplaceRuleEvalState>,
    index: usize,
    end: usize,
    replacements: usize,
    literal_match_index: usize,
    output: PersistentStringText,
    rendered_result: Result<Expr, EvalError>,
) -> Result<String, EvalError> {
    match rendered_result {
        Ok(rendered_expr) => resume_suspended_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            literal_match_index,
            output,
            rendered_expr,
        ),
        Err(EvalError::InputRequired { request }) => session
            .drive_evaluated_value::<Expr, String, _>(
                Err(EvalError::InputRequired { request }),
                move |session, rendered_expr| {
                    resume_suspended_replace_string_value_with_output(
                        session,
                        state,
                        index,
                        end,
                        replacements,
                        literal_match_index,
                        output,
                        rendered_expr,
                    )
                },
            ),
        Err(error) => Err(error),
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_replace_string_value_with_output(
    session: &mut Session,
    state: FrozenStringValue<StringReplaceRuleEvalState>,
    index: usize,
    end: usize,
    replacements: usize,
    literal_match_index: usize,
    output: PersistentStringText,
    rendered_expr: Expr,
) -> Result<String, EvalError> {
    let rendered = expr_as_string(&rendered_expr)
        .map(str::to_owned)
        .unwrap_or_else(|| rendered_expr.to_string());
    let mut output = output.appended(rendered);
    let fixed = state.as_ref();

    let next_index = if end == index {
        if index == fixed.chars.len() {
            return Ok(output.to_string());
        }
        output = output.appended(fixed.chars[index].to_string());
        index + 1
    } else {
        end
    };

    resume_suspended_replace_string_value_from_index(
        session,
        state,
        next_index,
        replacements + 1,
        literal_match_index,
        output,
    )
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

/// Renders one string per loop iteration instead of recursing per string.
/// On suspension the state, the index, and the accumulated output move into
/// the suspension frame by value — the old per-string `state.clone()` plus
/// `rendered.clone()` made a wide list quadratic in its element count.
fn resume_replace_string_values_from_index(
    session: &mut Session,
    state: StringReplaceValuesEvalState,
    mut index: usize,
    mut rendered: Vec<String>,
) -> Result<Expr, EvalError> {
    loop {
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
        match result {
            Ok(value) => {
                rendered.push(value);
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                let state = FrozenStringValue::freeze(state);
                let rendered = PersistentStringSequence::freeze(rendered);
                return session.drive_internal_value::<String, Expr, _>(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        resume_suspended_replace_string_values_with_value(
                            session, state, index, rendered, value,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_replace_string_values_from_index(
    session: &mut Session,
    state: FrozenStringValue<StringReplaceValuesEvalState>,
    mut index: usize,
    mut rendered: PersistentStringSequence<String>,
) -> Result<Expr, EvalError> {
    loop {
        let fixed = state.as_ref();
        let Some(value) = fixed.values.get(index) else {
            return Ok(string_list_expr(rendered.to_vec()));
        };

        let result = replace_string_value_with_rules(
            session,
            value,
            &fixed.rules,
            fixed.max_replacements,
            fixed.config,
        );
        match result {
            Ok(value) => {
                rendered = rendered.appended(value);
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                return session.drive_internal_value::<String, Expr, _>(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        resume_suspended_replace_string_values_with_value(
                            session, state, index, rendered, value,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_replace_string_values_with_value(
    session: &mut Session,
    state: FrozenStringValue<StringReplaceValuesEvalState>,
    index: usize,
    rendered: PersistentStringSequence<String>,
    value: String,
) -> Result<Expr, EvalError> {
    resume_suspended_replace_string_values_from_index(
        session,
        state,
        index + 1,
        rendered.appended(value),
    )
}

fn resume_collect_string_cases_from_index(
    session: &mut Session,
    state: StringCasesRuleEvalState,
    mut index: usize,
    mut matches: Vec<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    if state.literal_matches.is_some() {
        return resume_collect_literal_string_cases(session, state, matches);
    }

    while index <= state.chars.len() && matches.len() < state.max_matches {
        match &state.spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, match_state)) = best_pattern_match_at_with_bindings(
                    patterns,
                    &state.chars,
                    index,
                    state.config.match_config,
                ) {
                    matches.push(Expr::string(pattern_match_text(
                        &state.chars[index..match_state.end],
                    )));
                    let Some(next_index) = next_string_match_index(
                        index,
                        match_state.end,
                        state.chars.len(),
                        state.config.overlaps,
                    ) else {
                        break;
                    };
                    index = next_index;
                } else {
                    if index == state.chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((output_expr, end, bindings)) = rules.iter().find_map(|rule| {
                    best_pattern_match_state(
                        &rule.pattern,
                        &state.chars,
                        index,
                        state.config.match_config,
                    )
                    .map(|match_state| {
                        let output_expr = match &rule.output {
                            ExactStringCaseOutput::Immediate(expr)
                            | ExactStringCaseOutput::Delayed(expr) => expr.clone(),
                        };
                        (output_expr, match_state.end, match_state.bindings)
                    })
                }) {
                    let output_result =
                        evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);
                    return drive_string_cases_output_result(
                        session,
                        state,
                        index,
                        end,
                        matches,
                        output_result,
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

fn resume_collect_literal_string_cases(
    session: &mut Session,
    mut state: StringCasesRuleEvalState,
    matches: Vec<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    if matches.len() >= state.max_matches {
        return Ok(matches);
    }

    let next_match = state
        .literal_matches
        .as_ref()
        .and_then(|ranges| ranges.get(state.literal_match_index))
        .copied();
    let Some((start, end)) = next_match else {
        return Ok(matches);
    };
    state.literal_match_index += 1;

    let ExactStringCaseSpec::Rules(rules) = &state.spec else {
        unreachable!("literal rule matching requires a rule specification");
    };
    let rule = rules
        .first()
        .expect("literal case matching requires one rule");
    let output_expr = match &rule.output {
        ExactStringCaseOutput::Immediate(expr) | ExactStringCaseOutput::Delayed(expr) => {
            expr.clone()
        }
    };
    let output_result =
        evaluate_string_rule_output_expr_result(session, &output_expr, &SymbolExprMap::default());
    drive_string_cases_output_result(session, state, start, end, matches, output_result)
}

fn drive_string_cases_output_result(
    session: &mut Session,
    state: StringCasesRuleEvalState,
    index: usize,
    end: usize,
    matches: Vec<Expr>,
    output_result: Result<Expr, EvalError>,
) -> Result<Vec<Expr>, EvalError> {
    match output_result {
        Ok(output) => {
            resume_collect_string_cases_with_output(session, state, index, end, matches, output)
        }
        Err(EvalError::InputRequired { request }) => {
            let literal_match_index = state.literal_match_index;
            let state = FrozenStringValue::freeze(state);
            let matches = PersistentStringSequence::freeze(matches);
            session.drive_evaluated_value::<Expr, Vec<Expr>, _>(
                Err(EvalError::InputRequired { request }),
                move |session, output| {
                    resume_suspended_collect_string_cases_with_output(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output,
                    )
                },
            )
        }
        Err(error) => Err(error),
    }
}

fn resume_suspended_collect_string_cases_from_index(
    session: &mut Session,
    state: FrozenStringValue<StringCasesRuleEvalState>,
    mut index: usize,
    mut literal_match_index: usize,
    mut matches: PersistentStringSequence<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    let fixed = state.as_ref();
    if fixed.literal_matches.is_some() {
        if matches.len() >= fixed.max_matches {
            return Ok(matches.to_vec());
        }
        let next_match = fixed
            .literal_matches
            .as_ref()
            .and_then(|ranges| ranges.get(literal_match_index))
            .copied();
        let Some((start, end)) = next_match else {
            return Ok(matches.to_vec());
        };
        literal_match_index += 1;

        let ExactStringCaseSpec::Rules(rules) = &fixed.spec else {
            unreachable!("literal rule matching requires a rule specification");
        };
        let rule = rules
            .first()
            .expect("literal case matching requires one rule");
        let output_expr = match &rule.output {
            ExactStringCaseOutput::Immediate(expr) | ExactStringCaseOutput::Delayed(expr) => {
                expr.clone()
            }
        };
        let output_result = evaluate_string_rule_output_expr_result(
            session,
            &output_expr,
            &SymbolExprMap::default(),
        );
        return drive_suspended_string_cases_output_result(
            session,
            state,
            start,
            end,
            literal_match_index,
            matches,
            output_result,
        );
    }

    while index <= fixed.chars.len() && matches.len() < fixed.max_matches {
        match &fixed.spec {
            ExactStringCaseSpec::Patterns(patterns) => {
                if let Some((_pattern_index, match_state)) = best_pattern_match_at_with_bindings(
                    patterns,
                    &fixed.chars,
                    index,
                    fixed.config.match_config,
                ) {
                    matches = matches.appended(Expr::string(pattern_match_text(
                        &fixed.chars[index..match_state.end],
                    )));
                    let Some(next_index) = next_string_match_index(
                        index,
                        match_state.end,
                        fixed.chars.len(),
                        fixed.config.overlaps,
                    ) else {
                        break;
                    };
                    index = next_index;
                } else {
                    if index == fixed.chars.len() {
                        break;
                    }
                    index += 1;
                }
            }
            ExactStringCaseSpec::Rules(rules) => {
                if let Some((output_expr, end, bindings)) = rules.iter().find_map(|rule| {
                    best_pattern_match_state(
                        &rule.pattern,
                        &fixed.chars,
                        index,
                        fixed.config.match_config,
                    )
                    .map(|match_state| {
                        let output_expr = match &rule.output {
                            ExactStringCaseOutput::Immediate(expr)
                            | ExactStringCaseOutput::Delayed(expr) => expr.clone(),
                        };
                        (output_expr, match_state.end, match_state.bindings)
                    })
                }) {
                    let output_result =
                        evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);
                    return drive_suspended_string_cases_output_result(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output_result,
                    );
                }

                if index == fixed.chars.len() {
                    break;
                }
                index += 1;
            }
        }
    }

    Ok(matches.to_vec())
}

#[allow(clippy::too_many_arguments)]
fn drive_suspended_string_cases_output_result(
    session: &mut Session,
    state: FrozenStringValue<StringCasesRuleEvalState>,
    index: usize,
    end: usize,
    literal_match_index: usize,
    matches: PersistentStringSequence<Expr>,
    output_result: Result<Expr, EvalError>,
) -> Result<Vec<Expr>, EvalError> {
    match output_result {
        Ok(output) => resume_suspended_collect_string_cases_with_output(
            session,
            state,
            index,
            end,
            literal_match_index,
            matches,
            output,
        ),
        Err(EvalError::InputRequired { request }) => session
            .drive_evaluated_value::<Expr, Vec<Expr>, _>(
                Err(EvalError::InputRequired { request }),
                move |session, output| {
                    resume_suspended_collect_string_cases_with_output(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output,
                    )
                },
            ),
        Err(error) => Err(error),
    }
}

fn resume_suspended_collect_string_cases_with_output(
    session: &mut Session,
    state: FrozenStringValue<StringCasesRuleEvalState>,
    index: usize,
    end: usize,
    literal_match_index: usize,
    matches: PersistentStringSequence<Expr>,
    output: Expr,
) -> Result<Vec<Expr>, EvalError> {
    let matches = matches.appended(output);
    let Some(next_index) = next_string_match_index(
        index,
        end,
        state.as_ref().chars.len(),
        state.as_ref().config.overlaps,
    ) else {
        return Ok(matches.to_vec());
    };

    resume_suspended_collect_string_cases_from_index(
        session,
        state,
        next_index,
        literal_match_index,
        matches,
    )
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
    let Some(next_index) =
        next_string_match_index(index, end, state.chars.len(), state.config.overlaps)
    else {
        return Ok(matches);
    };

    resume_collect_string_cases_from_index(session, state, next_index, matches)
}

/// Collects one string per loop iteration instead of recursing per string.
/// On suspension the state, the index, and the accumulated output move into
/// the suspension frame by value — the old per-string `state.clone()` plus
/// `outputs.clone()` made a wide list quadratic in its element count.
fn resume_collect_string_cases_values_from_index(
    session: &mut Session,
    state: StringCasesValuesEvalState,
    mut index: usize,
    mut outputs: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
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
        match result {
            Ok(items) => {
                outputs.push(list_expr(items));
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                let state = FrozenStringValue::freeze(state);
                let outputs = PersistentStringSequence::freeze(outputs);
                return session.drive_internal_value::<Vec<Expr>, Expr, _>(
                    Err(EvalError::InputRequired { request }),
                    move |session, items| {
                        resume_suspended_collect_string_cases_values_with_value(
                            session, state, index, outputs, items,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_collect_string_cases_values_from_index(
    session: &mut Session,
    state: FrozenStringValue<StringCasesValuesEvalState>,
    mut index: usize,
    mut outputs: PersistentStringSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let fixed = state.as_ref();
        let Some(value) = fixed.values.get(index) else {
            return Ok(list_expr(outputs.to_vec()));
        };

        let result = collect_string_cases_with_rule_evaluation(
            session,
            value,
            &fixed.spec,
            fixed.max_matches,
            fixed.config,
        );
        match result {
            Ok(items) => {
                outputs = outputs.appended(list_expr(items));
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                return session.drive_internal_value::<Vec<Expr>, Expr, _>(
                    Err(EvalError::InputRequired { request }),
                    move |session, items| {
                        resume_suspended_collect_string_cases_values_with_value(
                            session, state, index, outputs, items,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_collect_string_cases_values_with_value(
    session: &mut Session,
    state: FrozenStringValue<StringCasesValuesEvalState>,
    index: usize,
    outputs: PersistentStringSequence<Expr>,
    items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    resume_suspended_collect_string_cases_values_from_index(
        session,
        state,
        index + 1,
        outputs.appended(list_expr(items)),
    )
}

pub(super) fn evaluate_string_rule_output_expr_result(
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
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<StringCasesRuleEvalState>();
    assert_traces::<StringCasesValuesEvalState>();
    assert_traces::<StringReplaceRuleEvalState>();
    assert_traces::<StringReplaceValuesEvalState>();
};
