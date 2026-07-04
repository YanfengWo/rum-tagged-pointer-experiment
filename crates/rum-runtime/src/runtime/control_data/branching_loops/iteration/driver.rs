use super::spec::{build_iteration_body_or_nested, evaluate_do_spec_expr, parse_do_spec};
use super::table_fast_path::{
    eval_single_spec_constant_table_fast_path, eval_single_spec_table_fast_path,
};
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_do(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Do",
                args,
                "argx",
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        self.resume_do_after_spec_eval(args.to_vec(), 0)
    }

    pub(in crate::runtime) fn eval_table(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "Table",
                args,
                "argx",
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        self.resume_table_after_spec_eval(args.to_vec(), 0)
    }

    fn resume_do_after_spec_eval(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
    ) -> Result<Expr, EvalError> {
        let spec_result = evaluate_do_spec_expr(self, args[spec_index + 1].clone());
        let spec_expr = self.suspend_with_frame(spec_result, {
            let args = args.clone();
            move |session, spec_expr| session.finish_do_spec_eval(args, spec_index, spec_expr)
        })?;

        self.finish_do_spec_eval(args, spec_index, spec_expr)
    }

    fn finish_do_spec_eval(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec_expr: Expr,
    ) -> Result<Expr, EvalError> {
        let spec = match parse_do_spec(self, &spec_expr, IterationConsumer::Do)? {
            Some(spec) => spec,
            None => {
                return Ok(self.invalid_call_with_arg_form(
                    "Do",
                    &args,
                    "expected a supported repetition count or iterator specification",
                ));
            }
        };

        self.resume_do_iterations(args, spec_index, spec, 0)
    }

    fn resume_do_iterations(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        mut iteration: usize,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &spec {
                DoSpec::Count(count) => {
                    if iteration >= *count {
                        return Ok(system_expr("Null"));
                    }
                    let body = build_iteration_body_or_nested(&args, spec_index, "Do");
                    self.evaluate(body)
                }
                DoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(iteration).cloned() else {
                        return Ok(system_expr("Null"));
                    };
                    let body = build_iteration_body_or_nested(&args, spec_index, "Do");
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(_) | Err(EvalError::Continue) => {
                    iteration += 1;
                }
                Err(EvalError::Break) => return Ok(system_expr("Null")),
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, outcome| {
                        session
                            .resume_do_after_iteration(args, spec_index, spec, iteration, outcome)
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_do_after_iteration(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        iteration: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) | Err(EvalError::Continue) => {
                self.resume_do_iterations(args, spec_index, spec, iteration + 1)
            }
            Err(EvalError::Break) => Ok(system_expr("Null")),
            Err(error) => Err(error),
        }
    }

    fn resume_table_after_spec_eval(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
    ) -> Result<Expr, EvalError> {
        let spec_result = evaluate_do_spec_expr(self, args[spec_index + 1].clone());
        let spec_expr = self.suspend_with_frame(spec_result, {
            let args = args.clone();
            move |session, spec_expr| session.finish_table_spec_eval(args, spec_index, spec_expr)
        })?;

        self.finish_table_spec_eval(args, spec_index, spec_expr)
    }

    fn finish_table_spec_eval(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec_expr: Expr,
    ) -> Result<Expr, EvalError> {
        if spec_index + 2 >= args.len() {
            if let Some(result) =
                eval_single_spec_constant_table_fast_path(self, &args[0], &spec_expr)?
            {
                return Ok(result);
            }
        }

        let spec = match parse_do_spec(self, &spec_expr, IterationConsumer::Table)? {
            Some(spec) => spec,
            None => {
                return Ok(self.invalid_call_with_arg_form(
                    "Table",
                    &args,
                    "expected a supported repetition count or iterator specification",
                ));
            }
        };
        let capacity = spec.iteration_count();
        if spec_index + 2 >= args.len() {
            match eval_single_spec_table_fast_path(self, &args[0], spec) {
                Ok(result) => return Ok(result),
                Err(spec_after_fast_path) => {
                    return self.resume_table_iterations(
                        args,
                        spec_index,
                        spec_after_fast_path,
                        0,
                        Vec::with_capacity(capacity),
                    );
                }
            }
        }

        self.resume_table_iterations(args, spec_index, spec, 0, Vec::with_capacity(capacity))
    }

    fn resume_table_iterations(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        mut iteration: usize,
        mut collected: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        loop {
            let outcome = match &spec {
                DoSpec::Count(count) => {
                    if iteration >= *count {
                        return Ok(Expr::call(system_expr("List"), collected));
                    }
                    let body = build_iteration_body_or_nested(&args, spec_index, "Table");
                    self.evaluate(body)
                }
                DoSpec::Iterator { symbol, values } => {
                    let Some(value) = values.get(iteration).cloned() else {
                        return Ok(Expr::call(system_expr("List"), collected));
                    };
                    let body = build_iteration_body_or_nested(&args, spec_index, "Table");
                    self.evaluate_loop_iterator_body(*symbol, value, body)
                }
            };

            match outcome {
                Ok(value) => {
                    collected.push(value);
                    iteration += 1;
                }
                Err(EvalError::InputRequired { request }) => {
                    self.push_suspension_frame(move |session, outcome| {
                        session.resume_table_after_iteration(
                            args, spec_index, spec, iteration, collected, outcome,
                        )
                    });
                    return Err(EvalError::InputRequired { request });
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn resume_table_after_iteration(
        &mut self,
        args: Vec<Expr>,
        spec_index: usize,
        spec: DoSpec,
        iteration: usize,
        mut collected: Vec<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                collected.push(value);
                self.resume_table_iterations(args, spec_index, spec, iteration + 1, collected)
            }
            Err(error) => Err(error),
        }
    }

    pub(in crate::runtime::control_data::branching_loops) fn evaluate_loop_iterator_body(
        &mut self,
        symbol: Symbol,
        value: Expr,
        body: Expr,
    ) -> Result<Expr, EvalError> {
        if self.dynamic_scope_symbol_state_is_clean(&symbol) {
            return self.evaluate_loop_iterator_body_with_dynamic_value(symbol, value, body);
        }

        let saved_state = self.capture_loop_iterator_symbol_state(&symbol);
        self.install_loop_iterator_symbol_value(&symbol, value, &saved_state);

        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, {
            let saved_state = saved_state.clone();
            move |session, outcome| session.finish_loop_iterator_scope(symbol, saved_state, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_loop_iterator_scope(symbol, saved_state, outcome),
        }
    }

    fn evaluate_loop_iterator_body_with_dynamic_value(
        &mut self,
        symbol: Symbol,
        value: Expr,
        body: Expr,
    ) -> Result<Expr, EvalError> {
        let dynamic_depth = self.push_dynamic_own_value_binding(symbol, Some(value));
        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, move |session, outcome| {
            session.finish_loop_iterator_dynamic_scope(symbol, dynamic_depth, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_loop_iterator_dynamic_scope(symbol, dynamic_depth, outcome),
        }
    }

    fn finish_loop_iterator_dynamic_scope(
        &mut self,
        symbol: Symbol,
        dynamic_depth: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.truncate_dynamic_own_value_bindings(dynamic_depth);
        self.clear_dynamic_scope_symbol_side_effects(&symbol);
        outcome
    }

    fn capture_loop_iterator_symbol_state(&self, symbol: &Symbol) -> BlockSymbolState {
        BlockSymbolState {
            definition: self.definitions.get(symbol).cloned(),
            attributes: self.attributes.get(symbol).copied(),
            options: self.options.get(symbol).cloned(),
            syntax_information: self.syntax_information.get(symbol).cloned(),
            messages: self
                .message_definitions
                .iter()
                .filter(|(key, _)| key.symbol == *symbol)
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        }
    }

    fn install_loop_iterator_symbol_value(
        &mut self,
        symbol: &Symbol,
        value: Expr,
        saved_state: &BlockSymbolState,
    ) {
        let removed_up_values = saved_state
            .definition
            .as_ref()
            .is_some_and(|definition| !definition.up_values.is_empty());

        self.definitions.insert(
            *symbol,
            SymbolDefinition {
                own_value: Some(value),
                ..SymbolDefinition::default()
            },
        );
        self.bump_evaluation_epoch_for_symbol(symbol);

        if removed_up_values {
            self.refresh_has_up_value_definitions();
        }

        if self.attributes.remove(symbol).is_some() {
            self.bump_attribute_revision();
        }
        self.options.remove(symbol);
        self.syntax_information.remove(symbol);

        let messages_before = self.message_definitions.len();
        self.message_definitions
            .retain(|key, _| key.symbol != *symbol);
        if self.message_definitions.len() != messages_before {
            self.rebuild_message_symbol_counts();
        }
    }

    fn finish_loop_iterator_scope(
        &mut self,
        symbol: Symbol,
        saved_state: BlockSymbolState,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let current_had_up_values = self
            .definitions
            .get(&symbol)
            .is_some_and(|definition| !definition.up_values.is_empty());
        let restored_has_up_values = saved_state
            .definition
            .as_ref()
            .is_some_and(|definition| !definition.up_values.is_empty());

        match saved_state.definition {
            Some(definition) => {
                self.definitions.insert(symbol, definition);
            }
            None => {
                self.definitions.remove(&symbol);
            }
        }
        self.bump_evaluation_epoch_for_symbol(&symbol);

        if current_had_up_values || restored_has_up_values {
            self.refresh_has_up_value_definitions();
        }

        let current_attributes = self.attributes.get(&symbol).copied();
        if current_attributes != saved_state.attributes {
            match saved_state.attributes {
                Some(attributes) => {
                    self.attributes.insert(symbol, attributes);
                }
                None => {
                    self.attributes.remove(&symbol);
                }
            }
            self.bump_attribute_revision();
        }

        match saved_state.options {
            Some(options) => {
                self.options.insert(symbol, options);
            }
            None => {
                self.options.remove(&symbol);
            }
        }

        match saved_state.syntax_information {
            Some(syntax_information) => {
                self.syntax_information.insert(symbol, syntax_information);
            }
            None => {
                self.syntax_information.remove(&symbol);
            }
        }

        let messages_before = self.message_definitions.len();
        self.message_definitions
            .retain(|key, _| key.symbol != symbol);
        let messages_changed =
            self.message_definitions.len() != messages_before || !saved_state.messages.is_empty();
        for (key, definition) in saved_state.messages {
            self.message_definitions.insert(key, definition);
        }
        if messages_changed {
            self.rebuild_message_symbol_counts();
        }

        outcome
    }
}
