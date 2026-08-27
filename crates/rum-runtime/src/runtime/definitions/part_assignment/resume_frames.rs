use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumePositionalPartSelectorEvaluationSuspensionFrameL145C48 {
    pub(super) suspended:
        crate::runtime::definitions::part_assignment::SuspendedPositionalPartAssignment,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumePositionalPartSelectorEvaluationSuspensionFrameL145C48
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut suspended = self.suspended.clone();
        let resumed = (|| {
            let selector = outcome?;
            session.resume_suspended_positional_part_assignment(suspended, selector)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeSuspendedPositionalPartAssignmentSuspensionFrameL179C48 {
    pub(super) state:
        crate::runtime::definitions::part_assignment::SuspendedPositionalPartAssignment,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedPositionalPartAssignmentSuspensionFrameL179C48
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            let selector = outcome?;
            session.resume_suspended_positional_part_assignment(state, selector)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
