mod resume_frames;
use resume_frames::*;

use super::super::super::matcher::best_pattern_match_state;
use super::super::rules::evaluate_string_rule_output_expr_result;
use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(FrameRootTrace)]
struct GuardedStringCasesRuleEvalState {
    chars: Vec<char>,
    rules: Vec<GuardedExactStringCaseRule>,
    max_matches: usize,
    config: StringCasesConfig,
}

#[derive(FrameRootTrace)]
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
            return match guard_result {
                Ok(accepted) => resume_collect_string_cases_after_guard(
                    session, state, index, end, rule_index, matches, bindings, accepted,
                ),
                Err(EvalError::InputRequired { request }) => {
                    let state = FrozenStringValue::freeze(state);
                    let matches = PersistentStringSequence::freeze(matches);
                    let bindings = FrozenStringValue::freeze(bindings);
                    rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(accepted) => { (|| {
                                                resume_suspended_collect_string_cases_after_guard(
                                                    session, state, index, end, rule_index, matches, bindings, accepted,
                                                )
                                            })() },
                        suspend => ResumeCollectStringCasesFromCandidateSuspensionFrame { state, index, end, rule_index, matches, bindings },
                    )
                }
                Err(error) => Err(error),
            };
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
        Err(EvalError::InputRequired { request }) => {
            let state = FrozenStringValue::freeze(state);
            let matches = PersistentStringSequence::freeze(matches);
            rum_runtime::resume_or_suspend!(
                session,
                Err(EvalError::InputRequired { request }),
                ok(output_expr) => { let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(output_expr); (|| {
                                resume_suspended_collect_string_cases_with_output(
                                    session, state, index, end, matches, output,
                                )
                            })() },
                suspend => EvaluateStringCasesOutputSuspensionFrame { state, index, end, matches },
            )
        }
        Err(error) => Err(error),
    }
}

