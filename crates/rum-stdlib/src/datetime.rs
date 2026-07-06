use std::time::{SystemTime, UNIX_EPOCH};

use rum_core::{BigInt, Expr, Rational};
use rum_runtime::kernel_api::{Capability, EvalError, Session, expr_as_string, system_expr};

mod calendar;
mod format;
mod iso;

pub(crate) fn eval_date_list(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [] => {
            if !clock_capability_allowed(session, "DateList") {
                return Ok(system_expr("$Failed"));
            }
            Ok(calendar::date_list_expr_from_timestamp(&current_unix_time()))
        }
        [timestamp] => match calendar::timestamp_from_timestamp_or_date_list_expr(timestamp) {
            Ok(timestamp) => Ok(calendar::date_list_expr_from_timestamp(&timestamp)),
            Err(error) => Ok(fail_datetime(session, "DateList", error)),
        },
        _ => Ok(session.invalid_call_with_arg_range("DateList", args, 0, 1)),
    }
}

pub(crate) fn eval_date_string(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let timestamp = match args {
        [] => {
            if !clock_capability_allowed(session, "DateString") {
                return Ok(system_expr("$Failed"));
            }
            current_unix_time()
        }
        [value] => match calendar::timestamp_from_timestamp_or_date_list_expr(value) {
            Ok(timestamp) => timestamp,
            Err(error) => return Ok(fail_datetime(session, "DateString", error)),
        },
        _ => return Ok(session.invalid_call_with_arg_range("DateString", args, 0, 1)),
    };

    match format::date_string_from_timestamp(&timestamp) {
        Ok(value) => Ok(Expr::string(value)),
        Err(error) => Ok(fail_datetime(session, "DateString", error)),
    }
}

pub(crate) fn eval_from_date_string(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [source] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("FromDateString", args, 1));
    };
    let Some(source) = expr_as_string(source) else {
        return Ok(fail_datetime(
            session,
            "FromDateString",
            calendar::DateTimeError::date("FromDateString expects an ISO 8601 string."),
        ));
    };

    match iso::timestamp_from_date_string(source) {
        Ok(timestamp) => Ok(Expr::rational(timestamp)),
        Err(error) => Ok(fail_datetime(session, "FromDateString", error)),
    }
}

pub(crate) fn eval_date_plus(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [date, spec] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DatePlus", args, 2));
    };

    match calendar::date_plus(date, spec) {
        Ok(timestamp) => Ok(Expr::rational(timestamp)),
        Err(error) => Ok(fail_datetime(session, "DatePlus", error)),
    }
}

pub(crate) fn eval_date_difference(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [start, end, unit] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DateDifference", args, 3));
    };

    match calendar::date_difference(start, end, unit) {
        Ok(value) => Ok(Expr::rational(value)),
        Err(error) => Ok(fail_datetime(session, "DateDifference", error)),
    }
}

pub(crate) fn eval_day_of_week(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [date] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DayOfWeek", args, 1));
    };

    match calendar::day_of_week(date) {
        Ok(name) => Ok(system_expr(name)),
        Err(error) => Ok(fail_datetime(session, "DayOfWeek", error)),
    }
}

pub(crate) fn eval_leap_year_q(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [year] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("LeapYearQ", args, 1));
    };

    match calendar::leap_year_from_expr(year) {
        Ok(value) => Ok(bool_expr(value)),
        Err(error) => Ok(fail_datetime(session, "LeapYearQ", error)),
    }
}

pub(crate) fn eval_days_in_month(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [year, month] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DaysInMonth", args, 2));
    };

    match calendar::days_in_month_from_exprs(year, month) {
        Ok(value) => Ok(Expr::small_integer(value)),
        Err(error) => Ok(fail_datetime(session, "DaysInMonth", error)),
    }
}

fn current_unix_time() -> Rational {
    unix_time_rational(SystemTime::now())
}

fn unix_time_rational(time: SystemTime) -> Rational {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => Rational::new(
            BigInt::from(duration.as_nanos()),
            BigInt::from(1_000_000_000_u64),
        )
        .expect("positive denominator should always build a rational"),
        Err(error) => Rational::new(
            -BigInt::from(error.duration().as_nanos()),
            BigInt::from(1_000_000_000_u64),
        )
        .expect("positive denominator should always build a rational"),
    }
}

fn clock_capability_allowed(session: &mut Session, name: &str) -> bool {
    let policy = session.capability_policy();
    if policy.is_unrestricted() || policy.allows(Capability::Clock) {
        return true;
    }

    session.push_message(
        "General",
        "sandbox",
        format!("{name} is not permitted in this sandbox."),
    );
    false
}

fn fail_datetime(session: &mut Session, symbol: &str, error: calendar::DateTimeError) -> Expr {
    session.push_message(symbol, error.tag(), error.message());
    system_expr("$Failed")
}

fn bool_expr(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}
