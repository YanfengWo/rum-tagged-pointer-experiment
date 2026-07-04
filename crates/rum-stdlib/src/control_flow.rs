use rum_core::Expr;
use rum_runtime::kernel_api::{ControlDataBuiltin, EvalError, Session};

fn dispatch(
    session: &mut Session,
    builtin: ControlDataBuiltin,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    session.eval_control_data_builtin(builtin, head, args)
}

pub(crate) fn eval_compound_expression(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::CompoundExpression, head, args)
}

pub(crate) fn eval_if(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::If, head, args)
}

pub(crate) fn eval_which(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Which, head, args)
}

pub(crate) fn eval_switch(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Switch, head, args)
}

pub(crate) fn eval_do(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Do, head, args)
}

pub(crate) fn eval_product(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Product, head, args)
}

pub(crate) fn eval_sum(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Sum, head, args)
}

pub(crate) fn eval_while(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::While, head, args)
}

pub(crate) fn eval_until(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Until, head, args)
}

pub(crate) fn eval_for(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::For, head, args)
}

pub(crate) fn eval_table(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Table, head, args)
}

pub(crate) fn eval_abort(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Abort, head, args)
}

pub(crate) fn eval_exit(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Exit, head, args)
}

pub(crate) fn eval_check_abort(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::CheckAbort, head, args)
}

pub(crate) fn eval_abort_protect(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::AbortProtect, head, args)
}

pub(crate) fn eval_with_cleanup(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::WithCleanup, head, args)
}

pub(crate) fn eval_failsafe(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Failsafe, head, args)
}

pub(crate) fn eval_assert(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Assert, head, args)
}

pub(crate) fn eval_enclose(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Enclose, head, args)
}

pub(crate) fn eval_confirm(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Confirm, head, args)
}

pub(crate) fn eval_confirm_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ConfirmBy, head, args)
}

pub(crate) fn eval_confirm_match(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ConfirmMatch, head, args)
}

pub(crate) fn eval_confirm_quiet(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ConfirmQuiet, head, args)
}

pub(crate) fn eval_confirm_assert(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::ConfirmAssert, head, args)
}

pub(crate) fn eval_return(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Return, head, args)
}

pub(crate) fn eval_break(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Break, head, args)
}

pub(crate) fn eval_continue(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Continue, head, args)
}

pub(crate) fn eval_throw(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Throw, head, args)
}

pub(crate) fn eval_catch(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Catch, head, args)
}

pub(crate) fn eval_failure(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Failure, head, args)
}

pub(crate) fn eval_failure_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::FailureQ, head, args)
}

pub(crate) fn eval_success(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ControlDataBuiltin::Success, head, args)
}

pub(crate) fn eval_terminated_evaluation(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(
        session,
        ControlDataBuiltin::TerminatedEvaluation,
        head,
        args,
    )
}
