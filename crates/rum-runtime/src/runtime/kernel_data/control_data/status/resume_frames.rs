use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FailureVirtualStyledMessageValueSuspensionFrameL201C44;

impl crate::runtime::suspension::SuspensionFrame
    for FailureVirtualStyledMessageValueSuspensionFrameL201C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = (|| match outcome {
            Ok(message) => Ok(styled_message_expr(message)),
            Err(error) => Err(error),
        })();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FailureMessageFromTemplateSuspensionFrameL232C17 {
    pub(super) status: rum_core::Expr,
    pub(super) template: rum_core::Expr,
}

impl crate::runtime::suspension::SuspensionFrame
    for FailureMessageFromTemplateSuspensionFrameL232C17
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut status = self.status.clone();
        #[allow(unused_mut)]
        let mut template = self.template.clone();
        let resumed = match outcome {
            Ok(param_value) => (|| {
                let params = failure_message_template_parameters_from_expr(&param_value);
                session.finish_failure_message_from_template(status, template, params)
            })(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishFailureMessageFromTemplateSuspensionFrameL270C64;

impl crate::runtime::suspension::SuspensionFrame
    for FinishFailureMessageFromTemplateSuspensionFrameL270C64
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = match outcome {
            Ok(resumed_value) => {
                let text = <std::string::String as crate::runtime::suspension::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> { Ok(Expr::string(text)) })()
                    .map(|value| <rum_core::Expr as crate::runtime::suspension::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
