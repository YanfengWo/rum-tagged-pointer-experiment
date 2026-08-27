use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeMergeNestedFromIndexSuspensionFrameL139C25 {
    pub(super) combiner: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) merged: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeMergeNestedFromIndexSuspensionFrameL139C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut combiner = self.combiner.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut merged = self.merged.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_merge_nested_with_value(
                    session, combiner, grouped, index, merged, key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedMergeNestedFromIndexSuspensionFrameL173C25 {
    pub(super) combiner: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) merged: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedMergeNestedFromIndexSuspensionFrameL173C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut combiner = self.combiner.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut merged = self.merged.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_merge_nested_with_value(
                    session, combiner, grouped, index, merged, key, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeMergeFromIndexSuspensionFrameL250C25 {
    pub(super) combiner: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) merged: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeMergeFromIndexSuspensionFrameL250C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut combiner = self.combiner.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut merged = self.merged.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(combined) => (|| {
                resume_suspended_merge_with_value(
                    session, combiner, grouped, index, merged, key, combined,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedMergeFromIndexSuspensionFrameL299C25 {
    pub(super) combiner: rum_core::Expr,
    pub(super) grouped: crate::associations::suspended_state::FrozenAssociationSequence<(
        rum_core::Expr,
        std::vec::Vec<rum_core::Expr>,
    )>,
    pub(super) index: usize,
    pub(super) merged: crate::associations::suspended_state::PersistentAssociationSequence<(
        rum_core::Expr,
        rum_core::Expr,
    )>,
    pub(super) key: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedMergeFromIndexSuspensionFrameL299C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut combiner = self.combiner.clone();
        #[allow(unused_mut)]
        let mut grouped = self.grouped.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut merged = self.merged.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        let resumed = match outcome {
            Ok(combined) => (|| {
                resume_suspended_merge_with_value(
                    session, combiner, grouped, index, merged, key, combined,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeCountsByFromIndexSuspensionFrameL354C25 {
    pub(super) function: rum_core::Expr,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) counts: crate::associations::suspended_state::PersistentAssociationGroups<()>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeCountsByFromIndexSuspensionFrameL354C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut function = self.function.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut counts = self.counts.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_counts_by_with_value(session, function, items, index, counts, key)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedCountsByFromIndexSuspensionFrameL395C25 {
    pub(super) function: rum_core::Expr,
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) counts: crate::associations::suspended_state::PersistentAssociationGroups<()>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCountsByFromIndexSuspensionFrameL395C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut function = self.function.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut counts = self.counts.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_counts_by_with_value(session, function, items, index, counts, key)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
