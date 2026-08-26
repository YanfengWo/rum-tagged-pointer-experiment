use super::super::super::matcher::best_pattern_match_state;
use super::super::rules::evaluate_string_rule_output_expr_result;
use super::*;

struct GuardedStringCasesRuleEvalState {
    chars: Vec<char>,
    rules: Vec<GuardedExactStringCaseRule>,
    max_matches: usize,
    config: StringCasesConfig,
}

struct GuardedStringCasesValuesEvalState {
    values: Vec<String>,
    rules: Vec<GuardedExactStringCaseRule>,
    max_matches: usize,
    config: StringCasesConfig,
}

pub(in crate::strings::patterns::search) fn collect_string_cases_with_guarded_rules(
    session: &mut Session,
    value: &str,
    rules: &[GuardedExactStringCaseRule],
    max_matches: usize,
    config: StringCasesConfig,
) -> Result<Vec<Expr>, EvalError> {
    debug_assert!(rules.iter().any(|rule| rule.guard.is_some()));
    if max_matches == 0 {
        return Ok(Vec::new());
    }

    resume_collect_string_cases_from_candidate(
        session,
        GuardedStringCasesRuleEvalState {
            chars: value.chars().collect(),
            rules: rules.to_vec(),
            max_matches,
            config,
        },
        0,
        0,
        Vec::new(),
    )
}

pub(in crate::strings::patterns::search) fn collect_string_cases_values_with_guarded_rules_expr(
    session: &mut Session,
    values: Vec<String>,
    rules: &[GuardedExactStringCaseRule],
    max_matches: usize,
    config: StringCasesConfig,
) -> Result<Expr, EvalError> {
    debug_assert!(rules.iter().any(|rule| rule.guard.is_some()));
    let capacity = values.len();
    resume_collect_string_cases_values_from_index(
        session,
        GuardedStringCasesValuesEvalState {
            values,
            rules: rules.to_vec(),
            max_matches,
            config,
        },
        0,
        Vec::with_capacity(capacity),
    )
}

fn resume_collect_string_cases_from_candidate(
    session: &mut Session,
    state: GuardedStringCasesRuleEvalState,
    mut index: usize,
    mut rule_index: usize,
    matches: Vec<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    if matches.len() >= state.max_matches {
        return Ok(matches);
    }

    loop {
        let Some(rule) = state.rules.get(rule_index) else {
            if index == state.chars.len() {
                return Ok(matches);
            }
            index += 1;
            rule_index = 0;
            continue;
        };

        let Some(match_state) = best_pattern_match_state(
            &rule.rule.pattern,
            &state.chars,
            index,
            state.config.match_config,
        ) else {
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
            return session.drive_internal_value::<bool, Vec<Expr>, _>(
                guard_result,
                move |session, accepted| {
                    resume_collect_string_cases_after_guard(
                        session, state, index, end, rule_index, matches, bindings, accepted,
                    )
                },
            );
        }

        return evaluate_string_cases_output(
            session, state, index, end, matches, bindings, rule_index,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_collect_string_cases_after_guard(
    session: &mut Session,
    state: GuardedStringCasesRuleEvalState,
    index: usize,
    end: usize,
    rule_index: usize,
    matches: Vec<Expr>,
    bindings: SymbolExprMap,
    accepted: bool,
) -> Result<Vec<Expr>, EvalError> {
    if !accepted {
        return resume_collect_string_cases_from_candidate(
            session,
            state,
            index,
            rule_index + 1,
            matches,
        );
    }

    evaluate_string_cases_output(session, state, index, end, matches, bindings, rule_index)
}

fn evaluate_string_cases_output(
    session: &mut Session,
    state: GuardedStringCasesRuleEvalState,
    index: usize,
    end: usize,
    matches: Vec<Expr>,
    bindings: SymbolExprMap,
    rule_index: usize,
) -> Result<Vec<Expr>, EvalError> {
    let rule = &state.rules[rule_index].rule;
    let output_expr = match &rule.output {
        ExactStringCaseOutput::Immediate(expr) | ExactStringCaseOutput::Delayed(expr) => {
            expr.clone()
        }
    };
    let output_result = evaluate_string_rule_output_expr_result(session, &output_expr, &bindings);

    match output_result {
        Ok(output) => {
            resume_collect_string_cases_with_output(session, state, index, end, matches, output)
        }
        Err(error) => session.drive_evaluated_value::<Expr, Vec<Expr>, _>(
            Err(error),
            move |session, output| {
                resume_collect_string_cases_with_output(session, state, index, end, matches, output)
            },
        ),
    }
}

fn resume_collect_string_cases_with_output(
    session: &mut Session,
    state: GuardedStringCasesRuleEvalState,
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

    resume_collect_string_cases_from_candidate(session, state, next_index, 0, matches)
}

fn resume_collect_string_cases_values_from_index(
    session: &mut Session,
    state: GuardedStringCasesValuesEvalState,
    mut index: usize,
    mut outputs: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(value) = state.values.get(index) else {
            return Ok(list_expr(outputs));
        };

        let result = collect_string_cases_with_guarded_rules(
            session,
            value,
            &state.rules,
            state.max_matches,
            state.config,
        );
        match result {
            Ok(items) => {
                outputs.push(list_expr(items));
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                return session.drive_internal_value::<Vec<Expr>, Expr, _>(
                    Err(EvalError::InputRequired { request }),
                    move |session, items| {
                        resume_collect_string_cases_values_with_value(
                            session, state, index, outputs, items,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_collect_string_cases_values_with_value(
    session: &mut Session,
    state: GuardedStringCasesValuesEvalState,
    index: usize,
    mut outputs: Vec<Expr>,
    items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    outputs.push(list_expr(items));
    resume_collect_string_cases_values_from_index(session, state, index + 1, outputs)
}
