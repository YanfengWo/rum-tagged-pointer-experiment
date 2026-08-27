use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeAssociationKeyEvaluationSuspensionFrameL56C25 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) evaluated_entries:
        crate::associations::suspended_state::PersistentAssociationSequence<
            rum_runtime::kernel_api::AssociationEntryForm,
        >,
    pub(super) index: usize,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeAssociationKeyEvaluationSuspensionFrameL56C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut evaluated_entries = self.evaluated_entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_association_key_evaluation_with_key(
                    session,
                    entries,
                    evaluated_entries,
                    index,
                    entry,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedAssociationKeyEvaluationSuspensionFrameL97C25 {
    pub(super) entries: crate::associations::suspended_state::FrozenAssociationSequence<
        rum_runtime::kernel_api::AssociationEntryForm,
    >,
    pub(super) evaluated_entries:
        crate::associations::suspended_state::PersistentAssociationSequence<
            rum_runtime::kernel_api::AssociationEntryForm,
        >,
    pub(super) index: usize,
    pub(super) entry: rum_runtime::kernel_api::AssociationEntryForm,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedAssociationKeyEvaluationSuspensionFrameL97C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut entries = self.entries.clone();
        #[allow(unused_mut)]
        let mut evaluated_entries = self.evaluated_entries.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut entry = self.entry.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_association_key_evaluation_with_key(
                    session,
                    entries,
                    evaluated_entries,
                    index,
                    entry,
                    key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct EvalLookupSuspensionFrameL180C9 {
    pub(super) head: rum_core::Expr,
    pub(super) key: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalLookupSuspensionFrameL180C9 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut key = self.key.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        let resumed = match outcome {
            Ok(expr) => (|| eval_lookup_with_evaluated_data(session, head, expr, key, call_args))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct EvalLookupWithEvaluatedDataSuspensionFrameL198C9 {
    pub(super) head: rum_core::Expr,
    pub(super) expr: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalLookupWithEvaluatedDataSuspensionFrameL198C9 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut head = self.head.clone();
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        #[allow(unused_mut)]
        let mut call_args = self.call_args.clone();
        let resumed = match outcome {
            Ok(key) => (|| finish_eval_lookup(session, head, expr, key, call_args))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct FinishEvalLookupSuspensionFrameL233C29 {
    pub(super) lookup: crate::associations::suspended_state::FrozenAssociationValue<
        crate::associations::basic::PreparedLookup,
    >,
    pub(super) wrapper: std::option::Option<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for FinishEvalLookupSuspensionFrameL233C29 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut lookup = self.lookup.clone();
        #[allow(unused_mut)]
        let mut wrapper = self.wrapper.clone();
        let resumed = match outcome {
            Ok(default) => {
                (|| finish_suspended_prepared_lookup(session, lookup, Some(default), wrapper))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct FinishPreparedLookupSuspensionFrameL271C29 {
    pub(super) lookup: crate::associations::suspended_state::FrozenAssociationValue<
        crate::associations::basic::PreparedLookup,
    >,
    pub(super) default: std::option::Option<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for FinishPreparedLookupSuspensionFrameL271C29 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut lookup = self.lookup.clone();
        #[allow(unused_mut)]
        let mut default = self.default.clone();
        let resumed = match outcome {
            Ok(wrapper) => (|| {
                Ok(lookup
                    .as_ref()
                    .outcome
                    .to_expr(default.as_ref(), Some(&wrapper)))
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct FinishSuspendedPreparedLookupSuspensionFrameL306C29 {
    pub(super) lookup: crate::associations::suspended_state::FrozenAssociationValue<
        crate::associations::basic::PreparedLookup,
    >,
    pub(super) default: std::option::Option<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for FinishSuspendedPreparedLookupSuspensionFrameL306C29
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut lookup = self.lookup.clone();
        #[allow(unused_mut)]
        let mut default = self.default.clone();
        let resumed = match outcome {
            Ok(wrapper) => (|| {
                Ok(lookup
                    .as_ref()
                    .outcome
                    .to_expr(default.as_ref(), Some(&wrapper)))
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
