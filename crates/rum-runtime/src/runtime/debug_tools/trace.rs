use super::*;

impl Session {
    pub(in crate::runtime) fn eval_trace(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let default_include_heads = self.system_option_bool("Trace", "Heads", false)?;

        if let [expr] = args {
            let (_value, trace) = self.eval_with_structured_trace_capture(expr.clone())?;
            return Ok(trace);
        }

        let (expr, pattern, levels, include_heads) = match args {
            [expr, pattern] => (expr, Some(pattern.clone()), None, default_include_heads),
            [expr, pattern, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr, Some(pattern.clone()), None, include_heads)
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return Ok(self.invalid_call_with_arg_form(
                            "Trace",
                            args,
                            "expected a standard levelspec or Heads -> True|False in the current structured-trace subset",
                        ));
                    };
                    (
                        expr,
                        Some(pattern.clone()),
                        Some(levels),
                        default_include_heads,
                    )
                }
            }
            [expr, pattern, levels_expr, option] => {
                let Some(levels) = parse_standard_level_spec(levels_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Trace",
                        args,
                        "expected a standard levelspec as the third argument in the current structured-trace subset",
                    ));
                };
                let Some(include_heads) =
                    parse_heads_option_with_default(option, default_include_heads)
                else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Trace",
                        args,
                        "expected Heads -> True|False as the optional fourth argument in the current structured-trace subset",
                    ));
                };
                (expr, Some(pattern.clone()), Some(levels), include_heads)
            }
            _ => return Ok(self.invalid_call_with_arg_range("Trace", args, 2, 4)),
        };

        let (_value, trace) = self.eval_with_structured_trace_capture(expr.clone())?;
        self.filter_structured_trace_expr(trace, pattern, levels, include_heads)
    }

    pub(in crate::runtime) fn eval_trace_print(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let default_include_heads = self.system_option_bool("TracePrint", "Heads", false)?;

        if let [expr] = args {
            let (value, trace) = self.eval_with_structured_trace_capture(expr.clone())?;
            for line in render_structured_trace_lines(&trace) {
                self.push_print(line);
            }
            return Ok(value);
        }

        let (expr, pattern, levels, include_heads) = match args {
            [expr, pattern] => (expr, Some(pattern.clone()), None, default_include_heads),
            [expr, pattern, third] => {
                if let Some(include_heads) = parse_heads_option(third) {
                    (expr, Some(pattern.clone()), None, include_heads)
                } else {
                    let Some(levels) = parse_standard_level_spec(third) else {
                        return Ok(self.invalid_call_with_arg_form(
                            "TracePrint",
                            args,
                            "expected a standard levelspec or Heads -> True|False in the current structured-trace subset",
                        ));
                    };
                    (
                        expr,
                        Some(pattern.clone()),
                        Some(levels),
                        default_include_heads,
                    )
                }
            }
            [expr, pattern, levels_expr, option] => {
                let Some(levels) = parse_standard_level_spec(levels_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "TracePrint",
                        args,
                        "expected a standard levelspec as the third argument in the current structured-trace subset",
                    ));
                };
                let Some(include_heads) =
                    parse_heads_option_with_default(option, default_include_heads)
                else {
                    return Ok(self.invalid_call_with_arg_form(
                        "TracePrint",
                        args,
                        "expected Heads -> True|False as the optional fourth argument in the current structured-trace subset",
                    ));
                };
                (expr, Some(pattern.clone()), Some(levels), include_heads)
            }
            _ => return Ok(self.invalid_call_with_arg_range("TracePrint", args, 2, 4)),
        };

        let (value, trace) = self.eval_with_structured_trace_capture(expr.clone())?;
        let trace = self.filter_structured_trace_expr(trace, pattern, levels, include_heads)?;
        for line in render_structured_trace_lines(&trace) {
            self.push_print(line);
        }
        Ok(value)
    }

    fn eval_with_structured_trace_capture(
        &mut self,
        expr: Expr,
    ) -> Result<(Expr, Expr), EvalError> {
        let (value, events, depths) = self.capture_trace_events(expr.clone())?;
        Ok((
            value,
            self.structured_trace_expr_from_events(&expr, &events, &depths),
        ))
    }

    fn capture_trace_events(
        &mut self,
        expr: Expr,
    ) -> Result<(Expr, Vec<TraceEvent>, Vec<usize>), EvalError> {
        let prior_enabled = self.tracing_enabled;
        let trace_checkpoint = self.trace_events.len();
        self.tracing_enabled = true;

        let value = match self.evaluate(expr) {
            Ok(value) => value,
            Err(error) => {
                self.restore_trace_capture_on_error(trace_checkpoint, prior_enabled);
                return Err(error);
            }
        };

        self.finish_trace_event_capture(trace_checkpoint, prior_enabled, value)
    }

    fn finish_trace_event_capture(
        &mut self,
        trace_checkpoint: usize,
        prior_enabled: bool,
        value: Expr,
    ) -> Result<(Expr, Vec<TraceEvent>, Vec<usize>), EvalError> {
        let events = self.trace_events[trace_checkpoint..].to_vec();
        let depths = self.trace_event_depths[trace_checkpoint..].to_vec();
        self.tracing_enabled = prior_enabled;

        if !prior_enabled {
            self.truncate_trace_from(trace_checkpoint);
        }

        Ok((value, events, depths))
    }

    fn restore_trace_capture_on_error(&mut self, trace_checkpoint: usize, prior_enabled: bool) {
        self.tracing_enabled = prior_enabled;
        if !prior_enabled {
            self.truncate_trace_from(trace_checkpoint);
        }
    }

    pub(in crate::runtime::debug_tools) fn truncate_trace_from(&mut self, trace_checkpoint: usize) {
        self.trace_events.truncate(trace_checkpoint);
        self.trace_event_depths.truncate(trace_checkpoint);
    }

    pub(in crate::runtime::debug_tools) fn structured_trace_expr_from_events(
        &self,
        original: &Expr,
        events: &[TraceEvent],
        depths: &[usize],
    ) -> Expr {
        let mut builder = StructuredTraceBuilder::new();
        builder.push_expr(0, original.clone());
        let mut active_exprs = vec![original.clone()];

        for (event, raw_depth) in events.iter().zip(depths.iter().copied()) {
            match event {
                TraceEvent::EvaluationPass { before, after } => {
                    let before_expr = self.trace_expr_from_text(before);
                    let after_expr = self.trace_expr_from_text(after);
                    let depth =
                        trace_expr_path_depth(&active_exprs, &before_expr).unwrap_or(raw_depth);
                    active_exprs.truncate(depth + 1);
                    builder.push_evaluation_pass(depth, before_expr, after_expr.clone());
                    if active_exprs.len() <= depth {
                        active_exprs.resize(depth + 1, original.clone());
                    }
                    active_exprs[depth] = after_expr;
                }
                TraceEvent::DefinitionStored { value, .. } => {
                    let depth = raw_depth;
                    builder.push_expr(depth, self.trace_expr_from_text(value));
                }
                TraceEvent::RuleApplied { input, result, .. } => {
                    let input_expr = self.trace_expr_from_text(input);
                    let result_expr = self.trace_expr_from_text(result);
                    let depth = raw_depth;
                    active_exprs.truncate(depth + 1);
                    if active_exprs.len() <= depth {
                        active_exprs.resize(depth + 1, original.clone());
                    }
                    active_exprs[depth] = input_expr.clone();
                    builder.push_expr(depth, input_expr);
                    builder.push_expr(depth.saturating_add(1), result_expr.clone());
                    if active_exprs.len() <= depth + 1 {
                        active_exprs.push(result_expr);
                    } else {
                        active_exprs.truncate(depth + 2);
                        active_exprs[depth + 1] = result_expr;
                    }
                }
            }
        }

        builder.finish()
    }

    fn trace_expr_from_text(&self, text: &str) -> Expr {
        self.lower_text(text)
            .unwrap_or_else(|_| Expr::string(text.to_string()))
    }

    fn filter_structured_trace_expr(
        &mut self,
        trace: Expr,
        pattern: Option<Expr>,
        levels: Option<CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Expr, EvalError> {
        let Expr::Call { head, args } = trace else {
            return Ok(trace);
        };
        if !is_system_symbol_name(&head, "List") {
            return Ok(Expr::call(head.as_ref().clone(), args));
        }

        let filtered = self.filter_structured_trace_items(
            args,
            pattern.as_ref(),
            levels.as_ref(),
            include_heads,
        )?;
        Ok(Expr::call(system_expr("List"), filtered))
    }

    fn filter_structured_trace_items(
        &mut self,
        items: Vec<Expr>,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Vec<Expr>, EvalError> {
        let mut filtered = Vec::new();

        for item in items {
            match item {
                Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
                    let nested =
                        self.filter_structured_trace_items(args, pattern, levels, include_heads)?;
                    if !nested.is_empty() {
                        if nested.len() == 1 {
                            if let Expr::Call { head, args } = &nested[0] {
                                if is_system_symbol_name(head, "List") {
                                    filtered.push(Expr::call(system_expr("List"), args.clone()));
                                    continue;
                                }
                            }
                        }
                        filtered.push(Expr::call(system_expr("List"), nested));
                    }
                }
                other => {
                    for held in
                        self.filter_structured_trace_leaf(other, pattern, levels, include_heads)?
                    {
                        push_unique_trace_item(&mut filtered, held);
                    }
                }
            }
        }

        Ok(filtered)
    }

    fn filter_structured_trace_leaf(
        &mut self,
        item: Expr,
        pattern: Option<&Expr>,
        levels: Option<&CasesLevelSpec>,
        include_heads: bool,
    ) -> Result<Vec<Expr>, EvalError> {
        let Some(expr) = unwrap_trace_hold(&item) else {
            return Ok(vec![item]);
        };

        match (pattern, levels) {
            (Some(pattern), Some(levels)) => {
                let patterns = normalize_trace_patterns(pattern);
                let traversal = CasesTraversalConfig {
                    levels: levels.clone(),
                    max_matches: None,
                    include_heads,
                };
                let mut matched = Vec::new();

                for candidate in collect_level_parts_owned(&expr, traversal, 0) {
                    let mut is_match = false;
                    for pattern in &patterns {
                        let outcome = self.match_expr_owned(
                            pattern.clone(),
                            candidate.clone(),
                            BTreeMap::new(),
                        )?;
                        if outcome.matched {
                            is_match = true;
                            break;
                        }
                    }
                    if is_match {
                        push_unique_trace_expr(&mut matched, candidate);
                    }
                }

                Ok(matched
                    .into_iter()
                    .map(|candidate| Expr::call(system_expr("Hold"), vec![candidate]))
                    .collect())
            }
            (Some(pattern), None) => {
                if include_heads {
                    let all_levels = CasesLevelSpec::range(0, None);
                    self.filter_structured_trace_leaf(
                        Expr::call(system_expr("Hold"), vec![expr]),
                        Some(pattern),
                        Some(&all_levels),
                        true,
                    )
                } else {
                    let mut is_match = false;
                    for pattern in normalize_trace_patterns(pattern) {
                        let outcome =
                            self.match_expr_owned(pattern, expr.clone(), BTreeMap::new())?;
                        if outcome.matched {
                            is_match = true;
                            break;
                        }
                    }
                    Ok(if is_match {
                        vec![Expr::call(system_expr("Hold"), vec![expr])]
                    } else {
                        Vec::new()
                    })
                }
            }
            (None, Some(levels)) => {
                let traversal = CasesTraversalConfig {
                    levels: levels.clone(),
                    max_matches: None,
                    include_heads,
                };
                let mut selected = Vec::new();
                for candidate in collect_level_parts_owned(&expr, traversal, 0) {
                    push_unique_trace_expr(&mut selected, candidate);
                }
                Ok(selected
                    .into_iter()
                    .map(|candidate| Expr::call(system_expr("Hold"), vec![candidate]))
                    .collect())
            }
            (None, None) => Ok(vec![Expr::call(system_expr("Hold"), vec![expr])]),
        }
    }
}

