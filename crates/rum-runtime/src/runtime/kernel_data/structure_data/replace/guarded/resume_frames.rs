use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct PlanGuardedReplacePartTargetsOwnedSuspensionFrameL101C56 {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPlanning,
}

impl crate::runtime::suspension::SuspensionFrame
    for PlanGuardedReplacePartTargetsOwnedSuspensionFrameL101C56
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed =
            (|| session.plan_guarded_replace_part_pattern_targets_suspended(state, value?))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct MatchGuardedReplacePartPatternTargetsOwnedSuspensionFrameL205C48 {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPatternMatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for MatchGuardedReplacePartPatternTargetsOwnedSuspensionFrameL205C48
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
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session.match_guarded_replace_part_pattern_targets_suspended(state, Some(outcome))
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct SelectGuardedReplacePartTargetsOwnedSuspensionFrameL284C48 {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartSelection,
}

impl crate::runtime::suspension::SuspensionFrame
    for SelectGuardedReplacePartTargetsOwnedSuspensionFrameL284C48
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
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session.select_guarded_replace_part_targets_suspended(state, Some(outcome.matched))
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct PlanGuardedReplacePartTargetsSuspendedSuspensionFrameL342C56 {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPlanning,
}

impl crate::runtime::suspension::SuspensionFrame
    for PlanGuardedReplacePartTargetsSuspendedSuspensionFrameL342C56
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed =
            (|| session.plan_guarded_replace_part_pattern_targets_suspended(state, value?))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct MatchGuardedReplacePartPatternTargetsSuspendedSuspensionFrameL420C48 {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartPatternMatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for MatchGuardedReplacePartPatternTargetsSuspendedSuspensionFrameL420C48
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
            let outcome = CompiledMatchOutcome::from_continuation_expr(value?);
            session.match_guarded_replace_part_pattern_targets_suspended(state, Some(outcome))
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct SelectGuardedReplacePartTargetsSuspendedSuspensionFrameL492C48 {
    pub(super) state: crate::runtime::kernel_data::structure_data::replace::guarded::SuspendedGuardedReplacePartSelection,
}

impl crate::runtime::suspension::SuspensionFrame
    for SelectGuardedReplacePartTargetsSuspendedSuspensionFrameL492C48
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
            let matched = bool::from_continuation_expr(value?);
            session.select_guarded_replace_part_targets_suspended(state, Some(matched))
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
