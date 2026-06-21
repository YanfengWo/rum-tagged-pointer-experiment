use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum DoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Vec<Expr> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IterationConsumer {
    Do,
    Table,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum IteratorListTemplateArg {
    Iterator,
    Constant(Expr),
}

impl DoSpec {
    fn iteration_count(&self) -> usize {
        match self {
            Self::Count(count) => *count,
            Self::Iterator { values, .. } => values.len(),
        }
    }
}

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

    fn evaluate_loop_iterator_body(
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
fn evaluate_do_spec_expr(session: &mut Session, raw_spec: Expr) -> Result<Expr, EvalError> {
    match raw_spec {
        Expr::Call { head, args } if is_system_symbol_name(&head, heads::structure::LIST) => {
            let evaluated = if let Some((first, rest)) = args.split_first() {
                if expr_as_symbol(first).is_some() {
                    let mut normalized = Vec::with_capacity(args.len());
                    normalized.push(Expr::call(system_expr("Hold"), vec![first.clone()]));
                    normalized.extend(rest.iter().cloned());
                    Expr::call(system_expr("List"), normalized)
                } else {
                    Expr::call_with_shared_args(system_expr("List"), args)
                }
            } else {
                Expr::call_with_shared_args(system_expr("List"), args)
            };

            session.evaluate(evaluated)
        }
        other => session.evaluate(other),
    }
}

fn parse_do_spec(
    session: &Session,
    expr: &Expr,
    consumer: IterationConsumer,
) -> Result<Option<DoSpec>, EvalError> {
    if let Some(count) = expr_as_non_negative_bigint(expr) {
        return Ok(Some(DoSpec::Count(checked_iteration_count(
            session, &count, consumer,
        )?)));
    }

    let Expr::Call { head, args } = expr else {
        return Ok(None);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Ok(None);
    }

    Ok(match args.as_slice() {
        [count_expr] => {
            let Some(count) = expr_as_non_negative_bigint(count_expr) else {
                return Ok(None);
            };
            Some(DoSpec::Count(checked_iteration_count(
                session, &count, consumer,
            )?))
        }
        [symbol_expr, values_expr] => {
            let Some(symbol) = parse_held_loop_symbol(symbol_expr) else {
                return Ok(None);
            };

            if let Expr::Call { head, args } = values_expr {
                if is_system_symbol_name(head, heads::structure::LIST) {
                    checked_iteration_count(session, &BigInt::from(args.len()), consumer)?;
                    return Ok(Some(DoSpec::Iterator {
                        symbol,
                        values: args.to_vec(),
                    }));
                }
            }

            let Some(end) = expr_as_signed_bigint(values_expr) else {
                return Ok(None);
            };
            let values = checked_integer_range_values(
                session,
                BigInt::from(1_u8),
                end,
                BigInt::from(1_u8),
                consumer,
            )?;
            Some(DoSpec::Iterator { symbol, values })
        }
        [symbol_expr, start_expr, end_expr] => {
            let (Some(symbol), Some(start), Some(end)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
            ) else {
                return Ok(None);
            };
            let values =
                checked_integer_range_values(session, start, end, BigInt::from(1_u8), consumer)?;
            Some(DoSpec::Iterator { symbol, values })
        }
        [symbol_expr, start_expr, end_expr, step_expr] => {
            let (Some(symbol), Some(start), Some(end), Some(step)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
                expr_as_signed_bigint(step_expr),
            ) else {
                return Ok(None);
            };
            if step == BigInt::from(0_u8) {
                return Ok(None);
            }

            let values = checked_integer_range_values(session, start, end, step, consumer)?;
            Some(DoSpec::Iterator { symbol, values })
        }
        _ => None,
    })
}

fn build_iteration_body_or_nested(args: &[Expr], spec_index: usize, nested_head: &str) -> Expr {
    if spec_index + 2 >= args.len() {
        args[0].clone()
    } else {
        let mut nested_args = Vec::with_capacity(args.len() - spec_index - 1);
        nested_args.push(args[0].clone());
        nested_args.extend(args[(spec_index + 2)..].iter().cloned());
        Expr::call(system_expr(nested_head), nested_args)
    }
}

fn eval_single_spec_constant_table_fast_path(
    session: &Session,
    body: &Expr,
    spec_expr: &Expr,
) -> Result<Option<Expr>, EvalError> {
    if !table_fast_path_preserves_observability(session)
        || !table_body_is_repeated_constant(session, body)
    {
        return Ok(None);
    }

    let Some(count) = parse_do_spec_iteration_count(session, spec_expr, IterationConsumer::Table)?
    else {
        return Ok(None);
    };

    Ok(Some(Expr::call(
        system_expr("List"),
        vec![body.clone(); count],
    )))
}

fn eval_single_spec_table_fast_path(
    session: &Session,
    body: &Expr,
    spec: DoSpec,
) -> Result<Expr, DoSpec> {
    if !table_fast_path_preserves_observability(session) {
        return Err(spec);
    }

    if table_body_is_repeated_constant(session, body) {
        return Ok(Expr::call(
            system_expr("List"),
            vec![body.clone(); spec.iteration_count()],
        ));
    }

    let DoSpec::Iterator { symbol, values } = spec else {
        return Err(spec);
    };
    if expr_as_symbol(body).is_some_and(|body_symbol| *body_symbol == symbol) {
        return Ok(Expr::call(system_expr("List"), values));
    }

    match eval_single_spec_integer_plus_table_fast_path(session, body, symbol, values) {
        Ok(result) => Ok(result),
        Err(DoSpec::Iterator { symbol, values }) => {
            eval_single_spec_integer_list_template_table_fast_path(session, body, symbol, values)
        }
        Err(spec) => Err(spec),
    }
}

fn eval_single_spec_integer_plus_table_fast_path(
    session: &Session,
    body: &Expr,
    symbol: Symbol,
    values: Vec<Expr>,
) -> Result<Expr, DoSpec> {
    let Some(offset) = iterator_integer_plus_offset(session, body, &symbol) else {
        return Err(DoSpec::Iterator { symbol, values });
    };
    if !values
        .iter()
        .all(|value| matches!(value, Expr::Atom(Atom::Integer(_))))
    {
        return Err(DoSpec::Iterator { symbol, values });
    }

    let items = values
        .into_iter()
        .map(|value| match value {
            Expr::Atom(Atom::Integer(integer)) => Expr::integer(integer + offset.clone()),
            _ => unreachable!("values were prechecked as integers"),
        })
        .collect();
    Ok(Expr::call(system_expr("List"), items))
}

fn eval_single_spec_integer_list_template_table_fast_path(
    session: &Session,
    body: &Expr,
    symbol: Symbol,
    values: Vec<Expr>,
) -> Result<Expr, DoSpec> {
    let Some(template) = iterator_integer_list_template(session, body, &symbol) else {
        return Err(DoSpec::Iterator { symbol, values });
    };
    if !values
        .iter()
        .all(|value| matches!(value, Expr::Atom(Atom::Integer(_))))
    {
        return Err(DoSpec::Iterator { symbol, values });
    }

    let list_head = SharedExpr::new(system_expr("List"));
    let items = values
        .into_iter()
        .map(|value| instantiate_integer_list_template(&template, value, &list_head))
        .collect();
    Ok(Expr::call_shared(list_head, SharedArgs::new(items)))
}

fn iterator_integer_list_template(
    session: &Session,
    body: &Expr,
    symbol: &Symbol,
) -> Option<Vec<IteratorListTemplateArg>> {
    if !system_builtin_symbol_has_default_runtime_semantics(session, "List") {
        return None;
    }

    let Expr::Call { head, args } = body else {
        return None;
    };
    if !is_system_symbol_name(head, "List")
        || !args
            .iter()
            .all(|arg| iterator_list_template_arg_is_plain(arg, symbol))
    {
        return None;
    }

    args.iter()
        .map(|arg| iterator_list_template_arg(arg, symbol))
        .collect()
}

fn iterator_list_template_arg(arg: &Expr, symbol: &Symbol) -> Option<IteratorListTemplateArg> {
    if expr_as_symbol(arg).is_some_and(|arg_symbol| arg_symbol == symbol) {
        return Some(IteratorListTemplateArg::Iterator);
    }
    iteration_body_is_self_evaluating_atom(arg)
        .then(|| IteratorListTemplateArg::Constant(arg.clone()))
}

fn iterator_list_template_arg_is_plain(arg: &Expr, symbol: &Symbol) -> bool {
    iterator_list_template_arg(arg, symbol).is_some()
}

fn instantiate_integer_list_template(
    template: &[IteratorListTemplateArg],
    value: Expr,
    list_head: &SharedExpr,
) -> Expr {
    let mut remaining_iterator_slots = template
        .iter()
        .filter(|arg| matches!(arg, IteratorListTemplateArg::Iterator))
        .count();
    let mut value = Some(value);
    let items = template
        .iter()
        .map(|arg| match arg {
            IteratorListTemplateArg::Iterator => {
                remaining_iterator_slots -= 1;
                if remaining_iterator_slots == 0 {
                    value.take().expect("last iterator slot consumes value")
                } else {
                    value
                        .as_ref()
                        .expect("iterator value remains available before last slot")
                        .clone()
                }
            }
            IteratorListTemplateArg::Constant(expr) => expr.clone(),
        })
        .collect();
    Expr::call_shared(list_head.clone(), SharedArgs::new(items))
}

fn iterator_integer_plus_offset(session: &Session, body: &Expr, symbol: &Symbol) -> Option<BigInt> {
    if !system_builtin_symbol_has_default_runtime_semantics(session, "Plus") {
        return None;
    }

    let Expr::Call { head, args } = body else {
        return None;
    };
    if !is_system_symbol_name(head, "Plus") || args.len() != 2 {
        return None;
    }

    match args.as_slice() {
        [lhs, rhs] if expr_as_symbol(lhs).is_some_and(|lhs_symbol| lhs_symbol == symbol) => {
            expr_as_exact_integer(rhs)
        }
        [lhs, rhs] if expr_as_symbol(rhs).is_some_and(|rhs_symbol| rhs_symbol == symbol) => {
            expr_as_exact_integer(lhs)
        }
        _ => None,
    }
}

fn system_builtin_symbol_has_default_runtime_semantics(session: &Session, name: &str) -> bool {
    let symbol = Symbol::system(name);
    session.attribute_set_for_symbol(&symbol) == default_system_attribute_set(&symbol)
        && session
            .definitions
            .get(&symbol)
            .is_none_or(SymbolDefinition::is_empty)
        && !session
            .dynamic_own_values
            .iter()
            .any(|frame| frame.symbol == symbol)
        && !session.removed_symbols.contains(&symbol)
}

fn table_fast_path_preserves_observability(session: &Session) -> bool {
    !session.tracing_enabled
        && (session.monitor_scopes.is_empty() || !session.monitor_callbacks_enabled)
}

fn table_body_is_repeated_constant(session: &Session, expr: &Expr) -> bool {
    iteration_body_is_self_evaluating_atom(expr)
        || expr_as_symbol(expr).is_some_and(|symbol| {
            symbol.context() == SYSTEM_CONTEXT
                && matches!(symbol.name(), "Null" | "True" | "False")
                && system_builtin_symbol_has_default_runtime_semantics(session, symbol.name())
        })
}

fn iteration_body_is_self_evaluating_atom(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Atom(Atom::Integer(_) | Atom::Rational(_) | Atom::String(_))
    )
}

fn parse_do_spec_iteration_count(
    session: &Session,
    expr: &Expr,
    consumer: IterationConsumer,
) -> Result<Option<usize>, EvalError> {
    if let Some(count) = expr_as_non_negative_bigint(expr) {
        return Ok(Some(checked_iteration_count(session, &count, consumer)?));
    }

    let Expr::Call { head, args } = expr else {
        return Ok(None);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Ok(None);
    }

    Ok(match args.as_slice() {
        [count_expr] => {
            let Some(count) = expr_as_non_negative_bigint(count_expr) else {
                return Ok(None);
            };
            Some(checked_iteration_count(session, &count, consumer)?)
        }
        [symbol_expr, values_expr] => {
            let Some(_symbol) = parse_held_loop_symbol(symbol_expr) else {
                return Ok(None);
            };

            if let Expr::Call { head, args } = values_expr {
                if is_system_symbol_name(head, heads::structure::LIST) {
                    let count = BigInt::from(args.len());
                    return Ok(Some(checked_iteration_count(session, &count, consumer)?));
                }
            }

            let Some(end) = expr_as_signed_bigint(values_expr) else {
                return Ok(None);
            };
            checked_integer_range_iteration_count(
                session,
                &BigInt::from(1_u8),
                &end,
                &BigInt::from(1_u8),
                consumer,
            )?
        }
        [symbol_expr, start_expr, end_expr] => {
            let (Some(_symbol), Some(start), Some(end)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
            ) else {
                return Ok(None);
            };
            checked_integer_range_iteration_count(
                session,
                &start,
                &end,
                &BigInt::from(1_u8),
                consumer,
            )?
        }
        [symbol_expr, start_expr, end_expr, step_expr] => {
            let (Some(_symbol), Some(start), Some(end), Some(step)) = (
                parse_held_loop_symbol(symbol_expr),
                expr_as_signed_bigint(start_expr),
                expr_as_signed_bigint(end_expr),
                expr_as_signed_bigint(step_expr),
            ) else {
                return Ok(None);
            };
            if step == BigInt::from(0_u8) {
                return Ok(None);
            }
            checked_integer_range_iteration_count(session, &start, &end, &step, consumer)?
        }
        _ => None,
    })
}

fn parse_held_loop_symbol(expr: &Expr) -> Option<Symbol> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Hold") && args.len() == 1 => {
            expr_as_symbol(&args[0]).cloned()
        }
        _ => expr_as_symbol(expr).cloned(),
    }
}

