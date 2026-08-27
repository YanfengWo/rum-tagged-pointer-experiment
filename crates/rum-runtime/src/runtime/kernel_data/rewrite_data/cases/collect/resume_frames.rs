use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalCasesSuspensionFrameL43C13 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCasesSuspensionFrameL43C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(target) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareCasesFinalizer {
                    target,
                    context_target: None,
                    args: Some(self.args.clone()),
                    call_args: None,
                    kind: CasesContinuationKind::Target,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCasesAfterTargetSuspensionFrameL79C44 {
    pub(super) target: rum_core::Expr,
    pub(super) call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeCasesAfterTargetSuspensionFrameL79C44 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(resolved_spec) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareCasesFinalizer {
                    target: resolved_spec,
                    context_target: Some(self.target.clone()),
                    args: None,
                    call_args: Some(self.call_args.clone()),
                    kind: CasesContinuationKind::Spec,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(Clone, Copy)]
enum CasesContinuationKind {
    Target,
    Spec,
}

struct PrepareCasesFinalizer {
    target: Expr,
    context_target: Option<Expr>,
    args: Option<SharedArgs>,
    call_args: Option<SharedArgs>,
    kind: CasesContinuationKind,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareCasesFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        match self.kind {
            CasesContinuationKind::Target => Ok(Expr::call(
                system_expr("InternalContinuationCasesTarget"),
                vec![
                    self.target,
                    self.args
                        .expect("Cases target continuation stores call arguments")
                        .into_vec()
                        .into_continuation_expr(),
                ],
            )),
            CasesContinuationKind::Spec => Ok(Expr::call(
                system_expr("InternalContinuationCasesSpec"),
                vec![
                    self.context_target
                        .expect("Cases spec continuation stores its target"),
                    self.target,
                    self.call_args
                        .expect("Cases spec continuation stores call arguments")
                        .into_vec()
                        .into_continuation_expr(),
                ],
            )),
        }
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ContinueCasesFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueCasesFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let Expr::Call { head, args } = payload else {
                panic!("unexpected Cases continuation payload")
            };
            let mut args = args.into_iter();
            if is_system_symbol_name(&head, "InternalContinuationCasesTarget") {
                let target = args
                    .next()
                    .expect("Cases target continuation stores target");
                let call_args = Vec::<Expr>::from_continuation_expr(
                    args.next()
                        .expect("Cases target continuation stores call arguments"),
                );
                debug_assert!(args.next().is_none());
                session.resume_cases_after_target_suspended(target, SharedArgs::new(call_args))
            } else if is_system_symbol_name(&head, "InternalContinuationCasesSpec") {
                let target = args.next().expect("Cases spec continuation stores target");
                let resolved_spec = args
                    .next()
                    .expect("Cases spec continuation stores resolved spec");
                let call_args = Vec::<Expr>::from_continuation_expr(
                    args.next()
                        .expect("Cases spec continuation stores call arguments"),
                );
                debug_assert!(args.next().is_none());
                session.resume_cases_after_resolved_spec_suspended(
                    target,
                    resolved_spec,
                    SharedArgs::new(call_args),
                )
            } else {
                panic!("unexpected Cases continuation tag: {head:?}")
            }
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCasesAfterResolvedSpecSuspensionFrameL111C44 {
    pub(super) target: rum_core::Expr,
    pub(super) spec: crate::runtime::kernel_data::rewrite_data::suspended_state::FrozenRewriteValue<
        crate::runtime::CasesSpec,
    >,
    pub(super) call_args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeCasesAfterResolvedSpecSuspensionFrameL111C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        let spec = self.spec.clone();
        let call_args = self.call_args.clone();
        let value = outcome;
        let resumed = (|| {
            let traversal = CasesTraversalResolution::from_continuation_expr(value?);
            session
                .finish_cases_after_resolved_traversal_suspended(target, spec, traversal, call_args)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishCasesWithTraversalSuspensionFrameL154C51;

impl crate::runtime::suspension::SuspensionFrame
    for FinishCasesWithTraversalSuspensionFrameL154C51
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <crate::runtime::CasesCollectionOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(Expr::call(system_expr("List"), outcome.matches))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishCasesAtCurrentNodeSuspensionFrameL282C44 {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesNodeFinish,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishCasesAtCurrentNodeSuspensionFrameL282C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let case = match outcome {
            Ok(value) => Option::<Expr>::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        crate::runtime::suspension::FrameAction::finalize(FinishCasesCollectionFinalizer {
            outcome: finish_suspended_cases_node(self.state.clone(), case),
        })
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct CollectCasesChildrenSuspensionFrameL366C48 {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesChildren,
}

impl crate::runtime::suspension::SuspensionFrame for CollectCasesChildrenSuspensionFrameL366C48 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let outcome = match value {
            Ok(value) => CasesCollectionOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        match session.resume_suspended_cases_children(state, outcome) {
            Ok(outcome) => {
                crate::runtime::suspension::FrameAction::finalize(FinishCasesCollectionFinalizer {
                    outcome,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct CollectCasesChildrenSuspendedSuspensionFrameL437C48 {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectCasesChildrenSuspendedSuspensionFrameL437C48
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let outcome = match value {
            Ok(value) => CasesCollectionOutcome::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        match session.resume_suspended_cases_children(state, outcome) {
            Ok(outcome) => {
                crate::runtime::suspension::FrameAction::finalize(FinishCasesCollectionFinalizer {
                    outcome,
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishCasesAtCurrentNodeSuspendedSuspensionFrameL588C44 {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesNodeFinish,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishCasesAtCurrentNodeSuspendedSuspensionFrameL588C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let case = match outcome {
            Ok(value) => Option::<Expr>::from_continuation_expr(value),
            Err(error) => return crate::runtime::suspension::FrameAction::complete(Err(error)),
        };
        crate::runtime::suspension::FrameAction::finalize(FinishCasesCollectionFinalizer {
            outcome: finish_suspended_cases_node(self.state.clone(), case),
        })
    }
}

struct FinishCasesCollectionFinalizer {
    outcome: SuspendedCasesCollectionOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishCasesCollectionFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(self.outcome.into_owned().into_continuation_expr())
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct TryCollectCaseOwnedSuspensionFrameL623C52 {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesRuleMatch,
}

impl crate::runtime::suspension::SuspensionFrame for TryCollectCaseOwnedSuspensionFrameL623C52 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .finish_suspended_cases_rule_match(state, outcome)
                .map(|result| result.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct TryCollectCaseSuspendedSuspensionFrameL659C52 {
    pub(super) state:
        crate::runtime::kernel_data::rewrite_data::cases::collect::SuspendedCasesRuleMatch,
}

impl crate::runtime::suspension::SuspensionFrame for TryCollectCaseSuspendedSuspensionFrameL659C52 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session
                .finish_suspended_cases_rule_match(state, outcome)
                .map(|result| result.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishCasesPatternMatchSuspensionFrameL681C44 {
    pub(super) expr: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for FinishCasesPatternMatchSuspensionFrameL681C44 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut expr = self.expr.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            Ok(outcome.matched.then_some(expr).into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishCasesRuleMatchSuspensionFrameL717C61;

impl crate::runtime::suspension::SuspensionFrame for FinishCasesRuleMatchSuspensionFrameL717C61 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(value) => (|| Ok(value))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
