use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishHeldUpdateSuspensionFrameL443C44 {
    pub(super) operator: &'static str,
    pub(super) target: crate::runtime::definitions::definition_updates::SuspendedHeldUpdateTarget,
    pub(super) return_value: crate::runtime::definitions::definition_updates::HeldUpdateReturn,
}

impl crate::runtime::suspension::SuspensionFrame for FinishHeldUpdateSuspensionFrameL443C44 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut operator = self.operator;
        #[allow(unused_mut)]
        let mut target = self.target.clone();
        #[allow(unused_mut)]
        let mut return_value = self.return_value.clone();
        let resumed = (|| {
            let updated = outcome?;
            session.store_suspended_held_update(operator, target, updated, return_value)
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
