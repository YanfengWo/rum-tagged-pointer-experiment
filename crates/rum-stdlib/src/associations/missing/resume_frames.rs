use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDeleteMissingListAtTargetLevelSuspensionFrameL244C21 {
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingListAtTargetLevelSuspensionFrameL244C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_list_after_child(
                    session, items, level, scan, index, kept, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDeleteMissingListAtTargetLevelSuspensionFrameL278C21 {
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingListAtTargetLevelSuspensionFrameL278C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_list_after_child(
                    session, items, level, scan, index, kept, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrameL374C17 {
    pub(super) items:
        crate::associations::suspended_state::FrozenAssociationSequence<rum_core::Expr>,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept:
        crate::associations::suspended_state::PersistentAssociationSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingListAtTargetLevelSuspensionFrameL374C17
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_list_after_child(
                    session, items, level, scan, index, kept, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrameL448C21 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrameL448C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_predicate(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrameL502C21 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrameL502C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_child(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrameL537C21 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteMissingAssociationAtTargetLevelSuspensionFrameL537C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_child(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrameL644C21 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrameL644C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_predicate(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrameL712C17 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) level: usize,
    pub(super) scan: crate::associations::missing::DeleteMissingDepth,
    pub(super) index: usize,
    pub(super) kept: crate::associations::suspended_state::PersistentAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteMissingAssociationAtTargetLevelSuspensionFrameL712C17
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut level = self.level.clone();
        #[allow(unused_mut)]
        let mut scan = self.scan.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_missing_association_after_child(
                    session, entries, level, scan, index, kept, entry, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
