use super::*;

impl Session {
    pub(in crate::runtime) fn eval_sow(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
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

    pub(in crate::runtime) fn eval_reap(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

fn normalize_reap_patterns(expr: Option<&Expr>) -> (Vec<Expr>, bool) {
    match expr {
        None => (vec![Expr::call(system_expr("Blank"), vec![])], false),
        Some(Expr::Call { head, args }) if is_system_symbol_name(head, "List") => {
            (args.to_vec(), true)
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
