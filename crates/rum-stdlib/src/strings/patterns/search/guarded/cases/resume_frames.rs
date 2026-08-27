use super::*;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeCollectStringCasesFromCandidateSuspensionFrameL122C25 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) rule_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
    pub(super) bindings: crate::strings::suspended_state::FrozenStringValue<
        std::collections::HashMap<
            rum_core::Symbol,
            rum_core::Expr,
            std::hash::BuildHasherDefault<rum_runtime::RuntimeCacheHasher>,
        >,
    >,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeCollectStringCasesFromCandidateSuspensionFrameL122C25
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
        let mut rule_index = self.rule_index.clone();
        #[allow(unused_mut)]
        let mut matches = self.matches.clone();
        #[allow(unused_mut)]
        let mut bindings = self.bindings.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let accepted =
                    <bool as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(
                        resumed_value,
                    );
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                            resume_suspended_collect_string_cases_after_guard(
                                session, state, index, end, rule_index, matches, bindings, accepted,
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
pub(super) struct EvaluateStringCasesOutputSuspensionFrameL189C17 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame for EvaluateStringCasesOutputSuspensionFrameL189C17 {
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
        let mut matches = self.matches.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                    resume_suspended_collect_string_cases_with_output(
                        session, state, index, end, matches, output,
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
pub(super) struct ResumeSuspendedCollectStringCasesFromCandidateSuspensionFrameL251C25 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) rule_index: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
    pub(super) bindings: crate::strings::suspended_state::FrozenStringValue<
        std::collections::HashMap<
            rum_core::Symbol,
            rum_core::Expr,
            std::hash::BuildHasherDefault<rum_runtime::RuntimeCacheHasher>,
        >,
    >,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCollectStringCasesFromCandidateSuspensionFrameL251C25
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
        let mut rule_index = self.rule_index.clone();
        #[allow(unused_mut)]
        let mut matches = self.matches.clone();
        #[allow(unused_mut)]
        let mut bindings = self.bindings.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let accepted =
                    <bool as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(
                        resumed_value,
                    );
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                            resume_suspended_collect_string_cases_after_guard(
                                session, state, index, end, rule_index, matches, bindings, accepted,
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
pub(super) struct EvaluateSuspendedStringCasesOutputSuspensionFrameL347C17 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesRuleEvalState,
    >,
    pub(super) index: usize,
    pub(super) end: usize,
    pub(super) matches: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for EvaluateSuspendedStringCasesOutputSuspensionFrameL347C17
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
        let mut matches = self.matches.clone();
        let resumed = match outcome {
            Ok(resumed_value) => {
                let output = <rum_core::Expr as rum_runtime::kernel_api::ContinuationValue>::from_continuation_expr(resumed_value);
                (|| -> Result<Vec<rum_core::Expr>, EvalError> {
                    resume_suspended_collect_string_cases_with_output(
                        session, state, index, end, matches, output,
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
pub(super) struct ResumeCollectStringCasesValuesFromIndexSuspensionFrameL423C21 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeCollectStringCasesValuesFromIndexSuspensionFrameL423C21
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
pub(super) struct ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrameL461C21 {
    pub(super) state: crate::strings::suspended_state::FrozenStringValue<
        crate::strings::patterns::search::guarded::cases::GuardedStringCasesValuesEvalState,
    >,
    pub(super) index: usize,
    pub(super) outputs: crate::strings::suspended_state::PersistentStringSequence<rum_core::Expr>,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedCollectStringCasesValuesFromIndexSuspensionFrameL461C21
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
