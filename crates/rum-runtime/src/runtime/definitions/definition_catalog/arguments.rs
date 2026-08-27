use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, FrameRootTrace)]
struct ResolvedArgumentAccumulator {
    prefix: SharedArgs,
    tail: Option<Arc<ResolvedArgumentNode>>,
    len: usize,
}

#[derive(FrameRootTrace)]
struct ResolvedArgumentNode {
    previous: Option<Arc<ResolvedArgumentNode>>,
    value: Expr,
}

impl ResolvedArgumentAccumulator {
    fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: None,
            len,
        }
    }

    fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: Some(Arc::new(ResolvedArgumentNode {
                previous: self.tail.clone(),
                value,
            })),
            len: self.len + 1,
        }
    }

    fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail;
        while let Some(current) = node {
            let current = Arc::try_unwrap(current)
                .unwrap_or_else(|_| panic!("completed argument accumulator must be unique"));
            suffix.push(current.value);
            node = current.previous;
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedArgumentResolution {
    exprs: Arc<[Expr]>,
    cursor: usize,
    resolved: ResolvedArgumentAccumulator,
}

impl Session {
    pub(in crate::runtime) fn resolve_argument_exprs_owned(
        &mut self,
        exprs: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        self.resolve_argument_exprs_from_cursor(exprs, 0, Vec::new())
    }

    fn resolve_argument_exprs_from_cursor(
        &mut self,
        exprs: Vec<Expr>,
        mut cursor: usize,
        mut resolved: Vec<Expr>,
    ) -> Result<Vec<Expr>, EvalError> {
        loop {
            let Some(expr) = exprs.get(cursor) else {
                return Ok(resolved);
            };

            match self.evaluate(expr.clone()) {
                Ok(value) => {
                    resolved.push(value);
                    cursor += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    let suspended = SuspendedArgumentResolution {
                        exprs: Arc::from(exprs),
                        cursor,
                        resolved: ResolvedArgumentAccumulator::freeze(resolved),
                    };
                    self.push_suspension_frame(move |session, outcome| {
                        let value = outcome?;
                        session
                            .resume_suspended_argument_resolution(suspended, value)
                            .map(ContinuationValue::into_continuation_expr)
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_suspended_argument_resolution(
        &mut self,
        mut state: SuspendedArgumentResolution,
        value: Expr,
    ) -> Result<Vec<Expr>, EvalError> {
        state.resolved = state.resolved.appended(value);
        state.cursor += 1;

        loop {
            let Some(expr) = state.exprs.get(state.cursor) else {
                return Ok(state.resolved.into_vec());
            };

            match self.evaluate(expr.clone()) {
                Ok(value) => {
                    state.resolved = state.resolved.appended(value);
                    state.cursor += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, outcome| {
                        let value = outcome?;
                        session
                            .resume_suspended_argument_resolution(state, value)
                            .map(ContinuationValue::into_continuation_expr)
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ResolvedArgumentAccumulator>();
    assert_traces::<SuspendedArgumentResolution>();
};
