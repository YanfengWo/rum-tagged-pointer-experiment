use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalIfSuspensionFrameL50C13 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalIfSuspensionFrameL50C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(condition) => (|| session.resume_if_after_condition(&args, condition))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalSwitchSuspensionFrameL81C13 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for EvalSwitchSuspensionFrameL81C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = match outcome {
            Ok(subject) => (|| session.resume_switch_after_subject(&args, subject))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeWhileLoopSuspensionFrameL150C25 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeWhileLoopSuspensionFrameL150C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = (|| session.resume_while_loop(&args, WhilePhase::Test, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeWhileLoopSuspensionFrameL171C25 {
    pub(super) args: rum_core::SharedArgs,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeWhileLoopSuspensionFrameL171C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        let resumed = (|| session.resume_while_loop(&args, WhilePhase::Body, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeUntilLoopSuspensionFrameL227C17 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) phase: crate::runtime::kernel_data::control_data::branching_loops::UntilPhase,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeUntilLoopSuspensionFrameL227C17 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut phase = self.phase.clone();
        let resumed = (|| session.resume_until_loop(&args, phase, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeForLoopSuspensionFrameL284C17 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) phase: crate::runtime::kernel_data::control_data::branching_loops::ForPhase,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeForLoopSuspensionFrameL284C17 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut phase = self.phase.clone();
        let resumed = (|| session.resume_for_loop(&args, phase, Some(outcome)))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeCompoundExpressionFromIndexSuspensionFrameL337C13 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeCompoundExpressionFromIndexSuspensionFrameL337C13
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        let resumed = (|| session.resume_compound_expression_after_item(&args, index, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeWhichFromIndexSuspensionFrameL377C13 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) index: usize,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeWhichFromIndexSuspensionFrameL377C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        let resumed = match outcome {
            Ok(condition) => (|| session.resume_which_after_condition(&args, index, condition))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeSwitchFromIndexSuspensionFrameL431C44 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) subject: rum_core::Expr,
    pub(super) index: usize,
    pub(super) branch: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeSwitchFromIndexSuspensionFrameL431C44 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut subject = self.subject.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut branch = self.branch.clone();
        let resumed = (|| {
            let outcome = CompiledMatchOutcome::from_continuation_expr(outcome?);
            session.resume_switch_after_match(&args, subject, index, branch, outcome.matched)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
