use super::*;

const REPEATED_TIMING_SAMPLES: usize = 3;

impl Session {
    pub(in crate::runtime) fn eval_absolute_time(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(unix_time_expr(SystemTime::now())),
            [expr] => match expr {
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
