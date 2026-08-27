use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvalCatchExceptionsSuspensionFrameL177C13 {
    pub(super) handlers: std::option::Option<
        std::sync::Arc<[crate::runtime::kernel_data::control_data::exceptions::ExceptionHandler]>,
    >,
}

impl crate::runtime::suspension::SuspensionFrame for EvalCatchExceptionsSuspensionFrameL177C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut handlers = self.handlers.clone();
        let resumed =
            (|| session.resume_suspended_catch_exceptions_after_body(handlers, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct DispatchExceptionHandlersOwnedSuspensionFrameL314C56 {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for DispatchExceptionHandlersOwnedSuspensionFrameL314C56
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_exception_guard(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct DispatchExceptionHandlersOwnedSuspensionFrameL331C48 {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for DispatchExceptionHandlersOwnedSuspensionFrameL331C48
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_exception_match(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct DispatchSuspendedExceptionHandlersSuspensionFrameL368C48 {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for DispatchSuspendedExceptionHandlersSuspensionFrameL368C48
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_exception_match(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeSuspendedExceptionMatchOutcomeSuspensionFrameL412C44 {
    pub(super) state:
        crate::runtime::kernel_data::control_data::exceptions::SuspendedExceptionDispatch,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedExceptionMatchOutcomeSuspensionFrameL412C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_exception_guard(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvaluateExceptionHandlerRhsSuspensionFrameL455C53;

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateExceptionHandlerRhsSuspensionFrameL455C53
{
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
