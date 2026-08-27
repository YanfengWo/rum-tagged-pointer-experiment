use std::collections::VecDeque;
use std::mem::{size_of, size_of_val};
use std::sync::Arc;

use super::trace::{FrameRootTrace, FrameRootVisitor};
use super::*;

type DynSuspensionFrame = dyn SuspensionFrame;

pub(in crate::runtime) struct SuspendedEvaluation {
    pub(in crate::runtime) active_stack: Vec<EvaluationStackEntry>,
    frames: VecDeque<Arc<DynSuspensionFrame>>,
}

/// A traceable, immutable continuation frame installed at an input boundary.
#[doc(hidden)]
pub trait SuspensionFrame: FrameRootTrace + Send + Sync + 'static {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction;
}

/// A Session-free completion step run after its originating frame is dropped.
#[doc(hidden)]
pub trait PureFrameFinalizer: Send + 'static {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError>;
}

/// The closed transition protocol between a named frame and the driver.
#[doc(hidden)]
pub enum FrameAction {
    Complete(Result<Expr, EvalError>),
    Evaluate {
        expr: Expr,
        next_frame: Arc<DynSuspensionFrame>,
    },
    Finalize(Box<dyn PureFrameFinalizer>),
}

impl FrameAction {
    #[inline]
    pub fn complete(outcome: Result<Expr, EvalError>) -> Self {
        Self::Complete(outcome)
    }

    #[inline]
    pub fn evaluate<F>(expr: Expr, next_frame: F) -> Self
    where
        F: SuspensionFrame,
    {
        Self::Evaluate {
            expr,
            next_frame: Arc::new(next_frame),
        }
    }

    #[inline]
    pub fn finalize<F>(finalizer: F) -> Self
    where
        F: PureFrameFinalizer,
    {
        Self::Finalize(Box::new(finalizer))
    }
}

pub trait ContinuationValue: Sized {
    fn into_continuation_expr(self) -> Expr;

    fn from_continuation_expr(expr: Expr) -> Self;
}

impl SuspendedEvaluation {
    fn new(active_stack: Vec<EvaluationStackEntry>) -> Self {
        Self {
            active_stack,
            frames: VecDeque::new(),
        }
    }

    fn push_back(&mut self, frame: Arc<DynSuspensionFrame>) {
        self.frames.push_back(frame);
    }

    fn push_front(&mut self, frame: Arc<DynSuspensionFrame>) {
        self.frames.push_front(frame);
    }

    fn front(&self) -> Option<Arc<DynSuspensionFrame>> {
        self.frames.front().cloned()
    }

    fn pop_front(&mut self) -> Arc<DynSuspensionFrame> {
        self.frames
            .pop_front()
            .expect("the resumed suspension frame should remain queued")
    }

    fn append(&mut self, suffix: &mut Self) {
        self.frames.append(&mut suffix.frames);
    }

    pub(in crate::runtime) fn trace_frame_roots(&self, visitor: &mut dyn FrameRootVisitor) {
        for frame in &self.frames {
            frame.trace_roots(visitor);
        }
    }

    pub(in crate::runtime) fn frame_storage_memory_proxy_bytes(&self) -> usize {
        let queue_storage = self
            .frames
            .capacity()
            .saturating_mul(size_of::<Arc<DynSuspensionFrame>>());
        let allocations = self.frames.iter().fold(0usize, |total, frame| {
            total
                .saturating_add(2usize.saturating_mul(size_of::<usize>()))
                .saturating_add(size_of_val(frame.as_ref()))
        });
        queue_storage.saturating_add(allocations)
    }
}

impl Session {
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

    /// Inserts a named, traceable frame into the fresh pending suspension.
    #[doc(hidden)]
    #[cold]
    #[inline(never)]
    pub fn insert_suspension_frame<F>(&mut self, frame: F)
    where
        F: SuspensionFrame,
    {
        let suspension = self
            .pending_suspension
            .as_mut()
            .expect("input suspension should be initialized before frames are pushed");
        suspension.push_back(Arc::new(frame));
    }

