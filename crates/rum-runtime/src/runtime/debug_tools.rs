use std::time::{Duration, Instant};

use super::*;

impl Session {
    pub(super) fn eval_trace(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Trace", args, 1));
        };

        let (_value, trace_exprs) = self.eval_with_trace_capture(expr.clone())?;
        Ok(Expr::call(
            system_expr("List"),
            trace_exprs
                .into_iter()
                .map(|item| Expr::call(system_expr("Hold"), vec![item]))
                .collect(),
        ))
    }

    pub(super) fn eval_trace_print(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("TracePrint", args, 1));
        };

        let (value, trace_exprs) = self.eval_with_trace_capture(expr.clone())?;
        for item in trace_exprs {
            self.push_print(render_echo_fragment(&item));
        }
        Ok(value)
    }

    pub(super) fn eval_timing(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_timing_like("Timing", args)
    }

    pub(super) fn eval_absolute_timing(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_timing_like("AbsoluteTiming", args)
    }

    pub(super) fn eval_echo(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [value] => {
                self.push_print(render_echo_fragment(value));
                Ok(value.clone())
            }
            [value, label] => {
                self.push_print(format!(
                    "{}: {}",
                    render_echo_fragment(label),
                    render_echo_fragment(value)
                ));
                Ok(value.clone())
            }
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    pub(super) fn eval_echo_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let body = match args {
            [] => Expr::call(system_expr("Echo"), vec![debug_slot_expr()]),
            [label] => Expr::call(system_expr("Echo"), vec![debug_slot_expr(), label.clone()]),
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        Ok(Expr::call(system_expr("Function"), vec![body]))
    }

    pub(super) fn eval_echo_label(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [label] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(
            system_expr("Function"),
            vec![Expr::call(
                system_expr("Echo"),
                vec![debug_slot_expr(), label.clone()],
            )],
        ))
    }

    pub(super) fn eval_byte_count(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(BigInt::from(
            render_canonical_byte_count(expr) as u64,
        )))
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

    fn eval_with_trace_capture(&mut self, expr: Expr) -> Result<(Expr, Vec<Expr>), EvalError> {
        let prior_enabled = self.tracing_enabled;
        let trace_checkpoint = self.trace_events.len();
        self.tracing_enabled = true;

        let value = match self.evaluate(expr.clone()) {
            Ok(value) => value,
            Err(error) => {
                self.restore_trace_capture_on_error(trace_checkpoint, prior_enabled);
                return Err(error);
            }
        };

        self.finish_trace_capture(expr, trace_checkpoint, prior_enabled, value)
    }

    fn finish_trace_capture(
        &mut self,
        original: Expr,
        trace_checkpoint: usize,
        prior_enabled: bool,
        value: Expr,
    ) -> Result<(Expr, Vec<Expr>), EvalError> {
        let events = self.trace_events[trace_checkpoint..].to_vec();
        self.tracing_enabled = prior_enabled;

        if !prior_enabled {
            self.trace_events.truncate(trace_checkpoint);
        }

        let trace_exprs = self.trace_exprs_from_events(&original, &events);
        Ok((value, trace_exprs))
    }

    fn restore_trace_capture_on_error(&mut self, trace_checkpoint: usize, prior_enabled: bool) {
        self.tracing_enabled = prior_enabled;
        if !prior_enabled {
            self.trace_events.truncate(trace_checkpoint);
        }
    }

    fn trace_exprs_from_events(&self, original: &Expr, events: &[TraceEvent]) -> Vec<Expr> {
        let mut trace = Vec::new();
        push_unique_trace_expr(&mut trace, original.clone());

        for event in events {
            match event {
                TraceEvent::EvaluationPass { before, after } => {
                    push_unique_trace_expr(&mut trace, self.trace_expr_from_text(before));
                    push_unique_trace_expr(&mut trace, self.trace_expr_from_text(after));
                }
                TraceEvent::DefinitionStored { value, .. } => {
                    push_unique_trace_expr(&mut trace, self.trace_expr_from_text(value));
                }
                TraceEvent::RuleApplied { input, result, .. } => {
                    push_unique_trace_expr(&mut trace, self.trace_expr_from_text(input));
                    push_unique_trace_expr(&mut trace, self.trace_expr_from_text(result));
                }
            }
        }

        trace
    }

    fn trace_expr_from_text(&self, text: &str) -> Expr {
        self.lower_text(text)
            .unwrap_or_else(|_| Expr::string(text.to_string()))
    }
}

fn debug_slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}

fn render_echo_fragment(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::String(value)) => value.clone(),
        other => other.to_string(),
    }
}

fn render_canonical_byte_count(expr: &Expr) -> usize {
    render_echo_fragment(expr).len()
}

fn elapsed_duration_expr(duration: Duration) -> Expr {
    let numerator = BigInt::from(duration.as_nanos());
    let denominator = BigInt::from(1_000_000_000_u64);
    let rational = Rational::new(numerator, denominator)
        .expect("positive denominator should always build a rational");
    Expr::rational(rational)
}

fn push_unique_trace_expr(trace: &mut Vec<Expr>, expr: Expr) {
    if trace.last().is_some_and(|current| current == &expr) {
        return;
    }
    trace.push(expr);
}
