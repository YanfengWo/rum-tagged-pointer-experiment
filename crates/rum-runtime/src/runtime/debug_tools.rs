use std::time::{Duration, Instant};

use super::*;

const REPEATED_TIMING_SAMPLES: usize = 3;
const DEFAULT_SHORT_LIMIT: usize = 72;

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

    pub(super) fn eval_repeated_timing(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_short(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (expr, limit) = match args {
            [expr] => (expr, DEFAULT_SHORT_LIMIT),
            [expr, limit_expr] => {
                let Some(limit) = expr_as_non_negative_usize(limit_expr).filter(|limit| *limit > 0)
                else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Short",
                        args,
                        "expected the optional second argument to be a positive integer",
                    ));
                };
                (expr, limit)
            }
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        Ok(Expr::string(short_canonical_fragment(expr, limit)))
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

    pub(super) fn eval_on(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_message_switch("On", args, true)
    }

    pub(super) fn eval_off(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_message_switch("Off", args, false)
    }

    pub(super) fn eval_sow(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (value, tag) = match args {
            [value] => (value.clone(), system_expr("None")),
            [value, tag] => (value.clone(), tag.clone()),
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        let mut matched = Vec::new();
        for scope_index in 0..self.reap_scopes.len() {
            let pattern_count = self.reap_scopes[scope_index].patterns.len();
            for pattern_index in 0..pattern_count {
                let pattern = self.reap_scopes[scope_index].patterns[pattern_index].clone();
                let outcome = self.match_expr_owned(pattern, tag.clone(), BTreeMap::new())?;
                if outcome.matched {
                    matched.push((scope_index, pattern_index));
                }
            }
        }

        for (scope_index, pattern_index) in matched {
            record_sown_value(
                &mut self.reap_scopes[scope_index].buckets_by_pattern[pattern_index],
                tag.clone(),
                value.clone(),
            );
        }

        Ok(value)
    }

    pub(super) fn eval_reap(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Reap", args, 1, 3));
        }

        let (patterns, pattern_list_form) = normalize_reap_patterns(args.get(1));
        let handler = args.get(2).cloned();
        self.reap_scopes.push(ReapScope {
            buckets_by_pattern: vec![Vec::new(); patterns.len()],
            patterns,
            pattern_list_form,
            handler,
        });

        let body_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(body_result, {
            move |session, outcome| session.resume_reap_after_body(outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_reap_after_body(outcome),
        }
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

    fn eval_message_switch(
        &mut self,
        symbol: &str,
        args: &[Expr],
        enabled: bool,
    ) -> Result<Expr, EvalError> {
        let Some(selectors) = normalize_message_switch_selectors(args) else {
            return Ok(self.invalid_call_with_arg_form(
                symbol,
                args,
                "expected one or more symbols, MessageName expressions, \"symbol::tag\" strings, or lists of those",
            ));
        };

        if selectors.is_empty() {
            return Ok(self.invalid_call_with_message(
                symbol,
                args,
                "argx",
                "expected at least 1 argument",
            ));
        }

        if enabled {
            self.disabled_message_selectors
                .retain(|existing| !selectors.iter().any(|selector| selector.covers(existing)));
        } else {
            for selector in selectors {
                if !self.disabled_message_selectors.contains(&selector) {
                    self.disabled_message_selectors.push(selector);
                }
            }
        }

        Ok(system_expr("Null"))
    }

    fn resume_reap_after_body(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let scope = self
            .reap_scopes
            .pop()
            .expect("reap scope should exist while finishing Reap");

        match outcome {
            Ok(value) => Ok(Expr::call(
                system_expr("List"),
                vec![value, self.render_reap_scope(scope)?],
            )),
            Err(error) => Err(error),
        }
    }

    fn render_reap_scope(&mut self, scope: ReapScope) -> Result<Expr, EvalError> {
        if scope.pattern_list_form {
            let mut rendered = Vec::with_capacity(scope.buckets_by_pattern.len());
            for buckets in scope.buckets_by_pattern {
                rendered.push(self.render_reap_buckets(&scope.handler, buckets)?);
            }
            Ok(Expr::call(system_expr("List"), rendered))
        } else {
            let buckets = scope
                .buckets_by_pattern
                .into_iter()
                .next()
                .unwrap_or_default();
            self.render_reap_buckets(&scope.handler, buckets)
        }
    }

    fn render_reap_buckets(
        &mut self,
        handler: &Option<Expr>,
        buckets: Vec<ReapBucket>,
    ) -> Result<Expr, EvalError> {
        let mut rendered = Vec::with_capacity(buckets.len());
        for bucket in buckets {
            let values_expr = Expr::call(system_expr("List"), bucket.values);
            let group = match handler {
                Some(handler) => {
                    let applied = Expr::call(handler.clone(), vec![bucket.tag, values_expr]);
                    self.evaluate(applied)?
                }
                None => values_expr,
            };
            rendered.push(group);
        }
        Ok(Expr::call(system_expr("List"), rendered))
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

fn short_canonical_fragment(expr: &Expr, limit: usize) -> String {
    let rendered = render_echo_fragment(expr);
    if rendered.len() <= limit || limit <= 8 {
        return rendered;
    }

    let prefix_len = limit / 2;
    let suffix_len = limit.saturating_sub(prefix_len + 3);
    let prefix = rendered.chars().take(prefix_len).collect::<String>();
    let suffix = rendered
        .chars()
        .rev()
        .take(suffix_len)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{prefix}...{suffix}")
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

fn normalize_message_switch_selectors(args: &[Expr]) -> Option<Vec<MessageSelector>> {
    let mut selectors = Vec::new();

    for arg in args {
        selectors.extend(parse_message_selectors(arg)?);
    }

    Some(selectors)
}

fn normalize_reap_patterns(expr: Option<&Expr>) -> (Vec<Expr>, bool) {
    match expr {
        None => (vec![Expr::call(system_expr("Blank"), vec![])], false),
        Some(Expr::Call { head, args }) if is_system_symbol_name(head, "List") => {
            (args.clone(), true)
        }
        Some(pattern) => (vec![pattern.clone()], false),
    }
}

fn record_sown_value(buckets: &mut Vec<ReapBucket>, tag: Expr, value: Expr) {
    if let Some(bucket) = buckets.iter_mut().find(|bucket| bucket.tag == tag) {
        bucket.values.push(value);
        return;
    }

    buckets.push(ReapBucket {
        tag,
        values: vec![value],
    });
}
