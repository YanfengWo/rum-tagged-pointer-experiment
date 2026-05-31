use super::*;

const REPEATED_TIMING_SAMPLES: usize = 3;

impl Session {
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

        let mut best_elapsed = None;
        let mut last_value = system_expr("Null");

        for _ in 0..REPEATED_TIMING_SAMPLES {
            let started = Instant::now();
            let value = match self.evaluate(expr.clone()) {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            let elapsed = started.elapsed();
            best_elapsed = Some(best_elapsed.map_or(elapsed, |best: Duration| best.min(elapsed)));
            last_value = value;
        }

        Ok(Expr::call(
            system_expr("List"),
            vec![
                elapsed_duration_expr(best_elapsed.unwrap_or_default()),
                last_value,
            ],
        ))
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
}
