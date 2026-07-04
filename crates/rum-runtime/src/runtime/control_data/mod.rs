use super::*;

mod abort_cleanup;
mod assertions;
mod block_random;
mod branching_loops;
mod check_quiet;
mod cleanup;
mod confirmation;
mod failsafe;
mod nonlocal;
mod status;

pub(super) use self::failsafe::{
    apply_callable_failsafe_wrapper_to_args, is_callable_failsafe_wrapper,
};
pub(super) use self::status::{is_failure_expr, is_success_expr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlDataBuiltin {
    Abort,
    AbortProtect,
    Assert,
    Break,
    Catch,
    CheckAbort,
    CompoundExpression,
    Confirm,
    ConfirmAssert,
    ConfirmBy,
    ConfirmMatch,
    ConfirmQuiet,
    Continue,
    Do,
    Enclose,
    Exit,
    Failsafe,
    Failure,
    FailureQ,
    For,
    If,
    Product,
    Return,
    Success,
    Sum,
    Switch,
    Table,
    TerminatedEvaluation,
    Throw,
    Until,
    Which,
    While,
    WithCleanup,
}

impl Session {
    /// Dispatches control-family builtin handlers whose bodies remain kernel-side.
    pub fn eval_control_data_builtin(
        &mut self,
        builtin: ControlDataBuiltin,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match builtin {
            ControlDataBuiltin::Abort => self.eval_abort(head, args),
            ControlDataBuiltin::AbortProtect => self.eval_abort_protect(args),
            ControlDataBuiltin::Assert => self.eval_assert(args),
            ControlDataBuiltin::Break => self.eval_break(head, args),
            ControlDataBuiltin::Catch => self.eval_catch(args),
            ControlDataBuiltin::CheckAbort => self.eval_check_abort(args),
            ControlDataBuiltin::CompoundExpression => self.eval_compound_expression(args),
            ControlDataBuiltin::Confirm => self.eval_confirm(args),
            ControlDataBuiltin::ConfirmAssert => self.eval_confirm_assert(args),
            ControlDataBuiltin::ConfirmBy => self.eval_confirm_by(args),
            ControlDataBuiltin::ConfirmMatch => self.eval_confirm_match(args),
            ControlDataBuiltin::ConfirmQuiet => self.eval_confirm_quiet(args),
            ControlDataBuiltin::Continue => self.eval_continue(head, args),
            ControlDataBuiltin::Do => self.eval_do(args),
            ControlDataBuiltin::Enclose => self.eval_enclose(args),
            ControlDataBuiltin::Exit => self.eval_exit(args),
            ControlDataBuiltin::Failsafe => self.eval_failsafe(head, args),
            ControlDataBuiltin::Failure => self.eval_failure(head, args),
            ControlDataBuiltin::FailureQ => self.eval_failure_q(head, args),
            ControlDataBuiltin::For => self.eval_for(args),
            ControlDataBuiltin::If => self.eval_if(args),
            ControlDataBuiltin::Product => self.eval_product(args),
            ControlDataBuiltin::Return => self.eval_return(head, args),
            ControlDataBuiltin::Success => self.eval_success(head, args),
            ControlDataBuiltin::Sum => self.eval_sum(args),
            ControlDataBuiltin::Switch => self.eval_switch(args),
            ControlDataBuiltin::Table => self.eval_table(args),
            ControlDataBuiltin::TerminatedEvaluation => self.eval_terminated_evaluation(head, args),
            ControlDataBuiltin::Throw => self.eval_throw(head, args),
            ControlDataBuiltin::Until => self.eval_until(args),
            ControlDataBuiltin::Which => self.eval_which(args),
            ControlDataBuiltin::While => self.eval_while(args),
            ControlDataBuiltin::WithCleanup => self.eval_with_cleanup(args),
        }
    }
}
