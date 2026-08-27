use super::*;
use crate::runtime::suspension::trace::{FrameRootTrace, FrameRootVisitor};

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeDoAfterSpecEvalSuspensionFrameL77C13 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) spec_index: usize,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeDoAfterSpecEvalSuspensionFrameL77C13 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        let resumed = match outcome {
            Ok(spec_expr) => (|| session.finish_do_spec_eval(&args, spec_index, spec_expr))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeDoIterationsSuspensionFrameL140C48 {
    pub(super) state: crate::runtime::kernel_data::control_data::branching_loops::iteration::driver::SuspendedDoIterations,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeDoIterationsSuspensionFrameL140C48 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_do_after_iteration(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeSuspendedDoIterationsSuspensionFrameL193C48 {
    pub(super) state: crate::runtime::kernel_data::control_data::branching_loops::iteration::driver::SuspendedDoIterations,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedDoIterationsSuspensionFrameL193C48
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_do_after_iteration(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeTableAfterSpecEvalSuspensionFrameL211C13 {
    pub(super) args: rum_core::SharedArgs,
    pub(super) spec_index: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeTableAfterSpecEvalSuspensionFrameL211C13
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut args = self.args.clone();
        #[allow(unused_mut)]
        let mut spec_index = self.spec_index.clone();
        let resumed = match outcome {
            Ok(spec_expr) => (|| session.finish_table_spec_eval(&args, spec_index, spec_expr))(),
            Err(error) => Err(error),
        };
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeTableIterationsSuspensionFrameL299C48 {
    pub(super) state: crate::runtime::kernel_data::control_data::branching_loops::iteration::driver::SuspendedTableIterations,
}

impl crate::runtime::suspension::SuspensionFrame for ResumeTableIterationsSuspensionFrameL299C48 {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_table_after_iteration(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ResumeSuspendedTableIterationsSuspensionFrameL360C48 {
    pub(super) state: crate::runtime::kernel_data::control_data::branching_loops::iteration::driver::SuspendedTableIterations,
}

impl crate::runtime::suspension::SuspensionFrame
    for ResumeSuspendedTableIterationsSuspensionFrameL360C48
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut state = self.state.clone();
        let resumed = (|| session.resume_suspended_table_after_iteration(state, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvaluateLoopIteratorBodySuspensionFrameL387C13 {
    pub(super) symbol: rum_core::Symbol,
    pub(super) saved_state: crate::runtime::kernel_data::control_data::branching_loops::iteration::driver::SuspendedLoopIteratorSymbolState,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateLoopIteratorBodySuspensionFrameL387C13
{
    fn resume(
        self: std::sync::Arc<Self>,
        _session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        crate::runtime::suspension::FrameAction::finalize(PrepareLoopIteratorScopeFinalizer {
            symbol: self.symbol,
            saved_state: self.saved_state.clone(),
            outcome,
        })
    }
}

struct PrepareLoopIteratorScopeFinalizer {
    symbol: Symbol,
    saved_state: SuspendedLoopIteratorSymbolState,
    outcome: Result<Expr, EvalError>,
}

impl crate::runtime::suspension::PureFrameFinalizer for PrepareLoopIteratorScopeFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError> {
        Ok(self
            .saved_state
            .into_continuation_expr(self.symbol, self.outcome))
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct ContinueLoopIteratorScopeFrame;

impl crate::runtime::suspension::SuspensionFrame for ContinueLoopIteratorScopeFrame {
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        let resumed = outcome.and_then(|payload| {
            let (symbol, saved_state, body_outcome) =
                SuspendedLoopIteratorSymbolState::from_continuation_expr(payload);
            session.finish_loop_iterator_scope(symbol, saved_state, body_outcome)
        });
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}

#[derive(rum_derive::FrameRootTrace)]
pub(super) struct EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrameL407C13 {
    pub(super) symbol: rum_core::Symbol,
    pub(super) dynamic_depth: usize,
}

impl crate::runtime::suspension::SuspensionFrame
    for EvaluateLoopIteratorBodyWithDynamicValueSuspensionFrameL407C13
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> crate::runtime::suspension::FrameAction {
        #[allow(unused_mut)]
        let mut symbol = self.symbol.clone();
        #[allow(unused_mut)]
        let mut dynamic_depth = self.dynamic_depth.clone();
        let resumed =
            (|| session.finish_loop_iterator_dynamic_scope(symbol, dynamic_depth, outcome))();
        crate::runtime::suspension::FrameAction::complete(resumed)
    }
}
