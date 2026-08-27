use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalCheckAbortSuspensionFrameL121C17 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCheckAbortSuspensionFrameL121C17 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                session.resume_check_abort_with_propagation(
                    &args,
                    check_abort_propagation_from_value(value),
                )
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCheckAbortWithPropagationSuspensionFrameL161C13 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) propagate:
        crate::runtime::kernel_data::control_data::abort_cleanup::PropagateAbortsMode,
    pub(super) pending_checkpoint: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeCheckAbortWithPropagationSuspensionFrameL161C13
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut propagate = self.propagate.clone();
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint.clone();
        let resumed = (|| {
            session.resume_check_abort_after_primary(&args, propagate, pending_checkpoint, outcome)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalAbortProtectSuspensionFrameL191C13 {
    pub(super) pending_checkpoint: usize,
}

impl crate::runtime::suspension::SuspensionFrame for EvalAbortProtectSuspensionFrameL191C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint.clone();
        let resumed = (|| session.resume_abort_protect_after_body(pending_checkpoint, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCheckAbortAfterPrimarySuspensionFrameL213C21 {
    pub(super) propagate:
        crate::runtime::kernel_data::control_data::abort_cleanup::PropagateAbortsMode,
    pub(super) pending_checkpoint: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeCheckAbortAfterPrimarySuspensionFrameL213C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut propagate = self.propagate.clone();
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint.clone();
        let resumed = (|| {
            session.resume_check_abort_after_fallback(propagate, true, pending_checkpoint, outcome)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCheckAbortAfterPrimarySuspensionFrameL240C21 {
    pub(super) propagate:
        crate::runtime::kernel_data::control_data::abort_cleanup::PropagateAbortsMode,
    pub(super) protected: bool,
    pub(super) pending_checkpoint: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeCheckAbortAfterPrimarySuspensionFrameL240C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut propagate = self.propagate.clone();
        #[allow(unused_mut)]
        let mut protected = self.protected.clone();
        #[allow(unused_mut)]
        let mut pending_checkpoint = self.pending_checkpoint.clone();
        let resumed = (|| {
            session.resume_check_abort_after_fallback(
                propagate,
                protected,
                pending_checkpoint,
                outcome,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