fn resume_suspended_collect_string_cases_from_candidate(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringCasesRuleEvalState>,
    mut index: usize,
    mut rule_index: usize,
    matches: PersistentStringSequence<Expr>,
) -> Result<Vec<Expr>, EvalError> {
    if matches.len() >= state.as_ref().max_matches {
        return Ok(matches.to_vec());
    }

    loop {
        let fixed = state.as_ref();
        let Some(rule) = fixed.rules.get(rule_index) else {
            if index == fixed.chars.len() {
                return Ok(matches.to_vec());
            }
            index += 1;
            rule_index = 0;
            continue;
        };

        let Some(match_state) = best_pattern_match_state(
            &rule.rule.pattern,
            &fixed.chars,
            index,
            fixed.config.match_config,
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
                &fixed.chars,
                index,
                end,
                &bindings,
            );
            return match guard_result {
                Ok(accepted) => resume_suspended_collect_string_cases_after_guard_owned(
                    session, state, index, end, rule_index, matches, bindings, accepted,
                ),
                Err(EvalError::InputRequired { request }) => {
                    let bindings = FrozenStringValue::freeze(bindings);
                    rum_runtime::resume_or_suspend!(
                        session,
                        Err(EvalError::InputRequired { request }),
                        ok(accepted) => { (|| {
                                                resume_suspended_collect_string_cases_after_guard(
                                                    session, state, index, end, rule_index, matches, bindings, accepted,
                                                )
                                            })() },
                        suspend => ResumeSuspendedCollectStringCasesFromCandidateSuspensionFrame { state, index, end, rule_index, matches, bindings },
                    )
                }
                Err(error) => Err(error),
            };
        }

        return evaluate_suspended_string_cases_output(
            session, state, index, end, matches, &bindings, rule_index,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_collect_string_cases_after_guard(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringCasesRuleEvalState>,
    index: usize,
    end: usize,
    rule_index: usize,
    matches: PersistentStringSequence<Expr>,
    bindings: FrozenStringValue<SymbolExprMap>,
    accepted: bool,
) -> Result<Vec<Expr>, EvalError> {
    if !accepted {
        return resume_suspended_collect_string_cases_from_candidate(
            session,
            state,
            index,
            rule_index + 1,
            matches,
        );
    }
    evaluate_suspended_string_cases_output(
        session,
        state,
        index,
        end,
        matches,
        bindings.as_ref(),
        rule_index,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_collect_string_cases_after_guard_owned(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringCasesRuleEvalState>,
    index: usize,
    end: usize,
    rule_index: usize,
    matches: PersistentStringSequence<Expr>,
    bindings: SymbolExprMap,
    accepted: bool,
) -> Result<Vec<Expr>, EvalError> {
    if !accepted {
        return resume_suspended_collect_string_cases_from_candidate(
            session,
            state,
            index,
            rule_index + 1,
            matches,
        );
    }
    evaluate_suspended_string_cases_output(
        session, state, index, end, matches, &bindings, rule_index,
    )
}

fn evaluate_suspended_string_cases_output(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringCasesRuleEvalState>,
    index: usize,
    end: usize,
    matches: PersistentStringSequence<Expr>,
    bindings: &SymbolExprMap,
    rule_index: usize,
) -> Result<Vec<Expr>, EvalError> {
    let rule = &state.as_ref().rules[rule_index].rule;
    let output_expr = match &rule.output {
        ExactStringCaseOutput::Immediate(expr) | ExactStringCaseOutput::Delayed(expr) => {
            expr.clone()
        }
    };
    let output_result = evaluate_string_rule_output_expr_result(session, &output_expr, bindings);
    match output_result {
        Ok(output) => resume_suspended_collect_string_cases_with_output(
            session, state, index, end, matches, output,
        ),
        Err(EvalError::InputRequired { request }) => rum_runtime::resume_or_suspend!(
            session,
            Err(EvalError::InputRequired { request }),
            ok(output_expr) => { let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(output_expr); (|| {
                            resume_suspended_collect_string_cases_with_output(
                                session, state, index, end, matches, output,
                            )
                        })() },
            suspend => EvaluateSuspendedStringCasesOutputSuspensionFrame { state, index, end, matches },
        ),
        Err(error) => Err(error),
    }
}

fn resume_suspended_collect_string_cases_with_output(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringCasesRuleEvalState>,
    index: usize,
    end: usize,
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
    resume_suspended_collect_string_cases_from_candidate(session, state, next_index, 0, matches)
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
                let state = FrozenStringValue::freeze(state);
                let outputs = PersistentStringSequence::freeze(outputs);
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(items) => { (|| {
                                        resume_suspended_collect_string_cases_values_with_value(
                                            session, state, index, outputs, items,
                                        )
                                    })() },
                    suspend => ResumeCollectStringCasesValuesFromIndexSuspensionFrame { state, index, outputs },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_collect_string_cases_values_from_index(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringCasesValuesEvalState>,
    mut index: usize,
    mut outputs: PersistentStringSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let fixed = state.as_ref();
        let Some(value) = fixed.values.get(index) else {
            return Ok(list_expr(outputs.to_vec()));
        };
        let result = collect_string_cases_with_guarded_rules(
            session,
            value,
            &fixed.rules,
            fixed.max_matches,
            fixed.config,
        );
        match result {
            Ok(items) => {
                outputs = outputs.appended(list_expr(items));
                index += 1;
            }
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(items) => { (|| {
                                        resume_suspended_collect_string_cases_values_with_value(
                                            session, state, index, outputs, items,
                                        )
                                    })() },
                    suspend => ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrame { state, index, outputs },
                );
            }
            Err(error) => return Err(error),
        }
    }
}

fn resume_suspended_collect_string_cases_values_with_value(
    session: &mut Session,
    state: FrozenStringValue<GuardedStringCasesValuesEvalState>,
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
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<GuardedStringCasesRuleEvalState>();
    assert_traces::<GuardedStringCasesValuesEvalState>();
};
