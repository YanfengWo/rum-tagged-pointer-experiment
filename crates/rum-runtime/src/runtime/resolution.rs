use super::*;

impl Session {
    pub(super) fn normalize_match_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::Atom(_) => expr.clone(),
            Expr::Call { head, args } => {
                let normalized_head = self.normalize_match_expr(head);
                let normalized_args = args
                    .iter()
                    .map(|arg| self.normalize_match_expr(arg))
                    .collect::<Vec<_>>();
                let attributes = self.attribute_set_for_head(&normalized_head);
                let normalized_args = self.normalize_call_args_by_attributes(
                    &normalized_head,
                    normalized_args,
                    attributes,
                );

                Expr::call(normalized_head, normalized_args)
            }
        }
    }

    pub(super) fn lower_syntax_expr(&self, syntax: &SyntaxExpr) -> Result<Expr, EvalError> {
        lower_syntax_expr_with(syntax, &mut |raw| self.resolve_symbol_name(raw, true))
    }

    pub(super) fn builtin_spec_for_symbol(&self, symbol: &Symbol) -> Option<&BuiltinSpec> {
        (symbol.context() == SYSTEM_CONTEXT)
            .then(|| self.builtins.get(symbol.name()))
            .flatten()
    }

    pub(super) fn builtin_dispatch_spec_for_symbol(&self, symbol: &Symbol) -> Option<BuiltinSpec> {
        if symbol.context() != SYSTEM_CONTEXT {
            return None;
        }

        match (
            self.builtin_spec_for_symbol(symbol).copied(),
            default_builtin_dispatch_spec(symbol.name()),
        ) {
            (Some(spec), Some(fallback)) => Some(spec.with_fallback_dispatch(fallback)),
            (Some(spec), None) => Some(spec),
            (None, Some(fallback)) => Some(fallback),
            (None, None) => None,
        }
    }

    pub(super) fn resolve_symbol_name(&self, raw: &str, create_if_missing: bool) -> Option<Symbol> {
        if let Some((context, name)) = split_symbol_name(raw) {
            let context = normalize_context_name(context);
            return Some(self.active_symbol_for_name(&context, name));
        }

        let mut seen_contexts = BTreeSet::new();

        for context in self
            .context_path
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.current_context.as_str()))
        {
            if !seen_contexts.insert(context) {
                continue;
            }

            let symbol = self.active_symbol_for_name(context, raw);

            if self.symbol_exists(&symbol) {
                return Some(symbol);
            }
        }

        create_if_missing.then(|| self.active_symbol_for_name(&self.current_context, raw))
    }

    pub(super) fn active_symbol_for_name(&self, context: &str, name: &str) -> Symbol {
        if context == SYSTEM_CONTEXT {
            return Symbol::new(context, name);
        }

        self.symbol_generations
            .get(&symbol_generation_key(context, name))
            .copied()
            .map(|identity| Symbol::with_identity(context, name, identity))
            .unwrap_or_else(|| Symbol::new(context, name))
    }

    pub(super) fn symbol_exists(&self, symbol: &Symbol) -> bool {
        self.definitions.contains_key(symbol)
            || self.declared_stubs.contains(symbol)
            || self
                .attributes
                .get(symbol)
                .copied()
                .is_some_and(|attributes| !attributes.is_empty())
            || self
                .options
                .get(symbol)
                .is_some_and(|options| !options.is_empty())
            || self.syntax_information.contains_key(symbol)
            || self
                .message_definitions
                .keys()
                .any(|key| &key.symbol == symbol)
            || self.builtin_spec_for_symbol(symbol).is_some()
            || is_known_system_symbol(symbol)
    }

    pub(super) fn attribute_set_for_symbol(&self, symbol: &Symbol) -> AttributeSet {
        self.attributes
            .get(symbol)
            .copied()
            .unwrap_or_else(|| default_system_attribute_set(symbol))
    }

    pub(super) fn set_attribute_set_for_symbol(
        &mut self,
        symbol: Symbol,
        attributes: AttributeSet,
    ) {
        let previous = self.attribute_set_for_symbol(&symbol);
        if previous == attributes {
            return;
        }

        self.bump_attribute_revision();
        let default = default_system_attribute_set(&symbol);

        if attributes == default {
            self.attributes.remove(&symbol);
        } else {
            self.attributes.insert(symbol, attributes);
        }
    }

    pub(super) fn bump_attribute_revision(&mut self) {
        self.attribute_revision = self.attribute_revision.saturating_add(1);
        self.bump_evaluation_epoch();
    }

    pub(super) fn resolve_load_target(&self, spec: &str) -> Result<PathBuf, EvalError> {
        self.find_load_target(spec)
            .map(|(path, _)| path)
            .ok_or_else(|| {
                let (_, searched) = self.load_target_candidates(spec);
                EvalError::LoadPathNotFound {
                    target: spec.to_string(),
                    searched,
                }
            })
    }

    pub(super) fn load_search_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();

        if let Some(parent) = self
            .load_stack
            .last()
            .and_then(|path| path.parent().map(Path::to_path_buf))
        {
            roots.push(parent);
        }

        roots.extend(self.package_search_paths.iter().cloned());
        dedup_paths(roots)
    }

    pub(super) fn find_load_target(&self, spec: &str) -> Option<(PathBuf, Vec<String>)> {
        let (candidates, searched) = self.load_target_candidates(spec);

        for candidate in candidates {
            if candidate.is_file() {
                return Some((normalize_found_path(&candidate), searched));
            }
        }

        None
    }

    pub(super) fn load_target_candidates(&self, spec: &str) -> (Vec<PathBuf>, Vec<String>) {
        let specs = if spec.ends_with('`') {
            package_candidates_for_context(spec)
        } else {
            vec![PathBuf::from(spec)]
        };

        let mut candidates = Vec::new();
        let mut searched = Vec::new();

        for spec in specs {
            if spec.is_absolute() {
                searched.push(spec.display().to_string());
                candidates.push(spec);
                continue;
            }

            for root in self.load_search_roots() {
                let path = root.join(&spec);
                searched.push(path.display().to_string());
                candidates.push(path);
            }
        }

        (candidates, searched)
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        let mut current = expr;
        let mut evaluation_passes = 0;
        let mut fixed_point_seen = Vec::new();

        loop {
            if evaluation_passes >= self.resource_limits.evaluation_pass_limit {
                return Err(EvalError::EvaluationPassLimitExceeded {
                    limit: self.resource_limits.evaluation_pass_limit,
                });
            }
            evaluation_passes += 1;

            if should_track_evaluation_cycle(&current) {
                if fixed_point_seen.iter().any(|previous| previous == &current) {
                    return Err(EvalError::EvaluationCycleDetected {
                        expr: current.to_string(),
                    });
                }
                fixed_point_seen.push(current.clone());
            }

            let next_result = self.evaluate_once(current.clone());
            let next = match next_result {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    let current_for_resume = current.clone();
                    let seen_for_resume = fixed_point_seen.clone();
                    self.suspend_with_frame(
                        Err(EvalError::InputRequired { request }),
                        move |session, next| {
                            session.resume_fixed_point_from(
                                current_for_resume,
                                evaluation_passes,
                                seen_for_resume,
                                next,
                            )
                        },
                    )?
                }
                Err(error) => return Err(error),
            };

            if next == current {
                return Ok(next);
            }

            self.push_evaluation_pass_trace(&current, &next);

            current = next;
        }
    }

    pub(super) fn resume_fixed_point_from(
        &mut self,
        current: Expr,
        evaluation_passes: usize,
        fixed_point_seen: Vec<Expr>,
        next: Expr,
    ) -> Result<Expr, EvalError> {
        if next == current {
            return Ok(next);
        }

        self.push_evaluation_pass_trace(&current, &next);

        let mut current = next;
        let mut evaluation_passes = evaluation_passes;
        let mut fixed_point_seen = fixed_point_seen;

        loop {
            if evaluation_passes >= self.resource_limits.evaluation_pass_limit {
                return Err(EvalError::EvaluationPassLimitExceeded {
                    limit: self.resource_limits.evaluation_pass_limit,
                });
            }
            evaluation_passes += 1;

            if should_track_evaluation_cycle(&current) {
                if fixed_point_seen.iter().any(|previous| previous == &current) {
                    return Err(EvalError::EvaluationCycleDetected {
                        expr: current.to_string(),
                    });
                }
                fixed_point_seen.push(current.clone());
            }

            let next_result = self.evaluate_once(current.clone());
            let next = match next_result {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    let current_for_resume = current.clone();
                    let seen_for_resume = fixed_point_seen.clone();
                    self.suspend_with_frame(
                        Err(EvalError::InputRequired { request }),
                        move |session, next| {
                            session.resume_fixed_point_from(
                                current_for_resume,
                                evaluation_passes,
                                seen_for_resume,
                                next,
                            )
                        },
                    )?
                }
                Err(error) => return Err(error),
            };

            if next == current {
                return Ok(next);
            }

            self.push_evaluation_pass_trace(&current, &next);
            current = next;
        }
    }
}

const EVALUATION_CYCLE_HISTORY_NODE_LIMIT: usize = 8;

pub(super) fn should_track_evaluation_cycle(expr: &Expr) -> bool {
    expr_node_count_with_limit(expr, EVALUATION_CYCLE_HISTORY_NODE_LIMIT).is_some()
}

pub(super) fn expr_node_count_with_limit(expr: &Expr, limit: usize) -> Option<usize> {
    fn walk(expr: &Expr, remaining: &mut usize) -> bool {
        if *remaining == 0 {
            return false;
        }
        *remaining -= 1;

        match expr {
            Expr::Atom(_) => true,
            Expr::Call { head, args } => {
                if !walk(head, remaining) {
                    return false;
                }
                args.iter().all(|arg| walk(arg, remaining))
            }
        }
    }

    let mut remaining = limit;
    walk(expr, &mut remaining).then_some(limit - remaining)
}

pub(super) fn symbol_generation_key(context: &str, name: &str) -> (String, String) {
    (context.to_string(), name.to_string())
}
