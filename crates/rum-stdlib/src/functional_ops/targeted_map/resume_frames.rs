use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeMapAtFromIndexSuspensionFrameL332C25 {
    pub(super) state: crate::functional_ops::targeted_map::SuspendedMapAtEvalState,
    pub(super) index: usize,
    pub(super) current: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeMapAtFromIndexSuspensionFrameL332C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut current = self.current.clone();
        let resumed = match outcome {
            Ok(mapped) => {
                (|| resume_suspended_map_at_with_value(session, state, index, current, mapped))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedMapAtFromIndexSuspensionFrameL370C25 {
    pub(super) state: crate::functional_ops::targeted_map::SuspendedMapAtEvalState,
    pub(super) index: usize,
    pub(super) current: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedMapAtFromIndexSuspensionFrameL370C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut current = self.current.clone();
        let resumed = match outcome {
            Ok(mapped) => {
                (|| resume_suspended_map_at_with_value(session, state, index, current, mapped))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct FinishSubsetMapFromMappedSuspensionFrameL423C58;

impl rum_runtime::kernel_api::SuspensionFrame for FinishSubsetMapFromMappedSuspensionFrameL423C58 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct FinishSuspendedSubsetMapFromMappedSuspensionFrameL448C58;

impl rum_runtime::kernel_api::SuspensionFrame
    for FinishSuspendedSubsetMapFromMappedSuspensionFrameL448C58
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        let resumed = match outcome {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ApplySubsetMapSuspensionFrameL677C17 {
    pub(super) suspended_state: crate::functional_ops::targeted_map::SuspendedSubsetMapEvalState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ApplySubsetMapSuspensionFrameL677C17 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut suspended_state = self.suspended_state.clone();
        let resumed = match outcome {
            Ok(mapped) => {
                (|| finish_suspended_subset_map_from_mapped(session, suspended_state, mapped))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
