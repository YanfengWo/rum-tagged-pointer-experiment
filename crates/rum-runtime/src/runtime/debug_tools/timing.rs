use num_traits::ToPrimitive;

use super::*;

const REPEATED_TIMING_SAMPLES: usize = 3;

#[derive(Clone)]
struct RepeatedTimingEvalState {
    expr: Expr,
}

impl Session {
    pub(in crate::runtime) fn eval_absolute_time(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(unix_time_expr(SystemTime::now())),
            [expr] => match expr {
                Expr::Call { .. } if list_items(expr).is_some() => {
                    match absolute_time_from_date_list_expr(expr) {
                        Ok(timestamp) => Ok(Expr::rational(timestamp)),
                        Err(message) => {
                            self.push_message("AbsoluteTime", "date", message);
                            Ok(system_expr("$Failed"))
                        }
                    }
                }
                Expr::Call {
                    head: date_head,
                    args: date_args,
                } if is_system_symbol_name(date_head, "DateObject") && date_args.len() == 1 => {
                    Ok(date_args[0].clone())
                }
                Expr::Atom(Atom::Integer(_)) | Expr::Atom(Atom::Rational(_)) => Ok(expr.clone()),
                _ => Ok(Expr::call(head.clone(), args.to_vec())),
            },
            _ => Ok(self.invalid_call_with_arg_range("AbsoluteTime", args, 0, 1)),
        }
    }

    pub(in crate::runtime) fn eval_timing(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_timing_like("Timing", args)
    }

    pub(in crate::runtime) fn eval_absolute_timing(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_timing_like("AbsoluteTiming", args)
    }

    pub(in crate::runtime) fn eval_repeated_timing(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("RepeatedTiming", args, 1));
        };

        self.resume_repeated_timing_from_sample(
            RepeatedTimingEvalState { expr: expr.clone() },
            0,
            None,
            system_expr("Null"),
        )
    }

    fn eval_timing_like(&mut self, symbol: &str, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count(symbol, args, 1));
        };

        let started = Instant::now();
        let evaluated = self.evaluate(expr.clone());
        let value = self.suspend_with_frame(evaluated, {
            let symbol = symbol.to_string();
            move |session, value| session.finish_timing_like(&symbol, started, value)
        })?;

        self.finish_timing_like(symbol, started, value)
    }

    fn finish_timing_like(
        &mut self,
        _symbol: &str,
        started: Instant,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(
            system_expr("List"),
            vec![elapsed_duration_expr(started.elapsed()), value],
        ))
    }

    fn resume_repeated_timing_from_sample(
        &mut self,
        state: RepeatedTimingEvalState,
        sample: usize,
        best_elapsed: Option<Duration>,
        last_value: Expr,
    ) -> Result<Expr, EvalError> {
        if sample >= REPEATED_TIMING_SAMPLES {
            return Ok(Expr::call(
                system_expr("List"),
                vec![
                    elapsed_duration_expr(best_elapsed.unwrap_or_default()),
                    last_value,
                ],
            ));
        }

        let started = Instant::now();
        let suspended_state = state.clone();
        let value_result = self.evaluate(state.expr.clone());
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_repeated_timing_with_value(
                suspended_state,
                sample,
                best_elapsed,
                started,
                value,
            )
        })?;

        self.resume_repeated_timing_with_value(state, sample, best_elapsed, started, value)
    }

    fn resume_repeated_timing_with_value(
        &mut self,
        state: RepeatedTimingEvalState,
        sample: usize,
        best_elapsed: Option<Duration>,
        started: Instant,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let elapsed = started.elapsed();
        let best_elapsed = Some(best_elapsed.map_or(elapsed, |best| best.min(elapsed)));
        self.resume_repeated_timing_from_sample(state, sample + 1, best_elapsed, value)
    }
}

