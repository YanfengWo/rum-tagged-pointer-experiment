use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDuplicateFreeSameTestSuspensionFrameL412C25 {
    pub(super) state: crate::predicates::collections::SuspendedDuplicateFreeSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDuplicateFreeSameTestSuspensionFrameL412C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_duplicate_free_same_test_with_value(session, state, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedDuplicateFreeSameTestSuspensionFrameL456C25 {
    pub(super) state: crate::predicates::collections::SuspendedDuplicateFreeSameTestState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDuplicateFreeSameTestSuspensionFrameL456C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_duplicate_free_same_test_with_value(session, state, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSameTestMatrixSuspensionFrameL547C25 {
    pub(super) state: crate::predicates::collections::SuspendedSameTestMatrixState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeSameTestMatrixSuspensionFrameL547C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_same_test_matrix_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedSameTestMatrixSuspensionFrameL595C25 {
    pub(super) state: crate::predicates::collections::SuspendedSameTestMatrixState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedSameTestMatrixSuspensionFrameL595C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| resume_suspended_same_test_matrix_with_value(session, state, value))(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
