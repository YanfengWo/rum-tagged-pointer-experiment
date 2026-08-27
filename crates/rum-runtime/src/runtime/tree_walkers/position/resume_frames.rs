use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalPositionSuspensionFrameL113C13 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalPositionSuspensionFrameL113C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_position_after_target(target, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumePositionAfterTargetSuspensionFrameL128C13 {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumePositionAfterTargetSuspensionFrameL128C13
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(pattern) => (|| session.resume_position_after_pattern(target, pattern, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumePositionAfterPatternSuspensionFrameL149C44 {
    pub(super) target: rum_core::Expr,
    pub(super) pattern: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumePositionAfterPatternSuspensionFrameL149C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut pattern = self.pattern.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = (|| {
            let traversal = CasesTraversalResolution::from_continuation_expr(outcome?);
            session.finish_position_after_resolved_traversal(target, pattern, traversal, &args)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishPositionAfterResolvedTraversalSuspensionFrameL180C53;

impl crate::runtime::suspension::SuspensionFrame
    for FinishPositionAfterResolvedTraversalSuspensionFrameL180C53
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <crate::runtime::PositionCollectionOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(Expr::call(system_expr("List"), outcome.positions))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishPositionsAtCurrentNodeSuspensionFrameL344C44 {
    pub(super) state: crate::runtime::tree_walkers::position::SuspendedPositionNode,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishPositionsAtCurrentNodeSuspensionFrameL344C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            session
                .finish_suspended_position_node(state, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct CollectPositionChildrenSuspensionFrameL461C48 {
    pub(super) state: crate::runtime::tree_walkers::position::SuspendedPositionChildren,
}

impl crate::runtime::suspension::SuspensionFrame for CollectPositionChildrenSuspensionFrameL461C48 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = PositionCollectionOutcome::from_continuation_expr(value?);
            session
                .collect_position_children_suspended(state, Some(outcome))
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct CollectPositionChildrenSuspendedSuspensionFrameL532C48 {
    pub(super) state: crate::runtime::tree_walkers::position::SuspendedPositionChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for CollectPositionChildrenSuspendedSuspensionFrameL532C48
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = PositionCollectionOutcome::from_continuation_expr(value?);
            session
                .collect_position_children_suspended(state, Some(outcome))
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishPositionsAtCurrentNodeSuspendedSuspensionFrameL574C44 {
    pub(super) state: crate::runtime::tree_walkers::position::SuspendedPositionNode,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishPositionsAtCurrentNodeSuspendedSuspensionFrameL574C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| {
            session
                .finish_suspended_position_node(state, outcome)
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
