use super::*;
use std::io::Write;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeclarePackageSpec {
    context: String,
    names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PersistContextSelection {
    All,
    None,
    Automatic,
    Explicit(BTreeSet<String>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PersistContextFilters {
    included: PersistContextSelection,
    excluded: PersistContextSelection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackageToolNormalizationError {
    InvalidContext,
    InvalidNames,
    InvalidPath,
    InvalidObjects,
}

impl PackageToolNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidContext => "expected a string context name",
            Self::InvalidNames => "expected a list of string symbol names",
            Self::InvalidPath => "expected a string or File[\"...\"] path",
            Self::InvalidObjects => {
                "expected a symbol, a string names pattern or context, or a list of them"
            }
        }
    }
}

impl Session {
    pub(super) fn eval_declare_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let spec = match self.compile_declare_package_spec(args) {
            Ok(spec) => spec,
            Err(PackageToolNormalizationError::InvalidContext) => {
                return Ok(self.invalid_call_with_arg_form(
                    "DeclarePackage",
                    args,
                    PackageToolNormalizationError::InvalidContext.message(),
                ));
            }
            Err(PackageToolNormalizationError::InvalidNames) => {
                return Ok(self.invalid_call_with_arg_form(
                    "DeclarePackage",
                    args,
                    PackageToolNormalizationError::InvalidNames.message(),
                ));
            }
            Err(_) => return Ok(self.invalid_call_with_exact_arg_count("DeclarePackage", args, 2)),
        };

        for name in spec.names {
            self.declared_stubs
                .insert(Symbol::new(spec.context.clone(), name));
        }

        self.context_path = prepend_context(&spec.context, &self.context_path);
        Ok(Expr::string(spec.context))
    }

    pub(super) fn eval_save(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_persist_package_state("Save", args, false)
    }

    pub(super) fn eval_dump_save(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_persist_package_state("DumpSave", args, true)
    }

    pub(super) fn eval_once_builtin(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (expr, location, _options) = match args {
            [expr] => (expr.clone(), None, None),
            [expr, location] => (expr.clone(), Some(location.clone()), None),
            [expr, location, options_expr] => {
                let options = match self.compile_assigned_options_from_resolved(options_expr) {
                    Ok(options) => options,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form("Once", args, error.message()));
                    }
                };
                (expr.clone(), Some(location.clone()), Some(options))
            }
            _ => return Ok(self.invalid_call_with_arg_range("Once", args, 1, 3)),
        };

        let cache_key = self.once_cache_key(&expr, location.as_ref());

        if let Some(value) = self.once_values.get(&cache_key).cloned() {
            return Ok(value);
        }

        let result = self.evaluate(expr);
        self.drive_evaluated_value(result, move |session, value: Expr| {
            session.once_values.insert(cache_key, value.clone());
            Ok(value)
        })
    }

    fn compile_declare_package_spec(
        &self,
        args: &[Expr],
    ) -> Result<DeclarePackageSpec, PackageToolNormalizationError> {
        match args {
            [Expr::Atom(Atom::String(context)), names_expr] => Ok(DeclarePackageSpec {
                context: expand_context_spec(context, &self.current_context),
                names: expr_as_string_list(names_expr)
                    .ok_or(PackageToolNormalizationError::InvalidNames)?,
            }),
            [_, _] => Err(PackageToolNormalizationError::InvalidContext),
            _ => Err(PackageToolNormalizationError::InvalidNames),
        }
    }

    fn eval_persist_package_state(
        &mut self,
        operator: &str,
        args: &[Expr],
        overwrite: bool,
    ) -> Result<Expr, EvalError> {
        let (path_expr, objects_expr, explicit_options) = match args {
            [path_expr, objects_expr] => (path_expr, objects_expr, None),
            [path_expr, objects_expr, options_expr] => {
                let options = match self.compile_assigned_options_from_resolved(options_expr) {
                    Ok(options) => options,
                    Err(error) => {
                        return Ok(self.invalid_call_with_arg_form(
                            operator,
                            args,
                            error.message(),
                        ));
                    }
                };
                (path_expr, objects_expr, Some(options))
            }
            _ => return Ok(self.invalid_call_with_arg_range(operator, args, 2, 3)),
        };

        let Some(path_spec) = load_spec_from_expr(path_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                operator,
                args,
                PackageToolNormalizationError::InvalidPath.message(),
            ));
        };

        let context_filters = match self.compile_persist_context_filters(explicit_options.as_ref())
        {
            Ok(filters) => filters,
            Err(message) => return Ok(self.invalid_call_with_arg_form(operator, args, message)),
        };

        let path = self.resolve_output_path(&path_spec);
        let symbols = match self.compile_save_symbols_from_expr(objects_expr) {
            Ok(symbols) => symbols,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(operator, args, error.message()));
            }
        };
        let symbols = self.filter_persist_symbols_by_contexts(symbols, &context_filters);

        let include_attributes = if overwrite {
            self.dump_save_include_attributes(explicit_options.as_ref())
        } else {
            true
        };
        let rendered = self.render_saved_symbols(&symbols, overwrite, include_attributes);
        if let Err(source) = self.write_saved_content(&path, &rendered, overwrite) {
            self.push_message(
                operator,
                "io",
                format!("could not write `{}`: {source}", path.display()),
            );
            return Ok(system_expr("$Failed"));
        }

        if overwrite {
            Ok(Expr::call(
                system_expr("List"),
                symbols
                    .into_iter()
                    .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
                    .collect(),
            ))
        } else {
            Ok(system_expr("Null"))
        }
    }

    fn compile_save_symbols_from_expr(
        &self,
        expr: &Expr,
    ) -> Result<Vec<Symbol>, PackageToolNormalizationError> {
        let mut collected = Vec::new();
        self.collect_save_symbols(expr, &mut collected)?;
        Ok(collected)
    }

    fn collect_save_symbols(
        &self,
        expr: &Expr,
        collected: &mut Vec<Symbol>,
    ) -> Result<(), PackageToolNormalizationError> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => {
                if !collected.iter().any(|existing| existing == symbol) {
                    collected.push(symbol.clone());
                }
                Ok(())
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
                for item in args {
                    self.collect_save_symbols(item, collected)?;
                }
                Ok(())
            }
            _ => {
                let Some(symbols) = self.resolve_names_matching_expr(expr) else {
                    return Err(PackageToolNormalizationError::InvalidObjects);
                };

                for symbol in symbols {
                    if !collected.iter().any(|existing| existing == &symbol) {
                        collected.push(symbol);
                    }
                }

                Ok(())
            }
        }
    }

    fn resolve_output_path(&self, spec: &str) -> PathBuf {
        let path = PathBuf::from(spec);

        if path.is_absolute() {
            return path;
        }

        self.load_stack
            .last()
            .and_then(|current| current.parent().map(|parent| parent.join(&path)))
            .unwrap_or(path)
    }

    fn render_saved_symbols(
        &self,
        symbols: &[Symbol],
        _textual_dump: bool,
        include_attributes: bool,
    ) -> String {
        let mut blocks = Vec::new();

        for symbol in symbols {
            let symbol_block = self.render_saved_symbol(symbol, include_attributes);
            if !symbol_block.is_empty() {
                blocks.push(symbol_block);
            }
        }

        if blocks.is_empty() {
            String::new()
        } else {
            format!("{}\n", blocks.join("\n\n"))
        }
    }

    fn render_saved_symbol(&self, symbol: &Symbol, include_attributes: bool) -> String {
        let symbol_expr = Expr::Atom(Atom::Symbol(symbol.clone()));
        let mut forms = Vec::new();

        if include_attributes && self.attributes.contains_key(symbol) {
            let attributes = self.attribute_set_for_symbol(symbol);
            if !attributes.is_empty() {
                forms.push(
                    Expr::call(
                        system_expr("SetAttributes"),
                        vec![symbol_expr.clone(), attributes.expr()],
                    )
                    .to_string(),
                );
            }
        }

        for entry in self.definition_catalog_entries_for_symbol(symbol) {
            forms.push(self.save_definition_expr(&entry).to_string());
        }

        if self.options.contains_key(symbol) {
            forms.push(
                Expr::call(
                    system_expr("Set"),
                    vec![
                        Expr::call(system_expr("Options"), vec![symbol_expr.clone()]),
                        self.options_for_symbol(symbol).expr(),
                    ],
                )
                .to_string(),
            );
        }

        if self.syntax_information.contains_key(symbol) {
            forms.push(
                Expr::call(
                    system_expr("Set"),
                    vec![
                        Expr::call(system_expr("SyntaxInformation"), vec![symbol_expr.clone()]),
                        self.syntax_information_for_symbol(symbol),
                    ],
                )
                .to_string(),
            );
        }

        for (key, definition) in self
            .message_definitions
            .iter()
            .filter(|(key, _)| key.symbol == *symbol)
        {
            if let Some(expr) = self.save_message_definition_expr(key, definition) {
                forms.push(expr.to_string());
            }
        }

        forms.join("\n\n")
    }

    fn save_definition_expr(&self, entry: &DefinitionCatalogEntry) -> Expr {
        let head = match (entry.kind, entry.delayed) {
            (DefinitionKind::UpValue, true) => "UpSetDelayed",
            (DefinitionKind::UpValue, false) => "UpSet",
            (_, true) => "SetDelayed",
            (_, false) => "Set",
        };

        Expr::call(
            system_expr(head),
            vec![entry.lhs.clone(), entry.rhs.clone()],
        )
    }

    fn save_message_definition_expr(
        &self,
        key: &MessageNameKey,
        definition: &MessageDefinition,
    ) -> Option<Expr> {
        let (head, rhs) = if let Some(expr) = &definition.immediate {
            ("Set", expr.clone())
        } else if let Some(expr) = &definition.delayed {
            ("SetDelayed", expr.clone())
        } else {
            return None;
        };

        Some(Expr::call(
            system_expr(head),
            vec![message_name_expr(key.symbol.clone(), key.tag.clone()), rhs],
        ))
    }

    fn write_saved_content(
        &self,
        path: &Path,
        content: &str,
        overwrite: bool,
    ) -> Result<(), std::io::Error> {
        if overwrite {
            return fs::write(path, content);
        }

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(content.as_bytes())
    }

    fn once_cache_key(&self, expr: &Expr, location: Option<&Expr>) -> String {
        let location = location
            .map(ToString::to_string)
            .unwrap_or_else(|| "KernelSession".to_string());
        format!("{location}\u{1f}{}", expr)
    }

    fn dump_save_include_attributes(&self, explicit: Option<&SymbolOptions>) -> bool {
        let default = default_system_options(&Symbol::system("DumpSave"));

        explicit
            .and_then(|options| options.get_by_name("SymbolAttributes"))
            .or_else(|| default.get(&Symbol::system("SymbolAttributes")))
            .map(|value| is_system_symbol_name(value, "True"))
            .unwrap_or(true)
    }

    fn compile_persist_context_filters(
        &self,
        explicit: Option<&SymbolOptions>,
    ) -> Result<PersistContextFilters, &'static str> {
        let default = default_system_options(&Symbol::system("Save"));
        let included = explicit
            .and_then(|options| options.get_by_name("IncludedContexts"))
            .or_else(|| default.get(&Symbol::system("IncludedContexts")))
            .and_then(|value| self.parse_persist_context_selection(value))
            .ok_or(
                "expected IncludedContexts to be All, Automatic, None, a context-name string, or a list of context-name strings",
            )?;
        let excluded = explicit
            .and_then(|options| options.get_by_name("ExcludedContexts"))
            .or_else(|| default.get(&Symbol::system("ExcludedContexts")))
            .and_then(|value| self.parse_persist_context_selection(value))
            .ok_or(
                "expected ExcludedContexts to be All, Automatic, None, a context-name string, or a list of context-name strings",
            )?;

        Ok(PersistContextFilters { included, excluded })
    }

    fn parse_persist_context_selection(&self, value: &Expr) -> Option<PersistContextSelection> {
        if is_system_symbol_name(value, "All") {
            return Some(PersistContextSelection::All);
        }
        if is_system_symbol_name(value, "None") {
            return Some(PersistContextSelection::None);
        }
        if is_system_symbol_name(value, "Automatic") {
            return Some(PersistContextSelection::Automatic);
        }
        if let Expr::Atom(Atom::String(context)) = value {
            return Some(PersistContextSelection::Explicit(BTreeSet::from([
                expand_context_spec(context, &self.current_context),
            ])));
        }

        expr_as_string_list(value).map(|contexts| {
            PersistContextSelection::Explicit(
                contexts
                    .into_iter()
                    .map(|context| expand_context_spec(&context, &self.current_context))
                    .collect(),
            )
        })
    }

    fn filter_persist_symbols_by_contexts(
        &self,
        symbols: Vec<Symbol>,
        filters: &PersistContextFilters,
    ) -> Vec<Symbol> {
        symbols
            .into_iter()
            .filter(|symbol| self.persist_context_is_included(symbol.context(), filters))
            .collect()
    }

    fn persist_context_is_included(&self, context: &str, filters: &PersistContextFilters) -> bool {
        self.persist_context_matches_selection(context, &filters.included, true)
            && !self.persist_context_matches_selection(context, &filters.excluded, false)
    }

    fn persist_context_matches_selection(
        &self,
        context: &str,
        selection: &PersistContextSelection,
        include_mode: bool,
    ) -> bool {
        match selection {
            PersistContextSelection::All => true,
            PersistContextSelection::None => false,
            PersistContextSelection::Automatic => {
                if include_mode {
                    self.context_path.iter().any(|entry| entry == context)
                        || self.current_context == context
                } else {
                    context == SYSTEM_CONTEXT
                }
            }
            PersistContextSelection::Explicit(contexts) => contexts.contains(context),
        }
    }
}
