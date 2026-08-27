mod resume_frames;
use resume_frames::*;

use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
struct NestWhileConfig {
    test_arity: NestWhileTestArity,
    max_iterations: Option<usize>,
}

#[derive(Clone, FrameRootTrace)]
struct NestEvalState {
    operator: Expr,
    count: usize,
    collect_all: bool,
}

#[derive(Clone, FrameRootTrace)]
struct FixedPointEvalState {
    operator: Expr,
    max_iterations: Option<usize>,
    limit: usize,
    collect_all: bool,
}

#[derive(Clone, FrameRootTrace)]
struct NestWhileEvalState {
    operator: Expr,
    test: Expr,
    config: NestWhileConfig,
    limit: usize,
    collect_all: bool,
}

impl Default for NestWhileConfig {
    fn default() -> Self {
        Self {
            test_arity: NestWhileTestArity::Count(1),
            max_iterations: None,
        }
    }
}

pub(crate) fn eval_nest(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator, count_expr] => {
            let Some(count) = expr_as_non_negative_usize(count_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "Nest",
                    args,
                    "expected a non-negative integer iteration count",
                ));
            };

            Ok(nest_operator_form("Nest", operator, count))
        }
        [operator, expr, count_expr] => {
            let Some(count) = expr_as_non_negative_usize(count_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "Nest",
                    args,
                    "expected a non-negative integer iteration count",
                ));
            };

            apply_nest(session, operator, expr, count)
        }
        _ => Ok(session.invalid_call_with_arg_range("Nest", args, 2, 3)),
    }
}

pub(crate) fn eval_nest_while(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator, test] => Ok(unary_operator_form_with_tail(
            "NestWhile",
            operator.clone(),
            test.clone(),
        )),
        [operator, expr, test] => apply_nest_while(
            session,
            operator,
            expr,
            test,
            NestWhileConfig::default(),
            false,
        ),
        [operator, expr, test, m_expr] => {
            let Some(config) = parse_nest_while_config(m_expr, None) else {
                return Ok(session.invalid_call_with_arg_form(
                    "NestWhile",
                    args,
                    "expected All or a positive integer test arity as the fourth argument",
                ));
            };

            apply_nest_while(session, operator, expr, test, config, false)
        }
        [operator, expr, test, m_expr, max_expr] => {
            let Some(config) = parse_nest_while_config(m_expr, Some(max_expr)) else {
                return Ok(session.invalid_call_with_arg_form(
                    "NestWhile",
                    args,
                    "expected All or a positive integer test arity followed by a non-negative integer iteration limit",
                ));
            };

            apply_nest_while(session, operator, expr, test, config, false)
        }
        _ => Ok(session.invalid_call_with_arg_range("NestWhile", args, 2, 5)),
    }
}

pub(crate) fn eval_nest_list(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator, count_expr] => {
            let Some(count) = expr_as_non_negative_usize(count_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "NestList",
                    args,
                    "expected a non-negative integer iteration count",
                ));
            };

            Ok(nest_operator_form("NestList", operator, count))
        }
        [operator, expr, count_expr] => {
            let Some(count) = expr_as_non_negative_usize(count_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "NestList",
                    args,
                    "expected a non-negative integer iteration count",
                ));
            };

            apply_nest_list(session, operator, expr, count)
        }
        _ => Ok(session.invalid_call_with_arg_range("NestList", args, 2, 3)),
    }
}

pub(crate) fn eval_nest_while_list(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator, test] => Ok(unary_operator_form_with_tail(
            "NestWhileList",
            operator.clone(),
            test.clone(),
        )),
        [operator, expr, test] => apply_nest_while(
            session,
            operator,
            expr,
            test,
            NestWhileConfig::default(),
            true,
        ),
        [operator, expr, test, m_expr] => {
            let Some(config) = parse_nest_while_config(m_expr, None) else {
                return Ok(session.invalid_call_with_arg_form(
                    "NestWhileList",
                    args,
                    "expected All or a positive integer test arity as the fourth argument",
                ));
            };

            apply_nest_while(session, operator, expr, test, config, true)
        }
        [operator, expr, test, m_expr, max_expr] => {
            let Some(config) = parse_nest_while_config(m_expr, Some(max_expr)) else {
                return Ok(session.invalid_call_with_arg_form(
                    "NestWhileList",
                    args,
                    "expected All or a positive integer test arity followed by a non-negative integer iteration limit",
                ));
            };

            apply_nest_while(session, operator, expr, test, config, true)
        }
        _ => Ok(session.invalid_call_with_arg_range("NestWhileList", args, 2, 5)),
    }
}

