use rum_core::Expr;
use rum_runtime::kernel_api::{ArithmeticDataBuiltin, EvalError, Session};

fn dispatch(
    session: &mut Session,
    builtin: ArithmeticDataBuiltin,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    session.eval_arithmetic_data_builtin(builtin, head, args)
}

pub(crate) fn eval_identity(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Identity, head, args)
}

pub(crate) fn eval_plus(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Plus, head, args)
}

pub(crate) fn eval_times(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Times, head, args)
}

pub(crate) fn eval_power(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Power, head, args)
}

pub(crate) fn eval_subtract(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Subtract, head, args)
}

pub(crate) fn eval_divide(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Divide, head, args)
}

pub(crate) fn eval_abs(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Abs, head, args)
}

pub(crate) fn eval_sign(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Sign, head, args)
}

pub(crate) fn eval_floor(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Floor, head, args)
}

pub(crate) fn eval_ceiling(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Ceiling, head, args)
}

pub(crate) fn eval_integer_part(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::IntegerPart, head, args)
}

pub(crate) fn eval_fractional_part(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::FractionalPart, head, args)
}

pub(crate) fn eval_minus(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Minus, head, args)
}

pub(crate) fn eval_round(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Round, head, args)
}

pub(crate) fn eval_factorial(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Factorial, head, args)
}

pub(crate) fn eval_factorial2(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Factorial2, head, args)
}

pub(crate) fn eval_total(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Total, head, args)
}

pub(crate) fn eval_denominator(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Denominator, head, args)
}

pub(crate) fn eval_numerator(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Numerator, head, args)
}

pub(crate) fn eval_mod(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Mod, head, args)
}

pub(crate) fn eval_quotient(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Quotient, head, args)
}

pub(crate) fn eval_power_mod(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::PowerMod, head, args)
}

pub(crate) fn eval_gcd(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::GCD, head, args)
}

pub(crate) fn eval_lcm(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::LCM, head, args)
}

pub(crate) fn eval_max(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Max, head, args)
}

pub(crate) fn eval_min(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::Min, head, args)
}

pub(crate) fn eval_min_max(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    dispatch(session, ArithmeticDataBuiltin::MinMax, head, args)
}
