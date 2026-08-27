use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeWhileSequenceFromIndexSuspensionFrameL230C25 {
    pub(super) sequence: crate::functional_ops::sequence::window::SuspendedSequenceLikeExpr,
    pub(super) criterion: rum_core::Expr,
    pub(super) index: usize,
    pub(super) mode: crate::functional_ops::sequence::window::WhileSequenceMode,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeWhileSequenceFromIndexSuspensionFrameL230C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut sequence = self.sequence.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut mode = self.mode.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_while_sequence_with_value(
                    session, sequence, criterion, index, mode, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedWhileSequenceFromIndexSuspensionFrameL269C25 {
    pub(super) sequence: crate::functional_ops::sequence::window::SuspendedSequenceLikeExpr,
    pub(super) criterion: rum_core::Expr,
    pub(super) index: usize,
    pub(super) mode: crate::functional_ops::sequence::window::WhileSequenceMode,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedWhileSequenceFromIndexSuspensionFrameL269C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut sequence = self.sequence.clone();
        #[allow(unused_mut)]
        let mut criterion = self.criterion.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut mode = self.mode.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_while_sequence_with_value(
                    session, sequence, criterion, index, mode, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeAccumulateFromIndexSuspensionFrameL325C25 {
    pub(super) sequence: crate::functional_ops::sequence::window::SuspendedSequenceLikeExpr,
    pub(super) index: usize,
    pub(super) accumulated:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeAccumulateFromIndexSuspensionFrameL325C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut sequence = self.sequence.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut accumulated = self.accumulated.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_accumulate_with_value(session, sequence, index, accumulated, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedAccumulateFromIndexSuspensionFrameL367C25 {
    pub(super) sequence: crate::functional_ops::sequence::window::SuspendedSequenceLikeExpr,
    pub(super) index: usize,
    pub(super) accumulated:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAccumulateFromIndexSuspensionFrameL367C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut sequence = self.sequence.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut accumulated = self.accumulated.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_accumulate_with_value(session, sequence, index, accumulated, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeBlockMapFromIndexSuspensionFrameL422C25 {
    pub(super) state: crate::functional_ops::sequence::window::SuspendedBlockMapEvalState,
    pub(super) index: usize,
    pub(super) results:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeBlockMapFromIndexSuspensionFrameL422C25 {
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
        let mut results = self.results.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_block_map_with_value(session, state, index, results, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedBlockMapFromIndexSuspensionFrameL459C25 {
    pub(super) state: crate::functional_ops::sequence::window::SuspendedBlockMapEvalState,
    pub(super) index: usize,
    pub(super) results:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedBlockMapFromIndexSuspensionFrameL459C25
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
        let mut results = self.results.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_block_map_with_value(session, state, index, results, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
