use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeTensorCallbackFromIndexSuspensionFrameL292C25 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedTensorCallbackEvalState,
    pub(super) index: usize,
    pub(super) outputs:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeTensorCallbackFromIndexSuspensionFrameL292C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut outputs = self.outputs.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_tensor_callback_with_value(session, state, index, outputs, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedTensorCallbackFromIndexSuspensionFrameL329C25 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedTensorCallbackEvalState,
    pub(super) index: usize,
    pub(super) outputs:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedTensorCallbackFromIndexSuspensionFrameL329C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut outputs = self.outputs.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_tensor_callback_with_value(session, state, index, outputs, value)
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeInnerFromIndicesSuspensionFrameL426C29 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedInnerEvalState,
    pub(super) output_index: usize,
    pub(super) contract_index: usize,
    pub(super) terms:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
    pub(super) output_scalars:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeInnerFromIndicesSuspensionFrameL426C29 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut output_index = self.output_index.clone();
        #[allow(unused_mut)]
        let mut contract_index = self.contract_index.clone();
        #[allow(unused_mut)]
        let mut terms = self.terms.clone();
        #[allow(unused_mut)]
        let mut output_scalars = self.output_scalars.clone();
        let resumed = match outcome {
            Ok(term) => (|| {
                resume_suspended_inner_term_with_value(
                    session,
                    state,
                    output_index,
                    contract_index,
                    terms,
                    output_scalars,
                    term,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeInnerFromIndicesSuspensionFrameL455C25 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedInnerEvalState,
    pub(super) output_index: usize,
    pub(super) output_scalars:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for ResumeInnerFromIndicesSuspensionFrameL455C25 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut output_index = self.output_index.clone();
        #[allow(unused_mut)]
        let mut output_scalars = self.output_scalars.clone();
        let resumed = match outcome {
            Ok(reduced) => (|| {
                resume_suspended_inner_reduced_with_value(
                    session,
                    state,
                    output_index,
                    output_scalars,
                    reduced,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedInnerFromIndicesSuspensionFrameL540C29 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedInnerEvalState,
    pub(super) output_index: usize,
    pub(super) contract_index: usize,
    pub(super) terms:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
    pub(super) output_scalars:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedInnerFromIndicesSuspensionFrameL540C29
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut output_index = self.output_index.clone();
        #[allow(unused_mut)]
        let mut contract_index = self.contract_index.clone();
        #[allow(unused_mut)]
        let mut terms = self.terms.clone();
        #[allow(unused_mut)]
        let mut output_scalars = self.output_scalars.clone();
        let resumed = match outcome {
            Ok(term) => (|| {
                resume_suspended_inner_term_with_value(
                    session,
                    state,
                    output_index,
                    contract_index,
                    terms,
                    output_scalars,
                    term,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedInnerFromIndicesSuspensionFrameL567C25 {
    pub(super) state: crate::functional_ops::arrays::tensor::SuspendedInnerEvalState,
    pub(super) output_index: usize,
    pub(super) output_scalars:
        crate::functional_ops::suspended_state::PersistentFunctionalSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedInnerFromIndicesSuspensionFrameL567C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        #[allow(unused_mut)]
        let mut output_index = self.output_index.clone();
        #[allow(unused_mut)]
        let mut output_scalars = self.output_scalars.clone();
        let resumed = match outcome {
            Ok(reduced) => (|| {
                resume_suspended_inner_reduced_with_value(
                    session,
                    state,
                    output_index,
                    output_scalars,
                    reduced,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
