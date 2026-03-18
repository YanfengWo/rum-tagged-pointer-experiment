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
            return Some(Symbol::new(normalize_context_name(context), name));
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

            let symbol = Symbol::new(context.to_string(), raw);

            if self.symbol_exists(&symbol) {
                return Some(symbol);
            }
        }

        create_if_missing.then(|| Symbol::new(self.current_context.clone(), raw))
    }

    pub(super) fn symbol_exists(&self, symbol: &Symbol) -> bool {
        self.definitions.contains_key(symbol)
            || self
                .attributes
                .get(symbol)
                .copied()
                .is_some_and(|attributes| !attributes.is_empty())
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
        let default = default_system_attribute_set(&symbol);

        if attributes == default {
            self.attributes.remove(&symbol);
        } else {
            self.attributes.insert(symbol, attributes);
        }
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
        let mut fixed_point_seen = Vec::new();

        loop {
            if fixed_point_seen.len() >= self.resource_limits.evaluation_pass_limit {
                return Err(EvalError::EvaluationPassLimitExceeded {
                    limit: self.resource_limits.evaluation_pass_limit,
                });
            }

            if fixed_point_seen.iter().any(|previous| previous == &current) {
                return Err(EvalError::EvaluationCycleDetected {
                    expr: current.to_string(),
                });
            }

            fixed_point_seen.push(current.clone());
            let next = self.evaluate_once(current.clone())?;

            if next == current {
                return Ok(next);
            }

            self.push_trace(TraceEvent::EvaluationPass {
                before: current.to_string(),
                after: next.to_string(),
            });

            current = next;
        }
    }
}
