use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeTallyTestFromCountSuspensionFrameL159C25 {
    pub(super) state__test: rum_core::Expr,
    pub(super) state__index: usize,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) counts: crate::structure_ops::suspended_state::PersistentStructureCounts,
    pub(super) count_index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeTallyTestFromCountSuspensionFrameL159C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state__test = self.state__test.clone();
        #[allow(unused_mut)]
        let mut state__index = self.state__index.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut counts = self.counts.clone();
        #[allow(unused_mut)]
        let mut count_index = self.count_index.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_tally_test_with_value(
                    session,
                    state__test,
                    items,
                    state__index,
                    counts,
                    count_index,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedTallyTestFromCountSuspensionFrameL217C25 {
    pub(super) test: rum_core::Expr,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) counts: crate::structure_ops::suspended_state::PersistentStructureCounts,
    pub(super) count_index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedTallyTestFromCountSuspensionFrameL217C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut test = self.test.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut counts = self.counts.clone();
        #[allow(unused_mut)]
        let mut count_index = self.count_index.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_tally_test_with_value(
                    session,
                    test,
                    items,
                    index,
                    counts,
                    count_index,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeGatherByFunctionListFromPositionSuspensionFrameL515C25 {
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) functions:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) item_index: usize,
    pub(super) function_index: usize,
    pub(super) keys:
        crate::structure_ops::suspended_state::PersistentStructureSequence<(usize, rum_core::Expr)>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeGatherByFunctionListFromPositionSuspensionFrameL515C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut functions = self.functions.clone();
        #[allow(unused_mut)]
        let mut item_index = self.item_index.clone();
        #[allow(unused_mut)]
        let mut function_index = self.function_index.clone();
        #[allow(unused_mut)]
        let mut keys = self.keys.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_gather_by_function_list_with_key(
                    session,
                    items,
                    functions,
                    item_index,
                    function_index,
                    keys,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedGatherByFunctionListFromPositionSuspensionFrameL589C25 {
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) functions:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) item_index: usize,
    pub(super) function_index: usize,
    pub(super) keys:
        crate::structure_ops::suspended_state::PersistentStructureSequence<(usize, rum_core::Expr)>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedGatherByFunctionListFromPositionSuspensionFrameL589C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut functions = self.functions.clone();
        #[allow(unused_mut)]
        let mut item_index = self.item_index.clone();
        #[allow(unused_mut)]
        let mut function_index = self.function_index.clone();
        #[allow(unused_mut)]
        let mut keys = self.keys.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_gather_by_function_list_with_key(
                    session,
                    items,
                    functions,
                    item_index,
                    function_index,
                    keys,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDecorateByFromIndexSuspensionFrameL793C25 {
    pub(super) state: crate::structure_ops::collection_stats::SuspendedDecorateByState,
    pub(super) index: usize,
    pub(super) decorated: crate::structure_ops::suspended_state::PersistentStructureSequence<
        crate::structure_ops::collection_stats::KeyedEntry,
    >,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeDecorateByFromIndexSuspensionFrameL793C25 {
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
        let mut decorated = self.decorated.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_decorate_by_with_key(session, state, index, decorated, item, key)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedDecorateByFromIndexSuspensionFrameL825C25 {
    pub(super) state: crate::structure_ops::collection_stats::SuspendedDecorateByState,
    pub(super) index: usize,
    pub(super) decorated: crate::structure_ops::suspended_state::PersistentStructureSequence<
        crate::structure_ops::collection_stats::KeyedEntry,
    >,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDecorateByFromIndexSuspensionFrameL825C25
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
        let mut decorated = self.decorated.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_decorate_by_with_key(session, state, index, decorated, item, key)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
