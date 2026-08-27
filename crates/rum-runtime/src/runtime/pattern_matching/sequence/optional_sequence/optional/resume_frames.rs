use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct MatchOptionalChoicesSuspensionFrameL149C44 {
    pub(super) state: crate::runtime::pattern_matching::sequence::optional_sequence::optional::SuspendedOptionalChoiceState,
}

impl crate::runtime::suspension::SuspensionFrame for MatchOptionalChoicesSuspensionFrameL149C44 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareOptionalPresentFinalizer {
                    state: self.state.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareOptionalPresentFinalizer {
    state: SuspendedOptionalChoiceState,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOptionalPresentFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationOptionalPresent"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ContinueOptionalPresentFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueOptionalPresentFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args = optional_state_payload(payload, "InternalContinuationOptionalPresent");
            let state = OptionalChoiceState::from_continuation_expr(
                args.next()
                    .expect("optional-present continuation stores its state"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("optional-present continuation stores its match outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_optional_present(state, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct CollectOptionalMatchesSuspensionFrameL254C44 {
    pub(super) state: crate::runtime::pattern_matching::sequence::optional_sequence::optional::SuspendedOptionalChoiceState,
}

impl crate::runtime::suspension::SuspensionFrame for CollectOptionalMatchesSuspensionFrameL254C44 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareOptionalPresentCollectionFinalizer {
                    state: self.state.clone(),
                    present: Vec::<SymbolExprMap>::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareOptionalPresentCollectionFinalizer {
    state: SuspendedOptionalChoiceState,
    present: Vec<SymbolExprMap>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOptionalPresentCollectionFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationOptionalPresentCollection"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.present.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ContinueOptionalPresentCollectionFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueOptionalPresentCollectionFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args =
                optional_state_payload(payload, "InternalContinuationOptionalPresentCollection");
            let state = OptionalChoiceState::from_continuation_expr(
                args.next()
                    .expect("optional-present collection stores its state"),
            );
            let present = Vec::<SymbolExprMap>::from_continuation_expr(
                args.next()
                    .expect("optional-present collection stores its matches"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_optional_present_collection(state, present)
                .map(Vec::<SymbolExprMap>::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishOptionalPresentCollectionSuspensionFrameL293C44 {
    pub(super) present: crate::runtime::pattern_matching::suspended_state::PersistentMatchResults,
}

impl crate::runtime::suspension::SuspensionFrame
    for FinishOptionalPresentCollectionSuspensionFrameL293C44
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                FinishOptionalPresentCollectionFinalizer {
                    present: self.present.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct FinishOptionalPresentCollectionFinalizer {
    present: crate::runtime::pattern_matching::suspended_state::PersistentMatchResults,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for FinishOptionalPresentCollectionFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(
            finish_optional_collection(self.present.into_vec(), self.outcome)
                .into_continuation_expr(),
        )
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResolveOptionalOmissionSuspensionFrameL370C44 {
    pub(super) state: crate::runtime::pattern_matching::sequence::optional_sequence::optional::SuspendedOptionalOmissionState,
}

impl crate::runtime::suspension::SuspensionFrame for ResolveOptionalOmissionSuspensionFrameL370C44 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => {
                crate::runtime::suspension::FrameAction::finalize(PrepareOptionalDefaultFinalizer {
                    state: self.state.clone(),
                    default_expr: Option::<Expr>::from_continuation_expr(payload),
                })
            }
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareOptionalDefaultFinalizer {
    state: SuspendedOptionalOmissionState,
    default_expr: Option<Expr>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOptionalDefaultFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationOptionalDefault"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.default_expr.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ContinueOptionalDefaultFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueOptionalDefaultFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args = optional_state_payload(payload, "InternalContinuationOptionalDefault");
            let state = OptionalOmissionState::from_continuation_expr(
                args.next()
                    .expect("optional-default continuation stores its state"),
            );
            let default_expr = Option::<Expr>::from_continuation_expr(
                args.next()
                    .expect("optional-default continuation stores its value"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_optional_default(state, default_expr)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct FinishOptionalDefaultSuspensionFrameL411C44 {
    pub(super) state: crate::runtime::pattern_matching::sequence::optional_sequence::optional::SuspendedOptionalOmissionState,
}

impl crate::runtime::suspension::SuspensionFrame for FinishOptionalDefaultSuspensionFrameL411C44 {
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        match outcome {
            Ok(payload) => crate::runtime::suspension::FrameAction::finalize(
                PrepareOptionalDefaultMatchFinalizer {
                    state: self.state.clone(),
                    outcome: CompiledMatchOutcome::from_continuation_expr(payload),
                },
            ),
            Err(error) => crate::runtime::suspension::FrameAction::complete(Err(error)),
        }
    }
}

struct PrepareOptionalDefaultMatchFinalizer {
    state: SuspendedOptionalOmissionState,
    outcome: CompiledMatchOutcome,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareOptionalDefaultMatchFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("InternalContinuationOptionalDefaultMatch"),
            vec![
                self.state.into_owned().into_continuation_expr(),
                self.outcome.into_continuation_expr(),
            ],
        ))
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ContinueOptionalDefaultMatchFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueOptionalDefaultMatchFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let mut args =
                optional_state_payload(payload, "InternalContinuationOptionalDefaultMatch");
            let state = OptionalOmissionState::from_continuation_expr(
                args.next()
                    .expect("optional-default match continuation stores its state"),
            );
            let match_outcome = CompiledMatchOutcome::from_continuation_expr(
                args.next()
                    .expect("optional-default match continuation stores its outcome"),
            );
            debug_assert!(args.next().is_none());
            session
                .finish_optional_default_match(state, match_outcome)
                .map(CompiledMatchOutcome::into_continuation_expr)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

fn optional_state_payload(payload: Expr, tag: &'static str) -> std::vec::IntoIter<Expr> {
    let Expr::Call { head, args } = payload else {
        panic!("unexpected {tag} continuation payload")
    };
    assert!(
        is_system_symbol_name(&head, tag),
        "unexpected {tag} continuation tag: {head:?}"
    );
    args.into_iter()
}
