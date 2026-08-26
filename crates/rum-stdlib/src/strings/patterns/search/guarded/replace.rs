use super::super::super::matcher::best_pattern_match_state;
use super::super::rules::evaluate_string_rule_output_expr_result;
use super::*;

struct GuardedStringReplaceRuleEvalState {
    chars: Vec<char>,
    rules: Vec<GuardedExactStringReplacementRule>,
    max_replacements: usize,
    config: StringPatternMatchConfig,
}

struct GuardedStringReplaceValuesEvalState {
    values: Vec<String>,
    rules: Vec<GuardedExactStringReplacementRule>,
    max_replacements: usize,
    config: StringPatternMatchConfig,
}

pub(in crate::strings::patterns::search) fn replace_string_value_with_guarded_rules(
    session: &mut Session,
    value: &str,
    rules: &[GuardedExactStringReplacementRule],
    max_replacements: usize,
    config: StringPatternMatchConfig,
) -> Result<String, EvalError> {
    debug_assert!(rules.iter().any(|rule| rule.guard.is_some()));
    if max_replacements == 0 {
        return Ok(value.to_string());
    }

    resume_replace_string_value_from_candidate(
        session,
        GuardedStringReplaceRuleEvalState {
            chars: value.chars().collect(),
            rules: rules.to_vec(),
            max_replacements,
            config,
        },
        0,
        0,
        0,
        String::new(),
    )
}

pub(in crate::strings::patterns::search) fn replace_string_values_with_guarded_rules_expr(
    session: &mut Session,
    values: Vec<String>,
    rules: &[GuardedExactStringReplacementRule],
    max_replacements: usize,
    config: StringPatternMatchConfig,
) -> Result<Expr, EvalError> {
    debug_assert!(rules.iter().any(|rule| rule.guard.is_some()));
    let capacity = values.len();
    resume_replace_string_values_from_index(
        session,
        GuardedStringReplaceValuesEvalState {
            values,
            rules: rules.to_vec(),
            max_replacements,
            config,
        },
        0,
        Vec::with_capacity(capacity),
    )
}

fn resume_replace_string_value_from_candidate(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    mut index: usize,
    replacements: usize,
    mut rule_index: usize,
    mut output: String,
) -> Result<String, EvalError> {
    loop {
        if replacements >= state.max_replacements {
            output.extend(state.chars[index..].iter());
            return Ok(output);
        }

        let Some(rule) = state.rules.get(rule_index) else {
            if index == state.chars.len() {
                return Ok(output);
            }
            output.push(state.chars[index]);
            index += 1;
            rule_index = 0;
            continue;
        };

        let Some(match_state) =
            best_pattern_match_state(&rule.rule.pattern, &state.chars, index, state.config)
        else {
            rule_index += 1;
            continue;
        };
        let end = match_state.end;
        let bindings = match_state.bindings;

        if let Some(guard) = &rule.guard {
            let guard_result = evaluate_string_rule_guard_result(
                session,
                guard,
                &state.chars,
                index,
                end,
                &bindings,
            );
            return session.drive_internal_value::<bool, String, _>(
                guard_result,
                move |session, accepted| {
                    resume_replace_string_value_after_guard(
                        session,
                        state,
                        index,
                        end,
                        replacements,
                        rule_index,
                        output,
                        bindings,
                        accepted,
                    )
                },
            );
        }

        return evaluate_replace_string_output(
            session,
            state,
            index,
            end,
            replacements,
            output,
            bindings,
            rule_index,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_replace_string_value_after_guard(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    rule_index: usize,
    output: String,
    bindings: SymbolExprMap,
    accepted: bool,
) -> Result<String, EvalError> {
    if !accepted {
        return resume_replace_string_value_from_candidate(
            session,
            state,
            index,
            replacements,
            rule_index + 1,
            output,
        );
    }

    evaluate_replace_string_output(
        session,
        state,
        index,
        end,
        replacements,
        output,
        bindings,
        rule_index,
    )
}

#[allow(clippy::too_many_arguments)]
fn evaluate_replace_string_output(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
    index: usize,
    end: usize,
    replacements: usize,
    output: String,
    bindings: SymbolExprMap,
    rule_index: usize,
) -> Result<String, EvalError> {
    let rule = &state.rules[rule_index].rule;
    let output_expr = match &rule.replacement {
        ExactStringReplacementValue::Immediate(expr)
        | ExactStringReplacementValue::Delayed(expr) => expr.clone(),
    };
    let output_result = evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);

    match output_result {
        Ok(rendered_expr) => resume_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            output,
            rendered_expr,
        ),
        Err(error) => session.drive_evaluated_value::<Expr, String, _>(
            Err(error),
            move |session, rendered_expr| {
                resume_replace_string_value_with_output(
                    session,
                    state,
                    index,
                    end,
                    replacements,
                    output,
                    rendered_expr,
                )
            },
        ),
    }
}

fn resume_replace_string_value_with_output(
    session: &mut Session,
    state: GuardedStringReplaceRuleEvalState,
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

    resume_replace_string_value_from_candidate(
        session,
        state,
        next_index,
        replacements + 1,
        0,
        output,
    )
}

fn resume_replace_string_values_from_index(
    session: &mut Session,
    state: GuardedStringReplaceValuesEvalState,
    mut index: usize,
    mut rendered: Vec<String>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(value) = state.values.get(index) else {
            return Ok(string_list_expr(rendered));
        };

        let result = replace_string_value_with_guarded_rules(
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
                return session.drive_internal_value::<String, Expr, _>(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        resume_replace_string_values_with_value(
                            session, state, index, rendered, value,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_replace_string_values_with_value(
    session: &mut Session,
    state: GuardedStringReplaceValuesEvalState,
    index: usize,
    mut rendered: Vec<String>,
    value: String,
) -> Result<Expr, EvalError> {
    rendered.push(value);
    resume_replace_string_values_from_index(session, state, index + 1, rendered)
}