fn push_unique_trace_expr(trace: &mut Vec<Expr>, expr: Expr) {
    if trace.last().is_some_and(|current| current == &expr) {
        return;
    }
    trace.push(expr);
}

fn push_unique_trace_item(items: &mut Vec<Expr>, item: Expr) {
    if items.last().is_some_and(|current| current == &item) {
        return;
    }
    items.push(item);
}

fn unwrap_trace_hold(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Hold") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn render_structured_trace_lines(trace: &Expr) -> Vec<String> {
    let mut lines = Vec::new();
    collect_structured_trace_lines(trace, 0, &mut lines);
    let min_depth = lines.iter().map(|(depth, _)| *depth).min().unwrap_or(0);
    lines
        .into_iter()
        .map(|(depth, line)| format!("{}{}", "  ".repeat(depth.saturating_sub(min_depth)), line))
        .collect()
}

fn collect_structured_trace_lines(trace: &Expr, depth: usize, lines: &mut Vec<(usize, String)>) {
    match trace {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            for item in args {
                match item {
                    Expr::Call { head, args: _ } if is_system_symbol_name(head, "List") => {
                        collect_structured_trace_lines(item, depth + 1, lines);
                    }
                    other => {
                        if let Some(expr) = unwrap_trace_hold(other) {
                            lines.push((depth, render_echo_fragment(&expr)));
                        } else {
                            lines.push((depth, render_echo_fragment(other)));
                        }
                    }
                }
            }
        }
        other => {
            if let Some(expr) = unwrap_trace_hold(other) {
                lines.push((depth, render_echo_fragment(&expr)));
            } else {
                lines.push((depth, render_echo_fragment(other)));
            }
        }
    }
}
fn normalize_trace_patterns(expr: &Expr) -> Vec<Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => args.clone(),
        other => vec![other.clone()],
    }
}
struct StructuredTraceBuilder {
    containers: Vec<Vec<Expr>>,
    current_exprs: Vec<Option<Expr>>,
}

