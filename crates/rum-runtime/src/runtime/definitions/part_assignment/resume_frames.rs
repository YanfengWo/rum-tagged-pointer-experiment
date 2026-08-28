use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumePositionalPartSelectorEvaluationSuspensionFrame {
    pub(super) suspended:
        crate::runtime::definitions::part_assignment::SuspendedPositionalPartAssignment,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumePositionalPartSelectorEvaluationSuspensionFrame
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
pub(super) struct ResumeSuspendedPositionalPartAssignmentSuspensionFrame {
    pub(super) state:
        crate::runtime::definitions::part_assignment::SuspendedPositionalPartAssignment,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedPositionalPartAssignmentSuspensionFrame
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
