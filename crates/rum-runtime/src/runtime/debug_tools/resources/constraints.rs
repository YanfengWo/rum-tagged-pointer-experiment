use super::memory::expr_memory_proxy_bytes;
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_time_constrained(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (body, limit_expr, fallback) = match args {
            [body, limit] => (body.clone(), limit, None),
            [body, limit, fallback] => (body.clone(), limit, Some(fallback.clone())),
            _ => {
                return Ok(self.invalid_call_with_arg_range("TimeConstrained", args, 2, 3));
            }
        };

        let resolved_limit = self.evaluate(limit_expr.clone())?;
        let Some(limit) = parse_time_constraint_duration(&resolved_limit) else {
            return Ok(self.invalid_call_with_arg_form(
                "TimeConstrained",
                args,
                "expected the timeout argument to be a non-negative exact number in the current first-pass subset",
            ));
        };

        let prior_deadline = self.current_time_deadline;
        let next_deadline = combine_time_deadlines(prior_deadline, limit);
        self.current_time_deadline = Some(next_deadline);

        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let fallback = fallback.clone();
            move |session, outcome| {
                session.finish_time_constrained(outcome, prior_deadline, fallback)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_time_constrained(outcome, prior_deadline, fallback),
        }
    }

    pub(in crate::runtime) fn eval_memory_constrained(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (body, limit_expr, fallback) = match args {
            [body, limit] => (body.clone(), limit, None),
            [body, limit, fallback] => (body.clone(), limit, Some(fallback.clone())),
            _ => {
                return Ok(self.invalid_call_with_arg_range("MemoryConstrained", args, 2, 3));
            }
        };

        let resolved_limit = self.evaluate(limit_expr.clone())?;
        let Some(limit) = expr_as_non_negative_usize(&resolved_limit) else {
            return Ok(self.invalid_call_with_arg_form(
                "MemoryConstrained",
                args,
                "expected the memory limit argument to be a non-negative exact integer",
            ));
        };

        let prior_limit = self.current_memory_limit;
        self.current_memory_limit = Some(prior_limit.map_or(limit, |current| current.min(limit)));

        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let fallback = fallback.clone();
            move |session, outcome| {
                session.finish_memory_constrained(outcome, prior_limit, fallback)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_memory_constrained(outcome, prior_limit, fallback),
        }
    }

    pub(in crate::runtime) fn enforce_active_resource_limits_for_expr(
        &self,
        expr: &Expr,
    ) -> Result<(), EvalError> {
        if let Some(deadline) = self.current_time_deadline {
            if Instant::now() >= deadline {
                return Err(EvalError::TimeConstraintExceeded);
            }
        }

        if let Some(limit) = self.current_memory_limit {
            let projected = self
                .current_memory_proxy_bytes()
                .saturating_add(expr_memory_proxy_bytes(expr));
            if projected > limit {
                return Err(EvalError::MemoryConstraintExceeded { limit });
            }
        }

        Ok(())
    }

    fn finish_time_constrained(
        &mut self,
        outcome: Result<Expr, EvalError>,
        prior_deadline: Option<Instant>,
        fallback: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.current_time_deadline = prior_deadline;

        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::TimeConstraintExceeded) => match fallback {
                Some(expr) => self.evaluate(expr),
                None => Ok(system_expr("$Aborted")),
            },
            Err(error) => Err(error),
        }
    }

    fn finish_memory_constrained(
        &mut self,
        outcome: Result<Expr, EvalError>,
        prior_limit: Option<usize>,
        fallback: Option<Expr>,
    ) -> Result<Expr, EvalError> {
        self.current_memory_limit = prior_limit;

        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::MemoryConstraintExceeded { .. }) => match fallback {
                Some(expr) => self.evaluate(expr),
                None => Ok(system_expr("$Aborted")),
            },
            Err(error) => Err(error),
        }
    }
}

fn parse_time_constraint_duration(expr: &Expr) -> Option<Duration> {
    let rational = expr_as_exact_rational(expr)?;
    if rational.numerator() < &BigInt::from(0) {
        return None;
    }

    let nanos_numerator = rational.numerator().clone() * BigInt::from(1_000_000_000_u64);
    let nanos = nanos_numerator / rational.denominator().clone();
    let nanos = nanos
        .to_string()
        .parse::<u64>()
        .unwrap_or(u64::MAX.saturating_sub(1));

    Some(Duration::from_nanos(nanos))
}

fn combine_time_deadlines(prior: Option<Instant>, limit: Duration) -> Instant {
    let candidate = Instant::now()
        .checked_add(limit)
        .unwrap_or_else(Instant::now);
    prior.map_or(candidate, |current| current.min(candidate))
}
