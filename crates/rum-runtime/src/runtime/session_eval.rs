use super::*;

impl Session {
    pub fn inspect(&self, name: &str) -> Option<SymbolInfo> {
        let symbol = self
            .resolve_symbol_name(name, false)
            .or_else(|| self.resolve_symbol_name(name, true))?;
        let builtin = self.builtin_spec_for_symbol(&symbol);
        let definition = self.definitions.get(&symbol);
        let attributes = self.attribute_set_for_symbol(&symbol);
        let message_tags = self
            .message_definitions
            .keys()
            .filter(|key| key.symbol == symbol)
            .map(|key| key.tag.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let message_count = message_tags.len();
        let attribute_names = attributes
            .attributes()
            .into_iter()
            .map(|attribute| attribute.symbol_name().to_string())
            .collect::<Vec<_>>();

        if builtin.is_none()
            && definition.is_none()
            && attributes.is_empty()
            && message_count == 0
            && !is_known_system_symbol(&symbol)
        {
            return None;
        }

        Some(SymbolInfo {
            qualified_name: symbol.fully_qualified_name(),
            context: symbol.context().to_string(),
            display_name: symbol.to_string(),
            summary: builtin.map(|builtin| builtin.summary.to_string()),
            is_builtin: builtin.is_some() || is_known_system_symbol(&symbol),
            attribute_count: attributes.len(),
            attributes: attribute_names,
            has_own_value: definition.and_then(|def| def.own_value.as_ref()).is_some(),
            has_delayed_value: definition
                .and_then(|def| def.delayed_own_value.as_ref())
                .is_some(),
            has_up_values: definition
                .map(|definition| !definition.up_values.is_empty())
                .unwrap_or(false),
            has_down_values: definition
                .map(|definition| !definition.down_values.is_empty())
                .unwrap_or(false),
            has_sub_values: definition
                .map(|definition| !definition.sub_values.is_empty())
                .unwrap_or(false),
            message_count,
            message_tags,
        })
    }

    pub fn inspect_message_catalog(&self, query: Option<&str>) -> Option<Vec<MessageCatalogEntry>> {
        let query = query.map(str::trim).filter(|query| !query.is_empty());

        match query {
            None => Some(self.message_catalog_entries_for_symbol(None)),
            Some(query) => match self.resolve_message_catalog_query(query)? {
                MessageCatalogQuery::All => Some(self.message_catalog_entries_for_symbol(None)),
                MessageCatalogQuery::Symbol(symbol) => {
                    Some(self.message_catalog_entries_for_symbol(Some(&symbol)))
                }
                MessageCatalogQuery::Key(key) => Some(self.message_catalog_entries_for_key(&key)),
            },
        }
    }

    pub fn inspect_definition_catalog(&self, name: &str) -> Option<Vec<DefinitionCatalogEntry>> {
        let symbol = self
            .resolve_symbol_name(name, false)
            .or_else(|| self.resolve_symbol_name(name, true))?;
        let message_count = self
            .message_definitions
            .keys()
            .filter(|key| key.symbol == symbol)
            .count();

        if self.builtin_spec_for_symbol(&symbol).is_none()
            && !self.definitions.contains_key(&symbol)
            && message_count == 0
            && !is_known_system_symbol(&symbol)
        {
            return None;
        }

        Some(self.definition_catalog_entries_for_symbol(&symbol))
    }

    pub fn inspect_rewrite_rules(
        &mut self,
        expr: Expr,
    ) -> Result<Option<Vec<RewriteRuleInfo>>, EvalError> {
        let resolved = self.evaluate(expr)?;

        Ok(normalize_rewrite_rules(&resolved).map(|rules| {
            rules
                .into_iter()
                .map(|rule| RewriteRuleInfo {
                    lhs: rule.lhs,
                    rhs: rule.rhs,
                    delayed: rule.delayed,
                })
                .collect()
        }))
    }

    pub fn eval_text(&mut self, input: &str) -> Result<EvalOutcome, EvalError> {
        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let result = (|| {
            let syntax = parse_expr(input)?;
            let expr = self.lower_syntax_expr(&syntax)?;
            self.evaluate(expr)
        })();

        match result {
            Ok(value) => {
                let messages = self.messages.split_off(message_checkpoint);
                self.last_message_list = messages.clone();
                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);
                Ok(EvalOutcome { value, messages })
            }
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);
                self.last_message_list = self.messages[message_checkpoint..].to_vec();
                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);
                Err(err)
            }
        }
    }

    pub fn eval_script(&mut self, source: &str) -> Result<EvalOutcome, EvalError> {
        let message_checkpoint = self.messages.len();
        let forms = split_script_into_forms(source)?;
        let mut value = system_expr("Null");
        let mut messages = Vec::new();

        for form in forms {
            match self.eval_text(&form) {
                Ok(outcome) => {
                    value = outcome.value;
                    messages.extend(outcome.messages);
                }
                Err(err) => {
                    if !messages.is_empty() {
                        self.messages
                            .splice(message_checkpoint..message_checkpoint, messages);
                        self.last_message_list = self.messages[message_checkpoint..].to_vec();
                    }

                    return Err(err);
                }
            }
        }

        self.last_message_list = messages.clone();
        Ok(EvalOutcome { value, messages })
    }

    pub fn eval_file(&mut self, path: impl AsRef<Path>) -> Result<EvalOutcome, EvalError> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path).map_err(|source| EvalError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let snapshot = self.capture_loader_snapshot();

        self.load_stack.push(path);
        let result = self.eval_script(&source);
        self.load_stack.pop();

        if result.is_err() {
            self.restore_loader_snapshot(snapshot);
        }

        result
    }
}
