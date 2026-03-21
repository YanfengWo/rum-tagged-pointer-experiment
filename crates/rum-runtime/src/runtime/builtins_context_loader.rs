use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct NeedsLoadConfig {
    context: String,
    load_spec: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BeginPackageConfig {
    target_context: String,
    dependencies: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoaderSpecNormalizationError {
    InvalidLoadTarget,
    InvalidNeedsContext,
    InvalidNeedsLoadTarget,
    InvalidBeginPackageContext,
    InvalidBeginPackageDependencies,
    WrongArgCount {
        expected: &'static str,
        actual: usize,
    },
}

impl LoaderSpecNormalizationError {
    fn message(self) -> Option<&'static str> {
        match self {
            Self::InvalidLoadTarget => Some("expected a string or File[\"...\"] load target"),
            Self::InvalidNeedsContext => Some(
                "expected a string context name, optionally followed by a string or File[\"...\"] load target",
            ),
            Self::InvalidNeedsLoadTarget => {
                Some("expected a string or File[\"...\"] as the explicit load target")
            }
            Self::InvalidBeginPackageContext => Some(
                "expected a string context name, optionally followed by a list of string dependencies",
            ),
            Self::InvalidBeginPackageDependencies => {
                Some("expected the dependency list to be a list of strings")
            }
            Self::WrongArgCount { .. } => None,
        }
    }
}

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
        let spec = match self.compile_load_target_from_resolved_args(args) {
            Ok(spec) => spec,
            Err(LoaderSpecNormalizationError::WrongArgCount { .. }) => {
                return Ok(self.invalid_call_with_exact_arg_count("FindFile", args, 1));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "FindFile",
                    args,
                    error
                        .message()
                        .expect("loader target normalization errors should have messages"),
                ));
            }
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
        let spec = match self.compile_load_target_from_resolved_args(args) {
            Ok(spec) => spec,
            Err(LoaderSpecNormalizationError::WrongArgCount { .. }) => {
                return Ok(self.invalid_call_with_exact_arg_count("Get", args, 1));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "Get",
                    args,
                    error
                        .message()
                        .expect("loader target normalization errors should have messages"),
                ));
            }
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
        let config = match self.compile_needs_call_from_resolved_args(args) {
            Ok(config) => config,
            Err(LoaderSpecNormalizationError::WrongArgCount { expected, actual }) => {
                return Ok(self.invalid_call_with_message(
                    "Needs",
                    args,
                    "argx",
                    format!("expected {expected} argument(s), got {actual}"),
                ));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "Needs",
                    args,
                    error
                        .message()
                        .expect("loader spec normalization errors should have messages"),
                ));
            }
        };

        if self
            .loaded_packages
            .iter()
            .any(|loaded| loaded == &config.context)
        {
            return Ok(system_expr("Null"));
        }

        let result = self.eval_get(&[Expr::string(config.load_spec.clone())])?;

        if is_system_symbol_name(&result, "$Failed") {
            self.push_message(
                "Needs",
                "load",
                format!("context `{}` could not be loaded", config.context),
            );
            return Ok(system_expr("$Failed"));
        }

        self.mark_package_loaded(&config.context);
        self.context_path = prepend_context(&config.context, &self.context_path);
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
        let config = match self.compile_begin_package_call_from_resolved_args(args) {
            Ok(config) => config,
            Err(LoaderSpecNormalizationError::WrongArgCount { expected, actual }) => {
                return Ok(self.invalid_call_with_message(
                    "BeginPackage",
                    args,
                    "argx",
                    format!("expected {expected} argument(s), got {actual}"),
                ));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "BeginPackage",
                    args,
                    error
                        .message()
                        .expect("loader spec normalization errors should have messages"),
                ));
            }
        };

        let result = self.begin_package_context(config.target_context.clone());

        for context in &config.dependencies {
            self.eval_needs(&[Expr::string(context.clone())])?;
        }

        self.context_path = prepend_context(&config.target_context, &self.context_path);

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

    fn compile_load_target_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<String, LoaderSpecNormalizationError> {
        match args {
            [spec] => {
                load_spec_from_expr(spec).ok_or(LoaderSpecNormalizationError::InvalidLoadTarget)
            }
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1",
                actual: args.len(),
            }),
        }
    }

    fn compile_needs_call_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<NeedsLoadConfig, LoaderSpecNormalizationError> {
        match args {
            [Expr::Atom(Atom::String(context))] => {
                let context = normalize_context_name(context);
                Ok(NeedsLoadConfig {
                    load_spec: context.clone(),
                    context,
                })
            }
            [Expr::Atom(Atom::String(context)), spec] => Ok(NeedsLoadConfig {
                context: normalize_context_name(context),
                load_spec: load_spec_from_expr(spec)
                    .ok_or(LoaderSpecNormalizationError::InvalidNeedsLoadTarget)?,
            }),
            [_, ..] if args.len() <= 2 => Err(LoaderSpecNormalizationError::InvalidNeedsContext),
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1 or 2",
                actual: args.len(),
            }),
        }
    }

    fn compile_begin_package_call_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<BeginPackageConfig, LoaderSpecNormalizationError> {
        match args {
            [Expr::Atom(Atom::String(context))] => Ok(BeginPackageConfig {
                target_context: expand_context_spec(context, &self.current_context),
                dependencies: Vec::new(),
            }),
            [Expr::Atom(Atom::String(context)), dependencies] => Ok(BeginPackageConfig {
                target_context: expand_context_spec(context, &self.current_context),
                dependencies: expr_as_string_list(dependencies)
                    .ok_or(LoaderSpecNormalizationError::InvalidBeginPackageDependencies)?
                    .into_iter()
                    .map(normalize_context_name)
                    .collect(),
            }),
            [_, ..] if args.len() <= 2 => {
                Err(LoaderSpecNormalizationError::InvalidBeginPackageContext)
            }
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1 or 2",
                actual: args.len(),
            }),
        }
    }
}
