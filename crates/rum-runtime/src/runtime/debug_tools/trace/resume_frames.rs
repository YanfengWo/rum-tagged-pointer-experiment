use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalStructuredTraceCaptureExprSuspensionFrameL124C13 {
    pub(super) suspended_state: crate::runtime::debug_tools::trace::TraceEvalState,
    pub(super) trace_checkpoint: usize,
    pub(super) prior_enabled: bool,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvalStructuredTraceCaptureExprSuspensionFrameL124C13
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended_state = self.suspended_state.clone();
        #[allow(unused_mut)]
        let mut trace_checkpoint = self.trace_checkpoint.clone();
        #[allow(unused_mut)]
        let mut prior_enabled = self.prior_enabled.clone();
        let resumed = (|| {
            session.finish_structured_trace_capture_expr(
                suspended_state,
                trace_checkpoint,
                prior_enabled,
                outcome,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