fn absolute_time_from_date_list_expr(expr: &Expr) -> Result<Rational, String> {
    let items = list_items(expr)
        .ok_or_else(|| "Expected a date list with head List and one to six fields.".to_string())?;
    if !(1..=6).contains(&items.len()) {
        return Err("Date lists must have one to six fields: {y, mo, d, h, mi, s}.".to_string());
    }

    let year = expr_as_exact_integer(&items[0])
        .ok_or_else(|| "Date list year must be an exact integer.".to_string())?;
    let month = absolute_time_integer_field_or_default(items, 1, "month", 1, 1, 12)?;
    let day = absolute_time_integer_field_or_default(items, 2, "day", 1, 1, 31)?;
    let hour = absolute_time_integer_field_or_default(items, 3, "hour", 0, 0, 23)?;
    let minute = absolute_time_integer_field_or_default(items, 4, "minute", 0, 0, 59)?;
    let second = absolute_time_second_field_or_default(items)?;

    let max_day = absolute_time_days_in_month(&year, month);
    if day > max_day {
        return Err(format!(
            "Date list day {day} is invalid for month {month} in year {year}."
        ));
    }

    let days = absolute_time_days_from_civil(&year, month, day);
    let whole_seconds =
        days * BigInt::from(86_400_i64) + BigInt::from(hour * 3_600_i64 + minute * 60_i64);
    Ok(Rational::from_integer(whole_seconds).add(&second))
}

fn absolute_time_integer_field_or_default(
    items: &[Expr],
    index: usize,
    name: &str,
    default: i64,
    min: i64,
    max: i64,
) -> Result<i64, String> {
    let Some(expr) = items.get(index) else {
        return Ok(default);
    };
    let value = expr_as_exact_integer(expr)
        .ok_or_else(|| format!("Date list {name} must be an exact integer."))?;
    let min_big = BigInt::from(min);
    let max_big = BigInt::from(max);
    if value < min_big || value > max_big {
        return Err(format!(
            "Date list {name} must be in the range {min}..{max}."
        ));
    }
    value
        .to_i64()
        .ok_or_else(|| format!("Date list {name} is out of range."))
}

fn absolute_time_second_field_or_default(items: &[Expr]) -> Result<Rational, String> {
    let Some(expr) = items.get(5) else {
        return Ok(Rational::zero());
    };
    let second = expr_as_exact_rational(expr)
        .ok_or_else(|| "Date list second must be an exact number.".to_string())?;
    let zero = BigInt::from(0_i64);
    let upper = second.denominator() * BigInt::from(60_i64);
    if second.numerator() < &zero || second.numerator() >= &upper {
        return Err("Date list second must satisfy 0 <= s < 60.".to_string());
    }
    Ok(second)
}

fn absolute_time_days_from_civil(year: &BigInt, month: i64, day: i64) -> BigInt {
    let adjusted_year = if month <= 2 {
        year - BigInt::from(1_i64)
    } else {
        year.clone()
    };
    let era = if adjusted_year >= BigInt::from(0_i64) {
        &adjusted_year / BigInt::from(400_i64)
    } else {
        (&adjusted_year - BigInt::from(399_i64)) / BigInt::from(400_i64)
    };
    let yoe = adjusted_year - &era * BigInt::from(400_i64);
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = &yoe * BigInt::from(365_i64) + &yoe / 4_i64 - &yoe / 100_i64 + BigInt::from(doy);
    era * BigInt::from(146_097_i64) + doe - BigInt::from(719_468_i64)
}

fn absolute_time_is_leap_year(year: &BigInt) -> bool {
    let zero = BigInt::from(0_i64);
    year % 4_i64 == zero && (year % 100_i64 != zero || year % 400_i64 == zero)
}

fn absolute_time_days_in_month(year: &BigInt, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if absolute_time_is_leap_year(year) => 29,
        2 => 28,
        _ => unreachable!("month is validated before days_in_month"),
    }
}

pub(in crate::runtime) fn unix_time_expr(time: SystemTime) -> Expr {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let numerator = BigInt::from(duration.as_nanos());
            let denominator = BigInt::from(1_000_000_000_u64);
            Expr::rational(
                Rational::new(numerator, denominator)
                    .expect("positive denominator should always build a rational"),
            )
        }
        Err(error) => {
            let duration = error.duration();
            let numerator = -BigInt::from(duration.as_nanos());
            let denominator = BigInt::from(1_000_000_000_u64);
            Expr::rational(
                Rational::new(numerator, denominator)
                    .expect("positive denominator should always build a rational"),
            )
        }
    }
}