pub(crate) fn eval_fixed_point_list(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator] => Ok(unary_operator_form("FixedPointList", operator)),
        [operator, expr] => apply_fixed_point_list(session, operator, expr, None),
        [operator, expr, max_expr] => {
            let Some(max_iterations) = expr_as_non_negative_usize(max_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "FixedPointList",
                    args,
                    "expected a non-negative integer iteration limit",
                ));
            };

            apply_fixed_point_list(session, operator, expr, Some(max_iterations))
        }
        _ => Ok(session.invalid_call_with_arg_range("FixedPointList", args, 1, 3)),
    }
}

pub(crate) fn eval_fixed_point(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [operator] => Ok(unary_operator_form("FixedPoint", operator)),
        [operator, expr] => apply_fixed_point(session, operator, expr, None),
        [operator, expr, max_expr] => {
            let Some(max_iterations) = expr_as_non_negative_usize(max_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "FixedPoint",
                    args,
                    "expected a non-negative integer iteration limit",
                ));
            };

            apply_fixed_point(session, operator, expr, Some(max_iterations))
        }
        _ => Ok(session.invalid_call_with_arg_range("FixedPoint", args, 1, 3)),
    }
}

fn resume_nest_from_iteration(
    session: &mut Session,
    state: NestEvalState,
    iteration: usize,
    current: Expr,
    items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    if iteration >= state.count {
        return Ok(finish_nest_result(state.collect_all, current, items));
    }

    let limit = session.resource_limits().evaluation_pass_limit;
    if iteration >= limit {
        return Err(EvalError::EvaluationPassLimitExceeded { limit });
    }

    let value_result = session.evaluate(Expr::call(state.operator.clone(), vec![current]));
    match value_result {
        Ok(value) => resume_nest_with_value(session, state, iteration, items, value),
        Err(error) => rum_runtime::resume_or_suspend!(
            session,
            Err(error),
            ok(value) => Ok(value),
            suspend => {
        let items = PersistentFunctionalSequence::freeze(items);
        ResumeNestFromIterationSuspensionFrameL271C13 { state, iteration, items }
        },
        ),
    }
}

fn resume_suspended_nest_from_iteration(
    session: &mut Session,
    state: NestEvalState,
    iteration: usize,
    current: Expr,
    items: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    if iteration >= state.count {
        return Ok(finish_nest_result(
            state.collect_all,
            current,
            items.to_vec(),
        ));
    }

    let limit = session.resource_limits().evaluation_pass_limit;
    if iteration >= limit {
        return Err(EvalError::EvaluationPassLimitExceeded { limit });
    }

    let value_result = session.evaluate(Expr::call(state.operator.clone(), vec![current]));
    match value_result {
        Ok(value) => resume_suspended_nest_with_value(session, state, iteration, items, value),
        Err(error) => rum_runtime::resume_or_suspend!(
            session,
            Err(error),
            ok(value) => Ok(value),
            suspend => ResumeSuspendedNestFromIterationSuspensionFrameL302C13 { state, iteration, items },
        ),
    }
}

fn resume_suspended_nest_with_value(
    session: &mut Session,
    state: NestEvalState,
    iteration: usize,
    mut items: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if state.collect_all {
        items = items.appended(value.clone());
    }

    resume_suspended_nest_from_iteration(session, state, iteration + 1, value, items)
}

fn resume_nest_with_value(
    session: &mut Session,
    state: NestEvalState,
    iteration: usize,
    mut items: Vec<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if state.collect_all {
        items.push(value.clone());
    }

    resume_nest_from_iteration(session, state, iteration + 1, value, items)
}

