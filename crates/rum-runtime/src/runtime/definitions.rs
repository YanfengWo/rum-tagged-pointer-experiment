use super::*;

impl Session {
    pub(super) fn eval_normalize_rules(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => self.resume_normalize_rules(expr.clone()),
            _ => Ok(self.invalid_call_with_exact_arg_count("NormalizeRules", args, 1)),
        }
    }

    pub(super) fn eval_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Set", args, 2));
        }

        let lhs = args[0].clone();
        let rhs_result = self.evaluate(args[1].clone());
        let rhs = self.suspend_with_frame(rhs_result, {
            let lhs = lhs.clone();
            move |session, rhs| session.resume_set_with_rhs(lhs, rhs)
        })?;

        self.resume_set_with_rhs(lhs, rhs)
    }

    pub(super) fn eval_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("SetDelayed", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();
        self.finish_definition_assignment("SetDelayed", lhs, rhs, true)
    }

    pub(super) fn eval_rule(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Rule", args, 2));
        }

        let lhs = args[0].clone();
        let rhs_result = self.evaluate(args[1].clone());
        let rhs = self.suspend_with_frame(rhs_result, {
            let lhs = lhs.clone();
            move |session, rhs| session.resume_rule_with_rhs(lhs, rhs)
        })?;

        self.resume_rule_with_rhs(lhs, rhs)
    }

    pub(super) fn eval_rule_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("RuleDelayed", args, 2));
        }

        Ok(Expr::call(system_expr("RuleDelayed"), args.to_vec()))
    }

    fn resume_normalize_rules(&mut self, original: Expr) -> Result<Expr, EvalError> {
        self.drive_rewrite_rule_exprs_owned(original.clone(), move |session, rule_exprs| {
            match rule_exprs {
                Some(rule_exprs) => {
                    let rules = session
                        .compile_rewrite_rules_from_normalized_exprs(rule_exprs)
                        .expect("normalized rewrite rule expressions should recompile");
                    Ok(Self::normalized_rules_expr(&rules))
                }
                None => Ok(session.invalid_call_with_arg_form(
                    "NormalizeRules",
                    &[original],
                    "expected a rule, a rule list, or a symbol that evaluates to rules",
                )),
            }
        })
    }

    fn resume_set_with_rhs(&mut self, lhs: Expr, rhs: Expr) -> Result<Expr, EvalError> {
        self.finish_definition_assignment("Set", lhs, rhs, false)
    }

    fn resume_rule_with_rhs(&mut self, lhs: Expr, rhs: Expr) -> Result<Expr, EvalError> {
        Ok(Expr::call(system_expr("Rule"), vec![lhs, rhs]))
    }

    fn finish_definition_assignment(
        &mut self,
        symbol: &str,
        lhs: Expr,
        rhs: Expr,
        delayed: bool,
    ) -> Result<Expr, EvalError> {
        if !delayed {
            if is_path_symbol_expr(&lhs) {
                let Some(paths) = expr_as_string_list(&rhs) else {
                    return Ok(self.invalid_call_with_arg_form(
                        symbol,
                        &[lhs, rhs],
                        "expected $Path to be assigned a list of string paths",
                    ));
                };
                self.package_search_paths =
                    dedup_paths(paths.into_iter().map(PathBuf::from).collect());
                return Ok(rhs);
            }

            if let Some(target) = self.normalize_options_target_expr(&lhs) {
                if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
                    return Ok(rhs);
                }

                let options = match self.compile_assigned_options_from_resolved(&rhs) {
                    Ok(options) => options,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            symbol,
                            &[lhs, rhs],
                            error.message(),
                        ));
                    }
                };

                self.set_options_for_symbol(target, options);
                return Ok(rhs);
            }

            if let Some(target) = self.normalize_syntax_information_target_expr(&lhs) {
                if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
                    return Ok(rhs);
                }

                let syntax_information =
                    match self.compile_assigned_syntax_information_from_resolved(&rhs) {
                        Ok(syntax_information) => syntax_information,
                        Err(error) => {
                            return Ok(self.invalid_call_with_arg_form(
                                symbol,
                                &[lhs, rhs],
                                error.message(),
                            ));
                        }
                    };

                self.set_syntax_information_for_symbol(target, syntax_information);
                return Ok(rhs);
            }
        }

        if !delayed {
            if let Some(value) =
                self.try_set_list_destructuring_lvalue(symbol, lhs.clone(), rhs.clone())?
            {
                return Ok(value);
            }

            if let Some(value) = self.try_set_association_lvalue(lhs.clone(), rhs.clone())? {
                return Ok(value);
            }
        }

        let lhs = self.normalize_definition_lhs_call_heads(&lhs);

        match self.normalize_definition_assignment_target(&lhs) {
            Some(DefinitionAssignmentTarget::Message(key)) => {
                if !self.ensure_symbol_not_protected_for_write(symbol, &key.symbol) {
                    return Ok(if delayed { system_expr("Null") } else { rhs });
                }
                self.define_message(key, rhs.clone(), delayed);
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            Some(DefinitionAssignmentTarget::SyntaxInformation(target)) => {
                if !self.ensure_symbol_not_protected_for_write(symbol, &target) {
                    return Ok(if delayed { system_expr("Null") } else { rhs });
                }
                Ok(Expr::call(system_expr(symbol), vec![lhs, rhs]))
            }
            Some(DefinitionAssignmentTarget::Definition(target)) => {
                if !self.ensure_symbol_not_protected_for_write(symbol, target.symbol()) {
                    return Ok(if delayed { system_expr("Null") } else { rhs });
                }
                self.store_definition_target(target, lhs.clone(), rhs.clone(), delayed);
                Ok(if delayed { system_expr("Null") } else { rhs })
            }
            None => Ok(Expr::call(system_expr(symbol), vec![lhs, rhs])),
        }
    }

    fn try_set_list_destructuring_lvalue(
        &mut self,
        symbol: &str,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        if list_expr_args(&lhs).is_none() {
            return Ok(None);
        }

        if !list_destructuring_shapes_match(&lhs, &rhs) {
            self.push_message(
                symbol,
                "shape",
                format!("Lists {lhs} and {rhs} are not the same shape."),
            );
            return Ok(Some(rhs));
        }

        self.store_list_destructuring_assignments(symbol, &lhs, &rhs)?;
        Ok(Some(rhs))
    }

    fn store_list_destructuring_assignments(
        &mut self,
        symbol: &str,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<(), EvalError> {
        if let Some(lhs_items) = list_expr_args(lhs) {
            let rhs_items = list_expr_args(rhs).expect("destructuring shape checked before store");
            for (lhs_item, rhs_item) in lhs_items.iter().zip(rhs_items) {
                self.store_list_destructuring_assignments(symbol, lhs_item, rhs_item)?;
            }
            return Ok(());
        }

        self.finish_definition_assignment(symbol, lhs.clone(), rhs.clone(), false)?;
        Ok(())
    }

    fn normalize_definition_lhs_call_heads(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::Atom(atom) => Expr::Atom(atom.clone()),
            Expr::Call { head, args } => Expr::call(
                self.normalize_definition_lhs_head_expr(head),
                args.iter()
                    .map(|arg| self.normalize_definition_lhs_call_heads(arg))
                    .collect(),
            ),
        }
    }

    fn normalize_definition_lhs_head_expr(&self, head: &Expr) -> Expr {
        let normalized_head = self.normalize_definition_lhs_call_heads(head);
        self.resolve_definition_lhs_head_value(normalized_head, &mut BTreeSet::new())
    }

    fn resolve_definition_lhs_head_value(
        &self,
        expr: Expr,
        visited: &mut BTreeSet<Symbol>,
    ) -> Expr {
        let Expr::Atom(Atom::Symbol(symbol)) = expr else {
            return expr;
        };

        if !visited.insert(symbol.clone()) {
            return Expr::Atom(Atom::Symbol(symbol));
        }

        let Some(value) = self.definitions.get(&symbol).and_then(|definition| {
            definition
                .own_value
                .clone()
                .or_else(|| definition.delayed_own_value.clone())
        }) else {
            return Expr::Atom(Atom::Symbol(symbol));
        };

        let normalized_value = self.normalize_definition_lhs_call_heads(&value);
        self.resolve_definition_lhs_head_value(normalized_value, visited)
    }
}

fn is_path_symbol_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(Atom::Symbol(symbol)) if symbol.context() == SYSTEM_CONTEXT && symbol.name() == "$Path")
}

fn list_expr_args(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

fn list_destructuring_shapes_match(lhs: &Expr, rhs: &Expr) -> bool {
    let Some(lhs_items) = list_expr_args(lhs) else {
        return true;
    };

    let Some(rhs_items) = list_expr_args(rhs) else {
        return false;
    };

    lhs_items.len() == rhs_items.len()
        && lhs_items
            .iter()
            .zip(rhs_items)
            .all(|(lhs_item, rhs_item)| list_destructuring_shapes_match(lhs_item, rhs_item))
}
