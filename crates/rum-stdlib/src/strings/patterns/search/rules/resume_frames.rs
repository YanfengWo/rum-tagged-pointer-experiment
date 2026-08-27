use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct SuspendReplaceStringOutputSuspensionFrameL305C66 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) replacements: usize,
    pub(super) literal_match_index: usize,
    pub(super) output: crate::strings::suspended_state::PersistentStringText,
}

impl rum_runtime::kernel_api::SuspensionFrame for SuspendReplaceStringOutputSuspensionFrameL305C66 {
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
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut replacements = self.replacements.clone();
        #[allow(unused_mut)]
        let mut literal_match_index = self.literal_match_index.clone();
        #[allow(unused_mut)]
        let mut output = self.output.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let rendered_expr = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<std::string::String, EvalError> {
        resume_suspended_replace_string_value_with_output(
            session,
            state,
            index,
            end,
            replacements,
            literal_match_index,
            output,
            rendered_expr,
        )
    })()
                    .map(|value| <std::string::String as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct DriveSuspendedReplaceStringOutputResultSuspensionFrameL441C17 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) replacements: usize,
    pub(super) literal_match_index: usize,
    pub(super) output: crate::strings::suspended_state::PersistentStringText,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for DriveSuspendedReplaceStringOutputResultSuspensionFrameL441C17
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
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut replacements = self.replacements.clone();
        #[allow(unused_mut)]
        let mut literal_match_index = self.literal_match_index.clone();
        #[allow(unused_mut)]
        let mut output = self.output.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let rendered_expr = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<std::string::String, EvalError> {
                    resume_suspended_replace_string_value_with_output(
                        session,
                        state,
                        index,
                        end,
                        replacements,
                        literal_match_index,
                        output,
                        rendered_expr,
                    )
                })()
                    .map(|value| <std::string::String as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeReplaceStringValuesFromIndexSuspensionFrameL554C21 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) rendered:
        crate::strings::suspended_state::PersistentStringSequence<std::string::String>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeReplaceStringValuesFromIndexSuspensionFrameL554C21
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
        let mut rendered = self.rendered.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let value = <std::string::String as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                        resume_suspended_replace_string_values_with_value(
                            session, state, index, rendered, value,
                        )
                    })()
                    .map(|value| <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrameL593C21 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringReplaceValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) rendered:
        crate::strings::suspended_state::PersistentStringSequence<std::string::String>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedReplaceStringValuesFromIndexSuspensionFrameL593C21
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
        let mut rendered = self.rendered.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let value = <std::string::String as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                        resume_suspended_replace_string_values_with_value(
                            session, state, index, rendered, value,
                        )
                    })()
                    .map(|value| <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct DriveStringCasesOutputResultSuspensionFrameL750C17 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) literal_match_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for DriveStringCasesOutputResultSuspensionFrameL750C17
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
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut literal_match_index = self.literal_match_index.clone();
        #[allow(unused_mut)]
        let mut matches = self.matches.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                    resume_suspended_collect_string_cases_with_output(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output,
                    )
                })()
                    .map(|value| <Vec<rum_core::Expr> as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct DriveSuspendedStringCasesOutputResultSuspensionFrameL907C17 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) literal_match_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for DriveSuspendedStringCasesOutputResultSuspensionFrameL907C17
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
        let mut end = self.end.clone();
        #[allow(unused_mut)]
        let mut literal_match_index = self.literal_match_index.clone();
        #[allow(unused_mut)]
        let mut matches = self.matches.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                    resume_suspended_collect_string_cases_with_output(
                        session,
                        state,
                        index,
                        end,
                        literal_match_index,
                        matches,
                        output,
                    )
                })()
                    .map(|value| <Vec<rum_core::Expr> as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeCollectStringCasesValuesFromIndexSuspensionFrameL1001C21 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeCollectStringCasesValuesFromIndexSuspensionFrameL1001C21
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
            Ok(resumed_value) => {
                let items = <Vec<rum_core::Expr> as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                        resume_suspended_collect_string_cases_values_with_value(
                            session, state, index, outputs, items,
                        )
                    })()
                    .map(|value| <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrameL1040C21 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::rules::StringCasesValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrameL1040C21
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
            Ok(resumed_value) => {
                let items = <Vec<rum_core::Expr> as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<rum_core::Expr, EvalError> {
                        resume_suspended_collect_string_cases_values_with_value(
                            session, state, index, outputs, items,
                        )
                    })()
                    .map(|value| <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::into_continuation_expr(value))
            }
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
