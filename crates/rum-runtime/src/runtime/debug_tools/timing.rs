use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

const REPEATED_TIMING_SAMPLES: usize = 3;

#[derive(Clone, FrameRootTrace)]
struct RepeatedTimingEvalState {
    expr: Expr,
}

impl Session {
    pub(in crate::runtime) fn eval_absolute_time(
        &mut self,
        _head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("AbsoluteTime", args, 0));
        }

        Ok(unix_time_expr(SystemTime::now()))
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
        let value = self.suspend_with_frame_lazy(evaluated, || {
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
        let value_result = self.evaluate(state.expr.clone());
        let value = self.suspend_with_frame_lazy(value_result, || {
            let suspended_state = state.clone();
            move |session, value| {
                session.resume_repeated_timing_with_value(
                    suspended_state,
                    sample,
                    best_elapsed,
                    started,
                    value,
                )
            }
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
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<RepeatedTimingEvalState>();
};