fn resume_fixed_point_from_iteration(
    session: &mut Session,
    state: FixedPointEvalState,
    iteration: usize,
    current: Expr,
    seen: Vec<Expr>,
    items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    if iteration >= state.limit {
        return finish_fixed_point_at_limit(state, current, items);
    }

    let value_result = session.evaluate(Expr::call(state.operator.clone(), vec![current.clone()]));
    match value_result {
        Ok(value) => {
            resume_fixed_point_with_value(session, state, iteration, current, seen, items, value)
        }
        Err(error) => rum_runtime::resume_or_suspend!(
            session,
            Err(error),
            ok(value) => Ok(value),
            suspend => {
        let seen = PersistentFunctionalSequence::freeze(seen);
                    let items = PersistentFunctionalSequence::freeze(items);
        ResumeFixedPointFromIterationSuspensionFrameL357C13 { state, iteration, current, seen, items }
        },
        ),
    }
}

fn resume_suspended_fixed_point_from_iteration(
    session: &mut Session,
    state: FixedPointEvalState,
    iteration: usize,
    current: Expr,
    seen: PersistentFunctionalSequence<Expr>,
    items: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    if iteration >= state.limit {
        return finish_fixed_point_at_limit(state, current, items.to_vec());
    }

    let value_result = session.evaluate(Expr::call(state.operator.clone(), vec![current.clone()]));
    match value_result {
        Ok(value) => resume_suspended_fixed_point_with_value(
            session, state, iteration, current, seen, items, value,
        ),
        Err(error) => rum_runtime::resume_or_suspend!(
            session,
            Err(error),
            ok(value) => Ok(value),
            suspend => ResumeSuspendedFixedPointFromIterationSuspensionFrameL384C13 { state, iteration, current, seen, items },
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_fixed_point_with_value(
    session: &mut Session,
    state: FixedPointEvalState,
    iteration: usize,
    current: Expr,
    mut seen: PersistentFunctionalSequence<Expr>,
    mut items: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if value == current {
        if state.collect_all {
            items = items.appended(value.clone());
        }
        return finish_fixed_point_value(state, value, items.to_vec());
    }

    if state.max_iterations.is_none() {
        let repeats = if state.collect_all {
            items.any(|previous| previous == &value)
        } else {
            seen.any(|previous| previous == &value)
        };
        if repeats {
            return Err(EvalError::EvaluationCycleDetected {
                expr: value.to_string(),
            });
        }
    }

    if state.collect_all {
        items = items.appended(value.clone());
    } else if state.max_iterations.is_none() {
        seen = seen.appended(value.clone());
    }
    let next_iteration = iteration + 1;
    resume_suspended_fixed_point_from_iteration(session, state, next_iteration, value, seen, items)
}

fn resume_fixed_point_with_value(
    session: &mut Session,
    state: FixedPointEvalState,
    iteration: usize,
    current: Expr,
    mut seen: Vec<Expr>,
    mut items: Vec<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if value == current {
        if state.collect_all {
            items.push(value.clone());
        }
        return finish_fixed_point_value(state, value, items);
    }

    if state.max_iterations.is_none() {
        let previous_values = if state.collect_all { &items } else { &seen };
        if previous_values.iter().any(|previous| previous == &value) {
            return Err(EvalError::EvaluationCycleDetected {
                expr: value.to_string(),
            });
        }
    }

    if state.collect_all {
        items.push(value.clone());
    } else if state.max_iterations.is_none() {
        seen.push(value.clone());
    }
    let next_iteration = iteration + 1;
    resume_fixed_point_from_iteration(session, state, next_iteration, value, seen, items)
}

fn resume_nest_while_test_from_iteration(
    session: &mut Session,
    state: NestWhileEvalState,
    iteration: usize,
    history: Vec<Expr>,
) -> Result<Expr, EvalError> {
    let test_args = nest_while_test_args(&history, state.config.test_arity);
    let value_result = session.evaluate(Expr::call(state.test.clone(), test_args));
    match value_result {
        Ok(test_value) => {
            resume_nest_while_with_test_value(session, state, iteration, history, test_value)
        }
        Err(error) => rum_runtime::resume_or_suspend!(
            session,
            Err(error),
            ok(value) => Ok(value),
            suspend => {
        let history = PersistentFunctionalSequence::freeze(history);
        ResumeNestWhileTestFromIterationSuspensionFrameL480C13 { state, iteration, history }
        },
        ),
    }
}

fn resume_suspended_nest_while_test_from_iteration(
    session: &mut Session,
    state: NestWhileEvalState,
    iteration: usize,
    history: PersistentFunctionalSequence<Expr>,
) -> Result<Expr, EvalError> {
    let test_args = persistent_nest_while_test_args(&history, state.config.test_arity);
    let value_result = session.evaluate(Expr::call(state.test.clone(), test_args));
    match value_result {
        Ok(test_value) => resume_suspended_nest_while_with_test_value(
            session, state, iteration, history, test_value,
        ),
        Err(error) => rum_runtime::resume_or_suspend!(
            session,
            Err(error),
            ok(value) => Ok(value),
            suspend => ResumeSuspendedNestWhileTestFromIterationSuspensionFrameL502C13 { state, iteration, history },
        ),
    }
}

fn resume_nest_while_with_test_value(
    session: &mut Session,
    state: NestWhileEvalState,
    iteration: usize,
    history: Vec<Expr>,
    test_value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&test_value) != Some(true) {
        return Ok(finish_nest_while_result(state.collect_all, history));
    }

    if iteration >= state.limit {
        return if state.config.max_iterations.is_some() {
            Ok(finish_nest_while_result(state.collect_all, history))
        } else {
            Err(EvalError::EvaluationPassLimitExceeded { limit: state.limit })
        };
    }

    let current = history
        .last()
        .cloned()
        .expect("NestWhile history always starts with the initial expression");
    let value_result = session.evaluate(Expr::call(state.operator.clone(), vec![current]));
    match value_result {
        Ok(value) => resume_nest_while_with_step_value(session, state, iteration, history, value),
        Err(error) => rum_runtime::resume_or_suspend!(
            session,
            Err(error),
            ok(value) => Ok(value),
            suspend => {
        let history = PersistentFunctionalSequence::freeze(history);
        ResumeNestWhileWithTestValueSuspensionFrameL539C13 { state, iteration, history }
        },
        ),
    }
}

fn resume_suspended_nest_while_with_test_value(
    session: &mut Session,
    state: NestWhileEvalState,
    iteration: usize,
    history: PersistentFunctionalSequence<Expr>,
    test_value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&test_value) != Some(true) {
        return Ok(finish_suspended_nest_while_result(
            state.collect_all,
            history,
        ));
    }

    if iteration >= state.limit {
        return if state.config.max_iterations.is_some() {
            Ok(finish_suspended_nest_while_result(
                state.collect_all,
                history,
            ))
        } else {
            Err(EvalError::EvaluationPassLimitExceeded { limit: state.limit })
        };
    }

    let current = history
        .last()
        .cloned()
        .expect("NestWhile history always starts with the initial expression");
    let value_result = session.evaluate(Expr::call(state.operator.clone(), vec![current]));
    match value_result {
        Ok(value) => {
            resume_suspended_nest_while_with_step_value(session, state, iteration, history, value)
        }
        Err(error) => rum_runtime::resume_or_suspend!(
            session,
            Err(error),
            ok(value) => Ok(value),
            suspend => ResumeSuspendedNestWhileWithTestValueSuspensionFrameL583C13 { state, iteration, history },
        ),
    }
}

fn resume_suspended_nest_while_with_step_value(
    session: &mut Session,
    state: NestWhileEvalState,
    iteration: usize,
    history: PersistentFunctionalSequence<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let history = history.appended(value);
    resume_suspended_nest_while_test_from_iteration(session, state, iteration + 1, history)
}

fn resume_nest_while_with_step_value(
    session: &mut Session,
    state: NestWhileEvalState,
    iteration: usize,
    mut history: Vec<Expr>,
    value: Expr,
) -> Result<Expr, EvalError> {
    history.push(value);
    resume_nest_while_test_from_iteration(session, state, iteration + 1, history)
}

fn parse_nest_while_config(m_expr: &Expr, max_expr: Option<&Expr>) -> Option<NestWhileConfig> {
    let test_arity = if is_system_symbol_name(m_expr, "All") {
        NestWhileTestArity::All
    } else {
        let count = expr_as_non_negative_usize(m_expr)?;
        if count == 0 {
            return None;
        }
        NestWhileTestArity::Count(count)
    };

    let max_iterations = match max_expr {
        Some(expr) => Some(expr_as_non_negative_usize(expr)?),
        None => None,
    };

    Some(NestWhileConfig {
        test_arity,
        max_iterations,
    })
}

fn apply_nest(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    count: usize,
) -> Result<Expr, EvalError> {
    resume_nest_from_iteration(
        session,
        NestEvalState {
            operator: operator.clone(),
            count,
            collect_all: false,
        },
        0,
        expr.clone(),
        Vec::new(),
    )
}

fn apply_nest_list(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    count: usize,
) -> Result<Expr, EvalError> {
    resume_nest_from_iteration(
        session,
        NestEvalState {
            operator: operator.clone(),
            count,
            collect_all: true,
        },
        0,
        expr.clone(),
        vec![expr.clone()],
    )
}

fn apply_fixed_point(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    max_iterations: Option<usize>,
) -> Result<Expr, EvalError> {
    let limit = max_iterations.unwrap_or(session.resource_limits().evaluation_pass_limit);
    let seen = if max_iterations.is_none() {
        vec![expr.clone()]
    } else {
        Vec::new()
    };
    resume_fixed_point_from_iteration(
        session,
        FixedPointEvalState {
            operator: operator.clone(),
            max_iterations,
            limit,
            collect_all: false,
        },
        0,
        expr.clone(),
        seen,
        Vec::new(),
    )
}

fn apply_fixed_point_list(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    max_iterations: Option<usize>,
) -> Result<Expr, EvalError> {
    let limit = max_iterations.unwrap_or(session.resource_limits().evaluation_pass_limit);
    resume_fixed_point_from_iteration(
        session,
        FixedPointEvalState {
            operator: operator.clone(),
            max_iterations,
            limit,
            collect_all: true,
        },
        0,
        expr.clone(),
        Vec::new(),
        vec![expr.clone()],
    )
}

fn apply_nest_while(
    session: &mut Session,
    operator: &Expr,
    expr: &Expr,
    test: &Expr,
    config: NestWhileConfig,
    collect_all: bool,
) -> Result<Expr, EvalError> {
    let limit = config
        .max_iterations
        .unwrap_or(session.resource_limits().evaluation_pass_limit);
    resume_nest_while_test_from_iteration(
        session,
        NestWhileEvalState {
            operator: operator.clone(),
            test: test.clone(),
            config,
            limit,
            collect_all,
        },
        0,
        vec![expr.clone()],
    )
}

fn finish_nest_result(collect_all: bool, current: Expr, items: Vec<Expr>) -> Expr {
    if collect_all {
        Expr::call(system_expr("List"), items)
    } else {
        current
    }
}

fn finish_fixed_point_at_limit(
    state: FixedPointEvalState,
    current: Expr,
    items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    if state.collect_all {
        return Ok(Expr::call(system_expr("List"), items));
    }

    if state.max_iterations.is_some() {
        Ok(current)
    } else {
        Err(EvalError::EvaluationPassLimitExceeded { limit: state.limit })
    }
}

fn finish_fixed_point_value(
    state: FixedPointEvalState,
    value: Expr,
    items: Vec<Expr>,
) -> Result<Expr, EvalError> {
    if state.collect_all {
        Ok(Expr::call(system_expr("List"), items))
    } else {
        Ok(value)
    }
}

fn finish_nest_while_result(collect_all: bool, history: Vec<Expr>) -> Expr {
    if collect_all {
        Expr::call(system_expr("List"), history)
    } else {
        history
            .last()
            .cloned()
            .expect("NestWhile history always starts with the initial expression")
    }
}

fn finish_suspended_nest_while_result(
    collect_all: bool,
    history: PersistentFunctionalSequence<Expr>,
) -> Expr {
    if collect_all {
        Expr::call(system_expr("List"), history.to_vec())
    } else {
        history
            .last()
            .cloned()
            .expect("NestWhile history always starts with the initial expression")
    }
}

fn persistent_nest_while_test_args(
    history: &PersistentFunctionalSequence<Expr>,
    arity: NestWhileTestArity,
) -> Vec<Expr> {
    match arity {
        NestWhileTestArity::Count(count) => history.recent(count),
        NestWhileTestArity::All => history.to_vec(),
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FixedPointEvalState>();
    assert_traces::<NestEvalState>();
    assert_traces::<NestWhileConfig>();
    assert_traces::<NestWhileEvalState>();
};