impl StructuredTraceBuilder {
    fn new() -> Self {
        Self {
            containers: vec![Vec::new()],
            current_exprs: Vec::new(),
        }
    }

    fn push_expr(&mut self, depth: usize, expr: Expr) {
        self.close_to_depth(depth);

        while self.containers.len() <= depth {
            self.containers.push(Vec::new());
        }
        if self.current_exprs.len() <= depth {
            self.current_exprs.resize(depth + 1, None);
        }

        let duplicate = self.current_exprs[depth]
            .as_ref()
            .is_some_and(|current| current == &expr);
        self.current_exprs.truncate(depth + 1);
        if duplicate {
            return;
        }

        self.containers[depth].push(Expr::call(system_expr("Hold"), vec![expr.clone()]));
        self.current_exprs[depth] = Some(expr);
    }

    fn push_evaluation_pass(&mut self, depth: usize, before: Expr, after: Expr) {
        self.close_to_depth(depth);

        if !self.container_root_matches(depth, &before) {
            self.push_expr(depth, before);
        }
        self.push_expr(depth, after);
    }

    fn finish(mut self) -> Expr {
        self.close_to_depth(0);
        Expr::call(
            system_expr("List"),
            self.containers.pop().unwrap_or_default(),
        )
    }

    fn close_to_depth(&mut self, depth: usize) {
        while self.containers.len() > depth + 1 {
            let child = self.containers.pop().unwrap_or_default();
            if let Some(parent) = self.containers.last_mut() {
                parent.push(Expr::call(system_expr("List"), child));
            }
        }
    }

    fn container_root_matches(&self, depth: usize, target: &Expr) -> bool {
        self.containers.get(depth).and_then(|items| {
            items.iter().find_map(|item| match item {
                Expr::Call { head, args } if is_system_symbol_name(head, "Hold") => {
                    args.first().cloned()
                }
                _ => None,
            })
        }) == Some(target.clone())
    }
}

fn trace_expr_path_depth(active_exprs: &[Expr], target: &Expr) -> Option<usize> {
    active_exprs
        .iter()
        .enumerate()
        .rev()
        .find_map(|(depth, expr)| (expr == target).then_some(depth))
}