fn expr_as_signed_bigint(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") && args.len() == 2 => {
            if is_negative_one_expr(&args[0]) {
                expr_as_exact_integer(&args[1]).map(|value| -value)
            } else if is_negative_one_expr(&args[1]) {
                expr_as_exact_integer(&args[0]).map(|value| -value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn expr_as_non_negative_bigint(expr: &Expr) -> Option<BigInt> {
    let value = expr_as_exact_integer(expr)?;
    (value >= BigInt::from(0_u8)).then_some(value)
}

fn checked_integer_range_values(
    session: &Session,
    start: BigInt,
    end: BigInt,
    step: BigInt,
    consumer: IterationConsumer,
) -> Result<Vec<Expr>, EvalError> {
    let Some(count) = integer_range_count(&start, &end, &step) else {
        return Ok(Vec::new());
    };
    let count = checked_iteration_count(session, &count, consumer)?;
    Ok(integer_range_values(start, step, count))
}

fn checked_integer_range_iteration_count(
    session: &Session,
    start: &BigInt,
    end: &BigInt,
    step: &BigInt,
    consumer: IterationConsumer,
) -> Result<Option<usize>, EvalError> {
    let Some(count) = integer_range_count(start, end, step) else {
        return Ok(None);
    };
    Ok(Some(checked_iteration_count(session, &count, consumer)?))
}

fn checked_iteration_count(
    session: &Session,
    count: &BigInt,
    consumer: IterationConsumer,
) -> Result<usize, EvalError> {
    match consumer {
        IterationConsumer::Do => session.enforce_iteration_work_limit_for_big_count(count)?,
        IterationConsumer::Table => {
            if let Some(limit) = session.current_memory_limit {
                if count > &BigInt::from(usize::MAX) {
                    return Err(EvalError::MemoryConstraintExceeded { limit });
                }
                let count = bigint_count_to_usize(count)
                    .expect("Table count was checked against usize::MAX before conversion");
                session.enforce_active_memory_limit_for_projected_bytes(
                    generated_list_memory_proxy_bytes(count),
                )?;
            }
            session.enforce_generated_element_limit_for_big_count(count)?;
            session.enforce_iteration_work_limit_for_big_count(count)?;
        }
    }

    Ok(bigint_count_to_usize(count)
        .expect("resource limit prechecks keep iteration counts representable"))
}

fn integer_range_count(start: &BigInt, end: &BigInt, step: &BigInt) -> Option<BigInt> {
    let zero = BigInt::from(0_u8);
    if step == &zero {
        return None;
    }

    let one = BigInt::from(1_u8);
    if step > &zero {
        if start > end {
            return Some(zero);
        }
        return Some(((end - start) / step) + one);
    }

    if start < end {
        return Some(zero);
    }

    Some(((start - end) / -step) + one)
}

fn integer_range_values(start: BigInt, step: BigInt, count: usize) -> Vec<Expr> {
    let mut values = Vec::with_capacity(count);
    let mut current = start;

    for _ in 0..count {
        values.push(Expr::integer(current.clone()));
        current += step.clone();
    }

    values
}

fn bigint_count_to_usize(count: &BigInt) -> Option<usize> {
    count.to_string().parse().ok()
}

fn generated_list_memory_proxy_bytes(count: usize) -> usize {
    std::mem::size_of::<Expr>().saturating_add(count.saturating_mul(std::mem::size_of::<Expr>()))
}
