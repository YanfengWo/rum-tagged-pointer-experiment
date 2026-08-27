use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvaluateWithRandomStateRestoredSuspensionFrameL536C13 {
    pub(super) saved: crate::runtime::random::PrngState,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateWithRandomStateRestoredSuspensionFrameL536C13
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut saved = self.saved.clone();
        let resumed = (|| {
            session.prng_state = saved;
            outcome
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
