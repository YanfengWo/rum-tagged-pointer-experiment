use super::*;

impl Session {
    pub(in crate::runtime) fn eval_module(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.record_profile_eval_module_entry();
        let profile_start = lexical_profile_start();
        let eval_cache_profile_start =
            evaluation_cache_profile_source_timing_start("eval_module_by_source");

        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Module", args, 2));
        }

        let runtime_profile_compile = runtime_path_timing_start("module.compile_bindings");
        let config_result = self.compile_scoped_call_from_raw_args(args);
        record_profile_runtime_path_timing(
            runtime_profile_compile,
            if config_result.is_ok() {
                RuntimeProfileOutcome::Ok
            } else {
                RuntimeProfileOutcome::Error
            },
        );
        let config = match config_result {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Module", args, error.message()));
            }
        };
        let items = config.items;

        let mut renames = SymbolExprMap::default();

        let runtime_profile_allocate = runtime_path_timing_start("module.allocate_fresh_symbols");
        for (symbol, _) in &items {
            renames.insert(
                symbol.clone(),
                Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))),
            );
        }
        record_profile_runtime_path_timing(runtime_profile_allocate, RuntimeProfileOutcome::Ok);

        let gensyms: Vec<Symbol> = renames
            .values()
            .filter_map(|expr| expr_as_symbol(expr).cloned())
            .collect();
        let activation_depth = self.module_activations.len();
        let side_table_stamp = self.current_side_table_stamp();
        self.module_activations.push(ModuleActivation {
            gensyms,
            escaped: false,
            side_table_stamp,
        });

        let runtime_profile_total = runtime_path_timing_start("module.eval.total");
        let result =
            self.resume_module_initializers(items, args[1].clone(), renames, activation_depth, 0);
        record_profile_eval_module(profile_start);
        record_profile_eval_module_by_source(eval_cache_profile_start);
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    fn resume_module_initializers(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        renames: SymbolExprMap,
        activation_depth: usize,
        mut index: usize,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_resume = runtime_path_timing_start("module.resume_initializers");
        loop {
            record_profile_module_resume_entry(index);
            if index >= items.len() {
                let profile_start = lexical_profile_start();
                let eval_cache_profile_start = evaluation_cache_profile_source_timing_start(
                    "module_body_substitution_by_source",
                );
                let body = substitute_lexical_bindings(&body, &renames, &BTreeSet::new());
                record_profile_module_body_substitution(profile_start);
                record_profile_module_body_substitution_by_source(eval_cache_profile_start);
                let runtime_profile_body = runtime_path_timing_start("module.body.evaluate");
                let result = self.evaluate(body);
                record_profile_runtime_path_timing(
                    runtime_profile_body,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                let outcome = self.suspend_with_result_frame(result, move |session, outcome| {
                    session.finish_module_scope(activation_depth, outcome)
                });

                let result = match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.finish_module_scope(activation_depth, outcome),
                };
                record_profile_runtime_path_timing(
                    runtime_profile_resume,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                return result;
            }

            let (symbol, init) = items[index].clone();
            if let Some(init) = init {
                let profile_start = lexical_profile_start();
                let eval_cache_profile_start = evaluation_cache_profile_source_timing_start(
                    "module_init_substitution_by_source",
                );
                let renamed_init = substitute_lexical_bindings(&init, &renames, &BTreeSet::new());
                record_profile_module_init_substitution(profile_start);
                record_profile_module_init_substitution_by_source(eval_cache_profile_start);
                let runtime_profile_init = runtime_path_timing_start("module.initializer.evaluate");
                let init_result = self.evaluate(renamed_init);
                record_profile_runtime_path_timing(
                    runtime_profile_init,
                    runtime_profile_outcome_for_expr_result(&init_result),
                );
                let outcome = self.suspend_with_result_frame_lazy(init_result, || {
                    let items = items.clone();
                    let body = body.clone();
                    let renames = renames.clone();
                    let symbol = symbol.clone();
                    move |session, outcome| {
                        session.resume_module_after_initializer(
                            items,
                            body,
                            renames,
                            activation_depth,
                            index,
                            symbol,
                            outcome,
                        )
                    }
                });

                let result = match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.resume_module_after_initializer(
                        items,
                        body,
                        renames,
                        activation_depth,
                        index,
                        symbol,
                        outcome,
                    ),
                };
                record_profile_runtime_path_timing(
                    runtime_profile_resume,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                return result;
            }

            index += 1;
        }
    }

    fn resume_module_after_initializer(
        &mut self,
        items: Vec<(Symbol, Option<Expr>)>,
        body: Expr,
        renames: SymbolExprMap,
        activation_depth: usize,
        index: usize,
        symbol: Symbol,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                self.store_module_initializer_value(&renames, &symbol, value);
                self.resume_module_initializers(items, body, renames, activation_depth, index + 1)
            }
            Err(error) => self.finish_module_scope(activation_depth, Err(error)),
        }
    }

    fn store_module_initializer_value(
        &mut self,
        renames: &SymbolExprMap,
        symbol: &Symbol,
        value: Expr,
    ) {
        let Some(fresh_expr) = renames.get(symbol) else {
            return;
        };
        let Some(fresh_symbol) = expr_as_symbol(fresh_expr).cloned() else {
            return;
        };

        let definition = self.definitions.entry(fresh_symbol).or_default();
        definition.own_value = Some(value);
        definition.delayed_own_value = None;
        self.bump_evaluation_epoch_for_symbol(&fresh_symbol);
    }

    fn finish_module_scope(
        &mut self,
        activation_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let activation = self.module_activations[activation_depth].clone();
        let escaped =
            activation.escaped || self.current_side_table_stamp() != activation.side_table_stamp;

        let mut reclaim = Vec::new();
        if !escaped {
            match &outcome {
                Ok(result) => {
                    reclaim =
                        self.module_gensyms_to_reclaim(&activation, std::slice::from_ref(result));
                }
                Err(_) => {
                    let roots = gensym_gc_extra_roots_for_outcome(&outcome);
                    reclaim = self.module_gensyms_to_reclaim(&activation, &roots);
                }
            }
        }

        let reclaim_set: BTreeSet<Symbol> = reclaim.iter().copied().collect();
        for gensym in &reclaim {
            self.gc_candidate_gensyms.remove(gensym);
        }
        // Advisory only: rollback does not snapshot this set. A stale entry is
        // rechecked by mark/sweep; a missing entry only skips reclamation.
        for gensym in &activation.gensyms {
            if !reclaim_set.contains(gensym) {
                self.gc_candidate_gensyms.insert(*gensym);
            }
        }

        self.module_activations.truncate(activation_depth);
        for gensym in reclaim {
            self.clear_symbol_state(&gensym, true, true, true, true, true);
        }

        if self.gc_candidate_gensyms.len() >= gensym_gc_threshold() {
            let extra_roots = gensym_gc_extra_roots_for_outcome(&outcome);
            self.collect_and_sweep_dead_gensyms(&extra_roots);
        }

        outcome
    }

    fn module_gensyms_to_reclaim(
        &self,
        activation: &ModuleActivation,
        roots: &[Expr],
    ) -> Vec<Symbol> {
        let mut retained: BTreeSet<Symbol> = activation
            .gensyms
            .iter()
            .filter(|gensym| roots.iter().any(|root| expr_contains_symbol(root, gensym)))
            .cloned()
            .collect();

        loop {
            let retained_snapshot: Vec<Symbol> = retained.iter().cloned().collect();
            let mut grew = false;

            for retained_gensym in retained_snapshot {
                let Some(definition) = self.definitions.get(&retained_gensym) else {
                    continue;
                };

                for gensym in &activation.gensyms {
                    if retained.contains(gensym) {
                        continue;
                    }

                    if definition_references_symbol(definition, gensym) {
                        retained.insert(gensym.clone());
                        grew = true;
                    }
                }
            }

            if !grew {
                break;
            }
        }

        activation
            .gensyms
            .iter()
            .filter(|gensym| !retained.contains(*gensym))
            .cloned()
            .collect()
    }
}
