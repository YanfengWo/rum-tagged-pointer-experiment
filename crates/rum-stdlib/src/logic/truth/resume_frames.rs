use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeAndFromIndexSuspensionFrameL180C29 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
    pub(super) residual: crate::logic::suspended_state::PersistentLogicSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeAndFromIndexSuspensionFrameL180C29 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut residual = self.residual.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_and_with_value(session, args, index, residual, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedAndFromIndexSuspensionFrameL220C29 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
    pub(super) residual: crate::logic::suspended_state::PersistentLogicSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAndFromIndexSuspensionFrameL220C29
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut residual = self.residual.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_and_with_value(session, args, index, residual, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeOrFromIndexSuspensionFrameL278C29 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
    pub(super) residual: crate::logic::suspended_state::PersistentLogicSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeOrFromIndexSuspensionFrameL278C29 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut residual = self.residual.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_or_with_value(session, args, index, residual, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedOrFromIndexSuspensionFrameL318C29 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
    pub(super) residual: crate::logic::suspended_state::PersistentLogicSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSuspendedOrFromIndexSuspensionFrameL318C29 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut residual = self.residual.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_or_with_value(session, args, index, residual, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeNandFromIndexSuspensionFrameL376C29 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
    pub(super) residual: crate::logic::suspended_state::PersistentLogicSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeNandFromIndexSuspensionFrameL376C29 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut residual = self.residual.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_nand_with_value(session, args, index, residual, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedNandFromIndexSuspensionFrameL416C29 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
    pub(super) residual: crate::logic::suspended_state::PersistentLogicSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedNandFromIndexSuspensionFrameL416C29
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut residual = self.residual.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_nand_with_value(session, args, index, residual, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeNorFromIndexSuspensionFrameL475C29 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
    pub(super) residual: crate::logic::suspended_state::PersistentLogicSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeNorFromIndexSuspensionFrameL475C29 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut residual = self.residual.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_nor_with_value(session, args, index, residual, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedNorFromIndexSuspensionFrameL515C29 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
    pub(super) residual: crate::logic::suspended_state::PersistentLogicSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedNorFromIndexSuspensionFrameL515C29
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut residual = self.residual.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_nor_with_value(session, args, index, residual, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
