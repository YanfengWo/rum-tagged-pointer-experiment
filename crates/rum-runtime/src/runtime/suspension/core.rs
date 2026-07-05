use std::collections::VecDeque;

use super::*;

pub(in crate::runtime) struct SuspendedEvaluation {
    pub(in crate::runtime) active_stack: Vec<EvaluationStackEntry>,
    pub(in crate::runtime) frames: VecDeque<Box<dyn SuspensionFrame + Send>>,
}

pub(in crate::runtime) trait SuspensionFrame {
    fn resume(
        self: Box<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError>;
}

pub trait ContinuationValue: Sized {
    fn into_continuation_expr(self) -> Expr;

    fn from_continuation_expr(expr: Expr) -> Self;
}

struct ClosureFrame<F> {
    resume: Option<F>,
}

impl<F> SuspensionFrame for ClosureFrame<F>
where
    F: FnOnce(&mut Session, Result<Expr, EvalError>) -> Result<Expr, EvalError> + Send + 'static,
{
    fn resume(
        mut self: Box<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let resume = self
            .resume
            .take()
            .expect("suspension frames are resumed at most once");
        resume(session, outcome)
    }
}

impl SuspendedEvaluation {
    fn new(active_stack: Vec<EvaluationStackEntry>) -> Self {
        Self {
            active_stack,
            frames: VecDeque::new(),
        }
    }
}

impl Session {
    /// Drives a potentially suspended evaluated value through a continuation.
    pub fn drive_evaluated_value<T, U, F>(
        &mut self,
        result: Result<Expr, EvalError>,
        cont: F,
    ) -> Result<U, EvalError>
    where
        T: ContinuationValue + 'static,
        U: ContinuationValue + 'static,
        F: FnOnce(&mut Session, T) -> Result<U, EvalError> + Send + 'static,
    {
        match result {
            Ok(value) => cont(self, T::from_continuation_expr(value)),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, outcome| match outcome {
                    Ok(value) => {
                        let resumed = cont(session, T::from_continuation_expr(value))?;
                        Ok(resumed.into_continuation_expr())
                    }
                    Err(error) => Err(error),
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    /// Drives a potentially suspended internal value through a continuation.
    pub fn drive_internal_value<T, U, F>(
        &mut self,
        result: Result<T, EvalError>,
        cont: F,
    ) -> Result<U, EvalError>
    where
        T: ContinuationValue + 'static,
        U: ContinuationValue + 'static,
        F: FnOnce(&mut Session, T) -> Result<U, EvalError> + Send + 'static,
    {
        match result {
            Ok(value) => cont(self, value),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, outcome| match outcome {
                    Ok(value) => {
                        let resumed = cont(session, T::from_continuation_expr(value))?;
                        Ok(resumed.into_continuation_expr())
                    }
                    Err(error) => Err(error),
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    /// Adds a suspension frame that resumes with the evaluated expression value.
    pub fn suspend_with_frame<F>(
        &mut self,
        result: Result<Expr, EvalError>,
        frame: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&mut Session, Expr) -> Result<Expr, EvalError> + Send + 'static,
    {
        self.suspend_with_result_frame(result, move |session, outcome| match outcome {
            Ok(value) => frame(session, value),
            Err(error) => Err(error),
        })
    }

    pub(in crate::runtime) fn suspend_with_frame_lazy<M, F>(
        &mut self,
        result: Result<Expr, EvalError>,
        make_frame: M,
    ) -> Result<Expr, EvalError>
    where
        M: FnOnce() -> F,
        F: FnOnce(&mut Session, Expr) -> Result<Expr, EvalError> + Send + 'static,
    {
        self.suspend_with_result_frame_lazy(result, move || {
            let frame = make_frame();
            move |session, outcome| match outcome {
                Ok(value) => frame(session, value),
                Err(error) => Err(error),
            }
        })
    }

    pub(in crate::runtime) fn suspend_with_result_frame<F>(
        &mut self,
        result: Result<Expr, EvalError>,
        frame: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&mut Session, Result<Expr, EvalError>) -> Result<Expr, EvalError>
            + Send
            + 'static,
    {
        match result {
            Ok(value) => Ok(value),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(frame);
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime) fn suspend_with_result_frame_lazy<M, F>(
        &mut self,
        result: Result<Expr, EvalError>,
        make_frame: M,
    ) -> Result<Expr, EvalError>
    where
        M: FnOnce() -> F,
        F: FnOnce(&mut Session, Result<Expr, EvalError>) -> Result<Expr, EvalError>
            + Send
            + 'static,
    {
        match result {
            Ok(value) => Ok(value),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(make_frame());
                Err(EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime) fn begin_input_suspension(&mut self, request: RuntimeInputRequest) {
        // The current InputString call and its immediate caller are both
        // resumed from saved suspension frames rather than by restarting
        // evaluate_once. Keeping either expression on the active stack causes
        // false-positive cycle detection when matcher backtracking or other
        // helper-level continuations legitimately re-enter that caller after
        // input arrives.
        let retained_depth = self.evaluation_stack.len().saturating_sub(2);
        let active_stack = self
            .evaluation_stack
            .iter()
            .take(retained_depth)
            .cloned()
            .collect::<Vec<_>>();

        self.pending_input_request = Some(request);
        self.pending_suspension = Some(SuspendedEvaluation::new(active_stack));
    }

    pub(in crate::runtime) fn push_suspension_frame<F>(&mut self, frame: F)
    where
        F: FnOnce(&mut Session, Result<Expr, EvalError>) -> Result<Expr, EvalError>
            + Send
            + 'static,
    {
        let suspension = self
            .pending_suspension
            .as_mut()
            .expect("input suspension should be initialized before frames are pushed");
        suspension.frames.push_back(Box::new(ClosureFrame {
            resume: Some(frame),
        }));
    }

    pub(in crate::runtime) fn clear_suspension_state(&mut self) {
        self.pending_input_request = None;
        self.pending_suspension = None;
    }

    pub(in crate::runtime) fn resume_suspended_evaluation(
        &mut self,
        input: impl Into<String>,
    ) -> Result<Expr, EvalError> {
        let stack_marker = 0_u8;
        self.native_stack_base = (&stack_marker as *const u8) as usize;
        let mut suspension = self
            .pending_suspension
            .take()
            .expect("resume requires a pending suspended evaluation");
        let saved_stack = std::mem::replace(&mut self.evaluation_stack, suspension.active_stack);
        let saved_depth =
            std::mem::replace(&mut self.evaluation_depth, self.evaluation_stack.len());
        self.native_stack_next_check_depth = self
            .evaluation_depth
            .saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
        self.pending_input_request = None;
        let mut outcome = Ok(Expr::string(input.into()));

        let result = (|| -> Result<Expr, EvalError> {
            while let Some(frame) = suspension.frames.pop_front() {
                outcome = frame.resume(self, outcome);

                if let Err(EvalError::InputRequired { request }) = &outcome {
                    let mut nested = self
                        .pending_suspension
                        .take()
                        .expect("nested input request should establish a fresh suspension");
                    nested.frames.append(&mut suspension.frames);
                    self.pending_input_request = Some(request.clone());
                    self.pending_suspension = Some(nested);
                    return Err(EvalError::InputRequired {
                        request: request.clone(),
                    });
                }
            }

            outcome
        })();

        self.evaluation_stack = saved_stack;
        self.evaluation_depth = saved_depth;

        match result {
            Ok(value) => {
                self.pending_input_request = None;
                Ok(value)
            }
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            Err(error) => {
                self.clear_suspension_state();
                Err(error)
            }
        }
    }
}
