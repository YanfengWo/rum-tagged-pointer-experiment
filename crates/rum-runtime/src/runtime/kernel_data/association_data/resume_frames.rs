use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrameL783C48 {
    pub(super) suspended:
        crate::runtime::kernel_data::association_data::SuspendedAssociationSetLValue,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeAssociationSetLvalueSelectorEvaluationSuspensionFrameL783C48
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
            session.resume_suspended_association_set_lvalue(suspended, selector)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeSuspendedAssociationSetLvalueSuspensionFrameL819C48 {
    pub(super) state: crate::runtime::kernel_data::association_data::SuspendedAssociationSetLValue,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedAssociationSetLvalueSuspensionFrameL819C48
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
            session.resume_suspended_association_set_lvalue(state, selector)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
