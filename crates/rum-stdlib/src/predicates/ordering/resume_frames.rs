use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeOrderedQComparatorSuspensionFrameL149C25 {
    pub(super) state: crate::predicates::ordering::SuspendedOrderedQComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeOrderedQComparatorSuspensionFrameL149C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => {
                (|| resume_suspended_ordered_q_comparator_with_value(session, state, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedOrderedQComparatorSuspensionFrameL186C25 {
    pub(super) state: crate::predicates::ordering::SuspendedOrderedQComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedOrderedQComparatorSuspensionFrameL186C25
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
                (|| resume_suspended_ordered_q_comparator_with_value(session, state, value))()
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeLexicographicListComparatorSuspensionFrameL283C25 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeLexicographicListComparatorSuspensionFrameL283C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_forward_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeLexicographicListComparatorSuspensionFrameL315C25 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeLexicographicListComparatorSuspensionFrameL315C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_reverse_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedLexicographicListComparatorSuspensionFrameL351C25 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedLexicographicListComparatorSuspensionFrameL351C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_forward_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedLexicographicListComparatorSuspensionFrameL375C25 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedLexicographicListComparatorSuspensionFrameL375C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_reverse_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedLexicographicListComparatorWithForwardValueSuspensionFrameL412C21 {
    pub(super) state: crate::predicates::ordering::SuspendedLexicographicListComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedLexicographicListComparatorWithForwardValueSuspensionFrameL412C21
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_lexicographic_list_comparator_with_reverse_value(
                    session, state, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeLexicographicPairComparatorSuspensionFrameL447C21 {
    pub(super) state: crate::predicates::ordering::LexicographicPairComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeLexicographicPairComparatorSuspensionFrameL447C21
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
                (|| resume_lexicographic_pair_comparator_with_forward_value(session, state, value))(
                )
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeLexicographicPairComparatorWithForwardValueSuspensionFrameL476C21 {
    pub(super) state: crate::predicates::ordering::LexicographicPairComparatorState,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeLexicographicPairComparatorWithForwardValueSuspensionFrameL476C21
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
                (|| resume_lexicographic_pair_comparator_with_reverse_value(session, state, value))(
                )
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