    pub(in crate::runtime) fn clear_pending_suspension_state(&mut self) {
        self.pending_input_request = None;
        self.pending_suspension = None;
    }

    pub(in crate::runtime) fn clear_suspension_state(&mut self) {
        self.clear_pending_suspension_state();
        self.resuming_suspension = None;
    }

    pub(in crate::runtime) fn resume_suspended_evaluation(
        &mut self,
        input: impl Into<String>,
    ) -> Result<Expr, EvalError> {
        let stack_marker = 0_u8;
        self.native_stack_base = (&stack_marker as *const u8) as usize;
        debug_assert!(
            self.resuming_suspension.is_none(),
            "a Session resumes only one suspended evaluation at a time"
        );
        self.resuming_suspension = Some(
            self.pending_suspension
                .take()
                .expect("resume requires a pending suspended evaluation"),
        );
        let active_stack = std::mem::take(
            &mut self
                .resuming_suspension
                .as_mut()
                .expect("resuming suspension was just installed")
                .active_stack,
        );
        let saved_stack = std::mem::replace(&mut self.evaluation_stack, active_stack);
        let saved_depth =
            std::mem::replace(&mut self.evaluation_depth, self.evaluation_stack.len());
        self.native_stack_next_check_depth = self
            .evaluation_depth
            .saturating_add(NATIVE_STACK_DEPTH_POLL_STRIDE);
        self.pending_input_request = None;
        let outcome = Ok(Expr::string(input.into()));

        let result = self.drive_resuming_frames(outcome);

        self.evaluation_stack = saved_stack;
        self.evaluation_depth = saved_depth;

        match result {
            Ok(value) => {
                self.pending_input_request = None;
                self.resuming_suspension = None;
                Ok(value)
            }
            Err(EvalError::InputRequired { request }) => {
                debug_assert!(self.resuming_suspension.is_none());
                Err(EvalError::InputRequired { request })
            }
            Err(error) => {
                self.clear_suspension_state();
                Err(error)
            }
        }
    }

    fn drive_resuming_frames(
        &mut self,
        mut outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        loop {
            let Some(dispatch_frame) = self
                .resuming_suspension
                .as_ref()
                .and_then(SuspendedEvaluation::front)
            else {
                return outcome;
            };

            let action = dispatch_frame.resume(self, outcome);
            let completed_frame = self
                .resuming_suspension
                .as_mut()
                .expect("the active resumption remains Session-rooted")
                .pop_front();
            drop(completed_frame);

            outcome = match action {
                FrameAction::Complete(outcome) => outcome,
                FrameAction::Evaluate { expr, next_frame } => {
                    self.resuming_suspension
                        .as_mut()
                        .expect("the active resumption remains Session-rooted")
                        .push_front(next_frame);
                    self.evaluate(expr)
                }
                FrameAction::Finalize(finalizer) => finalizer.finish(),
            };

            if let Err(EvalError::InputRequired { request }) = &outcome {
                let mut nested = self
                    .pending_suspension
                    .take()
                    .expect("nested input request should establish a fresh suspension");
                let mut suffix = self
                    .resuming_suspension
                    .take()
                    .expect("the still-rooted suffix should remain in Session");
                nested.append(&mut suffix);
                self.pending_input_request = Some(request.clone());
                self.pending_suspension = Some(nested);
                return Err(EvalError::InputRequired {
                    request: request.clone(),
                });
            }
        }
    }
}

/// Matches a potentially suspended result without constructing frame state on
/// the ordinary path. The macro accepts data expressions only, never closures.
#[doc(hidden)]
#[macro_export]
macro_rules! resume_or_suspend {
    (
        $session:expr,
        $result:expr,
        ok($value:pat) => $continue:expr,
        suspend => $frame:expr $(,)?
    ) => {{
        match $result {
            Ok($value) => $continue,
            Err($crate::kernel_api::EvalError::InputRequired { request }) => {
                $session.insert_suspension_frame($frame);
                Err($crate::kernel_api::EvalError::InputRequired { request })
            }
            Err(error) => Err(error),
        }
    }};
    (insert $session:expr, $frame:expr $(,)?) => {{ $session.insert_suspension_frame($frame) }};
}
