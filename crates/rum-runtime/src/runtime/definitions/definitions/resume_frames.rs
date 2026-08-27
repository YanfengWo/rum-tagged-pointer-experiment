use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalSetSuspensionFrameL36C17 {
    pub(super) lhs: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalSetSuspensionFrameL36C17 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => (|| session.resume_set_with_rhs(lhs, rhs))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalRuleSuspensionFrameL76C13 {
    pub(super) lhs: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame for EvalRuleSuspensionFrameL76C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut lhs = self.lhs.clone();
        let resumed = match outcome {
            Ok(rhs) => (|| session.resume_rule_with_rhs(lhs, rhs))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
