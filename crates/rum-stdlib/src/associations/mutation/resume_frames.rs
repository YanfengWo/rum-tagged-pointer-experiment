use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct EvalAssociateToSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) target_expr: rum_core::Expr,
    pub(super) spec: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalAssociateToSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut target_expr = self.target_expr.clone();
        #[allow(unused_mut)]
        let mut spec = self.spec.clone();
        let resumed = match outcome {
            Ok(expr) => (|| finish_associate_to_value(session, args, target_expr, expr, spec))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct EvalKeyDropFromSuspensionFrame {
    pub(super) args: rum_core::SharedArgs,
    pub(super) target_expr: rum_core::Expr,
    pub(super) key_spec: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvalKeyDropFromSuspensionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut target_expr = self.target_expr.clone();
        #[allow(unused_mut)]
        let mut key_spec = self.key_spec.clone();
        let resumed = match outcome {
            Ok(expr) => {
                (|| finish_key_drop_from_value(session, args, target_expr, expr, key_spec))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
