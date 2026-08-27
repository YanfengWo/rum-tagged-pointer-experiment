use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalCountSuspensionFrameL48C13 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCountSuspensionFrameL48C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_count_after_target(target, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalMemberQSuspensionFrameL69C13 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalMemberQSuspensionFrameL69C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_member_q_after_target(target, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalFreeQSuspensionFrameL87C13 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalFreeQSuspensionFrameL87C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(target) => (|| session.resume_free_q_after_target(target, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCountAfterTargetSuspensionFrameL102C13 {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeCountAfterTargetSuspensionFrameL102C13 {
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
            Ok(pattern) => (|| session.resume_count_after_pattern(target, pattern, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeMemberQAfterTargetSuspensionFrameL137C13 {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeMemberQAfterTargetSuspensionFrameL137C13
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
            Ok(pattern) => (|| session.resume_member_q_after_pattern(target, pattern, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeFreeQAfterTargetSuspensionFrameL172C13 {
    pub(super) target: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeFreeQAfterTargetSuspensionFrameL172C13 {
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
            Ok(pattern) => (|| session.resume_free_q_after_pattern(target, pattern, &args))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishCountLikeAfterPatternSuspensionFrameL220C44 {
    pub(super) builtin_name: &'static str,
    pub(super) target: rum_core::Expr,
    pub(super) pattern: rum_core::Expr,
    pub(super) args: rum_core::SharedArgs,
    pub(super) finish: crate::runtime::tree_walkers::count_like::CountLikeFinish,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishCountLikeAfterPatternSuspensionFrameL220C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut builtin_name = self.builtin_name;
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut pattern = self.pattern.clone();
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut finish = self.finish.clone();
        let resumed = (|| {
            let traversal = TraversalWithoutLimitResolution::from_continuation_expr(outcome?);
            session.finish_count_like_after_traversal(
                builtin_name,
                target,
                pattern,
                &args,
                finish,
                traversal,
            )
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishCountLikeAfterTraversalSuspensionFrameL259C51 {
    pub(super) finish: crate::runtime::tree_walkers::count_like::CountLikeFinish,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishCountLikeAfterTraversalSuspensionFrameL259C51
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut finish = self.finish.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <crate::runtime::TraversalCountOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
            Ok(finish.finish(outcome.count))
        })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishCountAtCurrentNodeSuspensionFrameL386C49 {
    pub(super) count: usize,
    pub(super) traversal: crate::runtime::CasesTraversalConfig,
    pub(super) seen_before: usize,
    pub(super) depth: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishCountAtCurrentNodeSuspensionFrameL386C49
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut count = self.count.clone();
        #[allow(unused_mut)]
        let mut traversal = self.traversal.clone();
        #[allow(unused_mut)]
        let mut seen_before = self.seen_before.clone();
        #[allow(unused_mut)]
        let mut depth = self.depth.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let outcome = <CompiledMatchOutcome as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<crate::runtime::TraversalCountOutcome, EvalError> {
            let next_count = count + usize::from(outcome.matched);
            let exhausted = traversal.limit_reached(seen_before + next_count);
            Ok(TraversalCountOutcome::new(next_count, depth, exhausted))
        })()
                    .map(|value| <crate::runtime::TraversalCountOutcome as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct CountPatternChildrenSuspensionFrameL463C48 {
    pub(super) state: crate::runtime::tree_walkers::count_like::SuspendedCountChildren,
}

impl crate::runtime::suspension::SuspensionFrame for CountPatternChildrenSuspensionFrameL463C48 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let value = outcome;
        let resumed = (|| {
            let outcome = TraversalCountOutcome::from_continuation_expr(value?);
            session
                .count_pattern_children_suspended(state, Some(outcome))
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct CountPatternChildrenSuspendedSuspensionFrameL528C48 {
    pub(super) state: crate::runtime::tree_walkers::count_like::SuspendedCountChildren,
}

impl crate::runtime::suspension::SuspensionFrame
    for CountPatternChildrenSuspendedSuspensionFrameL528C48
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
            let outcome = TraversalCountOutcome::from_continuation_expr(value?);
            session
                .count_pattern_children_suspended(state, Some(outcome))
                .map(|outcome| outcome.into_continuation_expr())
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
