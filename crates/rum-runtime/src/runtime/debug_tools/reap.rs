use super::*;

struct ReapScopeRenderState {
    handler: Option<Expr>,
    pattern_list_form: bool,
    groups: Vec<ReapBucketGroup>,
}

struct ReapBucketRenderState {
    handler: Option<Expr>,
    buckets: Vec<ReapBucket>,
}

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
                let outcome =
                    self.match_expr_owned(pattern, tag.clone(), SymbolExprMap::default())?;
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
            buckets_by_pattern: vec![ReapBucketGroup::default(); patterns.len()],
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
            Ok(value) => {
                let suspended_value = value.clone();
                let rendered_result = self.render_reap_scope(scope);
                let rendered = self.suspend_with_frame(rendered_result, move |_, rendered| {
                    Ok(finish_reap_result(suspended_value, rendered))
                })?;
                Ok(finish_reap_result(value, rendered))
            }
            Err(error) => Err(error),
        }
    }

    fn render_reap_scope(&mut self, scope: ReapScope) -> Result<Expr, EvalError> {
        self.resume_reap_scope_groups_from_index(
            ReapScopeRenderState {
                handler: scope.handler,
                pattern_list_form: scope.pattern_list_form,
                groups: scope.buckets_by_pattern,
            },
            0,
            Vec::new(),
        )
    }

    /// Renders one group per loop iteration instead of recursing per group.
    /// On suspension the group index and accumulated state move into the
    /// suspension frame by value — the old per-group `state.clone()` plus
    /// `rendered_groups.clone()` made a wide Reap quadratic in its group
    /// count.
    fn resume_reap_scope_groups_from_index(
        &mut self,
        state: ReapScopeRenderState,
        group_index: usize,
        mut rendered_groups: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let mut group_index = group_index;
        loop {
            let Some(buckets) = state.groups.get(group_index) else {
                return Ok(if state.pattern_list_form {
                    Expr::call(system_expr("List"), rendered_groups)
                } else {
                    rendered_groups
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| Expr::call(system_expr("List"), Vec::new()))
                });
            };

            let bucket_state = ReapBucketRenderState {
                handler: state.handler.clone(),
                buckets: buckets.buckets.clone(),
            };
            let group_result = self.resume_reap_buckets_from_index(bucket_state, 0, Vec::new());
            match group_result {
                Ok(group) => {
                    rendered_groups.push(group);
                    group_index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, outcome| {
                        let group = outcome?;
                        session.resume_reap_scope_group_with_value(
                            state,
                            group_index,
                            rendered_groups,
                            group,
                        )
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_reap_scope_group_with_value(
        &mut self,
        state: ReapScopeRenderState,
        group_index: usize,
        mut rendered_groups: Vec<Expr>,
        group: Expr,
    ) -> Result<Expr, EvalError> {
        rendered_groups.push(group);
        self.resume_reap_scope_groups_from_index(state, group_index + 1, rendered_groups)
    }

    /// Renders one bucket per loop iteration instead of recursing per bucket.
    /// On suspension the bucket index, the render state, and the accumulated
    /// output move into the suspension frame by value — the old per-bucket
    /// `state.clone()` plus `rendered.clone()` made a wide Reap quadratic in
    /// its bucket count.
    fn resume_reap_buckets_from_index(
        &mut self,
        state: ReapBucketRenderState,
        bucket_index: usize,
        mut rendered: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let mut bucket_index = bucket_index;
        loop {
            let Some(bucket) = state.buckets.get(bucket_index) else {
                return Ok(Expr::call(system_expr("List"), rendered));
            };

            let values_expr = Expr::call(system_expr("List"), bucket.values.clone());
            let Some(handler) = &state.handler else {
                rendered.push(values_expr);
                bucket_index += 1;
                continue;
            };

            let applied = Expr::call(handler.clone(), vec![bucket.tag.clone(), values_expr]);
            let group_result = self.evaluate(applied);
            match group_result {
                Ok(group) => {
                    rendered.push(group);
                    bucket_index += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, outcome| {
                        let group = outcome?;
                        session.resume_reap_bucket_with_value(state, bucket_index, rendered, group)
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_reap_bucket_with_value(
        &mut self,
        state: ReapBucketRenderState,
        bucket_index: usize,
        mut rendered: Vec<Expr>,
        group: Expr,
    ) -> Result<Expr, EvalError> {
        rendered.push(group);
        self.resume_reap_buckets_from_index(state, bucket_index + 1, rendered)
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

const REAP_BUCKET_INDEX_THRESHOLD: usize = 16;

fn record_sown_value(group: &mut ReapBucketGroup, tag: Expr, value: Expr) {
    if group.tag_positions.is_none() && group.buckets.len() > REAP_BUCKET_INDEX_THRESHOLD {
        group.tag_positions = Some(reap_tag_positions(&group.buckets));
    }

    if let Some(tag_positions) = &group.tag_positions {
        if let Some(position) = tag_positions.get(&tag).copied() {
            group.buckets[position].values.push(value);
            return;
        }
    } else if let Some(bucket) = group.buckets.iter_mut().find(|bucket| bucket.tag == tag) {
        bucket.values.push(value);
        return;
    }

    let position = group.buckets.len();
    if let Some(tag_positions) = &mut group.tag_positions {
        tag_positions.insert(tag.clone(), position);
    }
    group.buckets.push(ReapBucket {
        tag,
        values: vec![value],
    });

    if group.tag_positions.is_none() && group.buckets.len() > REAP_BUCKET_INDEX_THRESHOLD {
        group.tag_positions = Some(reap_tag_positions(&group.buckets));
    }
}

#[inline(never)]
fn reap_tag_positions(buckets: &[ReapBucket]) -> std::collections::HashMap<Expr, usize> {
    buckets
        .iter()
        .enumerate()
        .map(|(position, bucket)| (bucket.tag.clone(), position))
        .collect()
}

fn finish_reap_result(value: Expr, harvested: Expr) -> Expr {
    Expr::call(system_expr("List"), vec![value, harvested])
}
