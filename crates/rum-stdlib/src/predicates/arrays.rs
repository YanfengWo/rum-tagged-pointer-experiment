use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Debug, PartialEq, Eq, FrameRootTrace)]
struct ArrayLeafTestState {
    leaves: Vec<Expr>,
    test: Expr,
    index: usize,
}

pub(crate) fn eval_vector_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => Ok(predicate_bool_expr(
            array_shape(expr).is_some_and(|shape| shape.len() == 1),
        )),
        [expr, test] => {
            let Some(shape) = array_shape(expr) else {
                return Ok(predicate_bool_expr(false));
            };
            if shape.len() != 1 {
                return Ok(predicate_bool_expr(false));
            }

            resume_array_leaf_test_from_index(session, ArrayLeafTestState::new(expr, test))
        }
        _ => Ok(session.invalid_call_with_arg_range("VectorQ", args, 1, 2)),
    }
}

pub(crate) fn eval_matrix_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => Ok(predicate_bool_expr(
            array_shape(expr).is_some_and(|shape| shape.len() == 2),
        )),
        [expr, test] => {
            let Some(shape) = array_shape(expr) else {
                return Ok(predicate_bool_expr(false));
            };
            if shape.len() != 2 {
                return Ok(predicate_bool_expr(false));
            }

            resume_array_leaf_test_from_index(session, ArrayLeafTestState::new(expr, test))
        }
        _ => Ok(session.invalid_call_with_arg_range("MatrixQ", args, 1, 2)),
    }
}

pub(crate) fn eval_array_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    if !(1..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("ArrayQ", args, 1, 3));
    }

    let Some(shape) = array_shape(&args[0]) else {
        return Ok(predicate_bool_expr(false));
    };

    if args.len() == 1 {
        return Ok(predicate_bool_expr(true));
    }

    let leaf_test = args
        .get(2)
        .map(|test| ArrayLeafTestState::new(&args[0], test));
    let rank_value = match rank_pattern_match_result(session, shape.len(), args[1].clone()) {
        Ok(value) => value,
        Err(EvalError::InputRequired { request }) => {
            return session
                .suspend_with_frame_lazy(Err(EvalError::InputRequired { request }), || {
                    move |session, value| finish_array_q_with_rank_value(session, leaf_test, value)
                });
        }
        Err(error) => return Err(error),
    };

    finish_array_q_with_rank_value(session, leaf_test, rank_value)
}

impl ArrayLeafTestState {
    fn new(expr: &Expr, test: &Expr) -> Self {
        let mut leaves = Vec::new();
        collect_array_leaves(expr, &mut leaves);
        Self {
            leaves,
            test: test.clone(),
            index: 0,
        }
    }
}

fn rank_pattern_match_result(
    session: &mut Session,
    rank: usize,
    pattern: Expr,
) -> Result<Expr, EvalError> {
    session.eval_match_q_data_builtin(&[Expr::integer(BigInt::from(rank)), pattern])
}

fn finish_array_q_with_rank_value(
    session: &mut Session,
    leaf_test: Option<ArrayLeafTestState>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&value) != Some(true) {
        return Ok(predicate_bool_expr(false));
    }

    match leaf_test {
        Some(state) => resume_array_leaf_test_from_index(session, state),
        None => Ok(predicate_bool_expr(true)),
    }
}

fn resume_array_leaf_test_from_index(
    session: &mut Session,
    mut state: ArrayLeafTestState,
) -> Result<Expr, EvalError> {
    while let Some(leaf) = state.leaves.get(state.index).cloned() {
        if state.index >= session.resource_limits().iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: session.resource_limits().iteration_work_limit,
            });
        }

        let call = Expr::call(state.test.clone(), vec![leaf]);
        let value = match session.evaluate(call) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame_lazy(
                    Err(EvalError::InputRequired { request }),
                    || {
                        move |session, value| {
                            resume_array_leaf_test_with_value(session, state, value)
                        }
                    },
                );
            }
            Err(error) => return Err(error),
        };

        if truthiness_symbol(&value) != Some(true) {
            return Ok(predicate_bool_expr(false));
        }
        state.index += 1;
    }

    Ok(predicate_bool_expr(true))
}

fn resume_array_leaf_test_with_value(
    session: &mut Session,
    mut state: ArrayLeafTestState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if truthiness_symbol(&value) != Some(true) {
        return Ok(predicate_bool_expr(false));
    }

    state.index += 1;
    resume_array_leaf_test_from_index(session, state)
}

fn collect_array_leaves(expr: &Expr, leaves: &mut Vec<Expr>) {
    match list_items(expr) {
        Some(items) => {
            for item in items {
                collect_array_leaves(item, leaves);
            }
        }
        None => leaves.push(expr.clone()),
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ArrayLeafTestState>();
};
