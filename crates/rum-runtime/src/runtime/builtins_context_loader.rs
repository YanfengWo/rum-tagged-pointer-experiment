use super::*;

impl Session {
    pub(super) fn eval_context(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(Expr::string(self.current_context.clone())),
            [Expr::Atom(Atom::Symbol(symbol))] => Ok(Expr::string(symbol.context().to_string())),
            [Expr::Atom(Atom::String(raw_name))] => self
                .resolve_symbol_name(raw_name, false)
                .map(|symbol| Expr::string(symbol.context().to_string()))
                .map_or_else(|| Ok(Expr::call(system_expr("Context"), args.to_vec())), Ok),
            [_] => Ok(self.invalid_call_with_arg_form(
                "Context",
                args,
                "expected a symbol or string argument",
            )),
            _ => Ok(self.invalid_call_with_message(
                "Context",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }

    pub(super) fn eval_find_file(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FindFile", args, 1));
        };

        let Some(spec) = load_spec_from_expr(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "FindFile",
                args,
                "expected a string or File[\"...\"] load target",
            ));
        };

        Ok(match self.find_load_target(&spec) {
            Some((path, _)) => Expr::string(path.display().to_string()),
            None => {
                self.emit_loader_message(
                    "FindFile",
                    &EvalError::LoadPathNotFound {
                        target: spec.clone(),
                        searched: self.load_target_candidates(&spec).1,
                    },
                );
                system_expr("$Failed")
            }
        })
    }

    pub(super) fn eval_get(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Get", args, 1));
        };

        let Some(spec) = load_spec_from_expr(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "Get",
                args,
                "expected a string or File[\"...\"] load target",
            ));
        };

        let path = match self.resolve_load_target(&spec) {
            Ok(path) => path,
            Err(err) => {
                self.emit_loader_message("Get", &err);
                return Ok(system_expr("$Failed"));
            }
        };

        match self.eval_file(path) {
            Ok(outcome) => {
                self.messages.extend(outcome.messages);
                Ok(outcome.value)
            }
            Err(err) => {
                self.emit_loader_message("Get", &err);
                Ok(system_expr("$Failed"))
            }
        }
    }

    pub(super) fn eval_needs(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (context, load_spec) = match args {
            [Expr::Atom(Atom::String(context))] => (
                normalize_context_name(context),
                normalize_context_name(context),
            ),
            [Expr::Atom(Atom::String(context)), spec] => {
                let Some(spec) = load_spec_from_expr(spec) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Needs",
                        args,
                        "expected a string or File[\"...\"] as the explicit load target",
                    ));
                };

                (normalize_context_name(context), spec)
            }
            [_, ..] if args.len() <= 2 => {
                return Ok(self.invalid_call_with_arg_form(
                    "Needs",
                    args,
                    "expected a string context name, optionally followed by a string or File[\"...\"] load target",
                ));
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "Needs",
                    args,
                    "argx",
                    format!("expected 1 or 2 argument(s), got {}", args.len()),
                ));
            }
        };

        if self.loaded_packages.iter().any(|loaded| loaded == &context) {
            return Ok(system_expr("Null"));
        }

        let result = self.eval_get(&[Expr::string(load_spec)])?;

        if is_system_symbol_name(&result, "$Failed") {
            self.push_message(
                "Needs",
                "load",
                format!("context `{context}` could not be loaded"),
            );
            return Ok(system_expr("$Failed"));
        }

        self.mark_package_loaded(&context);
        self.context_path = prepend_context(&context, &self.context_path);
        Ok(system_expr("Null"))
    }

    pub(super) fn eval_begin(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [context] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Begin", args, 1));
        };

        let Expr::Atom(Atom::String(context)) = context else {
            return Ok(self.invalid_call_with_arg_form(
                "Begin",
                args,
                "expected a string context name",
            ));
        };

        let target = expand_context_spec(context, &self.current_context);
        Ok(self.begin_context(target))
    }

    pub(super) fn eval_end(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("End", args, 0));
        }

        Ok(self.end_context().unwrap_or_else(|| {
            self.invalid_call_with_message(
                "End",
                &[],
                "context",
                "no matching Begin[] frame is active",
            )
        }))
    }

    pub(super) fn eval_begin_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (context, needs) = match args {
            [Expr::Atom(Atom::String(context))] => (context, Vec::new()),
            [Expr::Atom(Atom::String(context)), needs] => {
                let Some(needs) = expr_as_string_list(needs) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "BeginPackage",
                        args,
                        "expected the dependency list to be a list of strings",
                    ));
                };

                (context, needs)
            }
            [_, ..] if args.len() <= 2 => {
                return Ok(self.invalid_call_with_arg_form(
                    "BeginPackage",
                    args,
                    "expected a string context name, optionally followed by a list of string dependencies",
                ));
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "BeginPackage",
                    args,
                    "argx",
                    format!("expected 1 or 2 argument(s), got {}", args.len()),
                ));
            }
        };

        let target = expand_context_spec(context, &self.current_context);
        let result = self.begin_package_context(target.clone());

        for need in needs {
            let context = normalize_context_name(need);
            self.eval_needs(&[Expr::string(context)])?;
        }

        self.context_path = prepend_context(&target, &self.context_path);

        Ok(result)
    }

    pub(super) fn eval_end_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("EndPackage", args, 0));
        }

        Ok(self.end_package_context().unwrap_or_else(|| {
            self.invalid_call_with_message(
                "EndPackage",
                &[],
                "context",
                "no matching BeginPackage[] frame is active",
            )
        }))
    }
}
