use rum_core::Expr;
use rum_runtime::kernel_api::{EvalError, RewriteDataBuiltin, Session};

fn dispatch(
    session: &mut Session,
    builtin: RewriteDataBuiltin,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    session.eval_rewrite_data_builtin(builtin, head, args)
}

pub(crate) fn eval_cases(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, RewriteDataBuiltin::Cases, head, args)
}

pub(crate) fn eval_delete_cases(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, RewriteDataBuiltin::DeleteCases, head, args)
}

pub(crate) fn eval_match_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, RewriteDataBuiltin::MatchQ, head, args)
}

pub(crate) fn eval_replace(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, RewriteDataBuiltin::Replace, head, args)
}

pub(crate) fn eval_replace_all(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, RewriteDataBuiltin::ReplaceAll, head, args)
}

pub(crate) fn eval_replace_all_unheld(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, RewriteDataBuiltin::ReplaceAllUnheld, head, args)
}

pub(crate) fn eval_replace_list(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, RewriteDataBuiltin::ReplaceList, head, args)
}

pub(crate) fn eval_replace_repeated(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, RewriteDataBuiltin::ReplaceRepeated, head, args)
}
