use super::*;

impl Session {
    pub(super) fn resolve_message_name_args(
        &self,
        symbol_expr: &Expr,
        tag_expr: &Expr,
    ) -> Option<MessageNameKey> {
        let symbol = match symbol_expr {
            Expr::Atom(Atom::Symbol(symbol)) => symbol.clone(),
            Expr::Atom(Atom::String(raw)) => self.resolve_symbol_name(raw, true)?,
            _ => return None,
        };

        let tag = match tag_expr {
            Expr::Atom(Atom::String(tag)) => tag.clone(),
            Expr::Atom(Atom::Symbol(tag)) => tag.name().to_string(),
            _ => return None,
        };

        Some(MessageNameKey { symbol, tag })
    }

    pub(super) fn message_name_key_from_expr(&self, expr: &Expr) -> Option<MessageNameKey> {
        let Expr::Call { head, args } = expr else {
            return None;
        };

        if !is_system_symbol_name(head, "MessageName") || args.len() != 2 {
            return None;
        }

        self.resolve_message_name_args(&args[0], &args[1])
    }

    pub(super) fn define_message(&mut self, key: MessageNameKey, value: Expr, delayed: bool) {
        let definition = self.message_definitions.entry(key).or_default();

        if delayed {
            definition.immediate = None;
            definition.delayed = Some(value);
        } else {
            definition.immediate = Some(value);
            definition.delayed = None;
        }
    }

    pub(super) fn render_message_text(
        &mut self,
        key: &MessageNameKey,
        args: &[Expr],
    ) -> Result<String, EvalError> {
        let template = self.message_definitions.get(key).cloned();

        let rendered = match template {
            Some(MessageDefinition {
                immediate: Some(expr),
                ..
            }) => expr,
            Some(MessageDefinition {
                delayed: Some(expr),
                ..
            }) => self.evaluate(expr)?,
            _ => {
                if args.is_empty() {
                    return Ok("message emitted".to_string());
                }

                return Ok(args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "));
            }
        };

        let template = match rendered {
            Expr::Atom(Atom::String(text)) => text,
            other => other.to_string(),
        };

        Ok(format_message_template(&template, key, args))
    }

    pub(super) fn lookup_message_symbol(&self, expr: &Expr) -> Option<Symbol> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.clone()),
            Expr::Atom(Atom::String(raw)) => self.resolve_symbol_name(raw, true),
            _ => None,
        }
    }

    pub(super) fn lookup_attribute_target(
        &self,
        expr: &Expr,
        create_if_missing: bool,
    ) -> Option<Symbol> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.clone()),
            Expr::Atom(Atom::String(raw)) => self.resolve_symbol_name(raw, create_if_missing),
            _ => None,
        }
    }

    pub(super) fn lookup_attribute_targets(
        &self,
        expr: &Expr,
        create_if_missing: bool,
    ) -> Option<Vec<Symbol>> {
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "List") => args
                .iter()
                .map(|item| self.lookup_attribute_target(item, create_if_missing))
                .collect(),
            _ => Some(vec![self.lookup_attribute_target(expr, create_if_missing)?]),
        }
    }

    pub(super) fn resolve_message_catalog_query(&self, query: &str) -> Option<MessageCatalogQuery> {
        let query = query.trim();

        if query.is_empty() {
            return Some(MessageCatalogQuery::All);
        }

        if let Some((symbol_name, tag)) = query.rsplit_once("::") {
            let symbol_name = symbol_name.trim();
            let tag = tag.trim();

            if symbol_name.is_empty() || tag.is_empty() {
                return None;
            }

            return self.resolve_symbol_name(symbol_name, true).map(|symbol| {
                MessageCatalogQuery::Key(MessageNameKey {
                    symbol,
                    tag: tag.to_string(),
                })
            });
        }

        self.resolve_symbol_name(query, true)
            .map(MessageCatalogQuery::Symbol)
    }

    pub(super) fn message_names_expr_for_symbol(&self, symbol: Option<&Symbol>) -> Expr {
        let items = self
            .message_definitions
            .keys()
            .filter(|key| symbol.is_none_or(|symbol| key.symbol == *symbol))
            .map(|key| message_name_expr(key.symbol.clone(), key.tag.clone()))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(super) fn build_message_catalog_entry(
        &self,
        key: &MessageNameKey,
        def: &MessageDefinition,
    ) -> MessageCatalogEntry {
        let (template, delayed) = if let Some(rhs) = &def.immediate {
            (rhs.clone(), false)
        } else if let Some(rhs) = &def.delayed {
            (rhs.clone(), true)
        } else {
            (system_expr("Null"), false)
        };

        MessageCatalogEntry {
            message_name: message_name_expr(key.symbol.clone(), key.tag.clone()).to_string(),
            qualified_symbol_name: key.symbol.fully_qualified_name(),
            tag: key.tag.clone(),
            template,
            delayed,
        }
    }

    pub(super) fn message_definition_rule_expr(
        &self,
        key: &MessageNameKey,
        def: &MessageDefinition,
    ) -> Expr {
        let lhs = message_name_expr(key.symbol.clone(), key.tag.clone());

        if let Some(rhs) = &def.immediate {
            Expr::call(system_expr("Rule"), vec![lhs, rhs.clone()])
        } else if let Some(rhs) = &def.delayed {
            Expr::call(system_expr("RuleDelayed"), vec![lhs, rhs.clone()])
        } else {
            Expr::call(system_expr("Rule"), vec![lhs, system_expr("Null")])
        }
    }

    pub(super) fn message_definitions_expr_for_symbol(&self, symbol: Option<&Symbol>) -> Expr {
        let items = self
            .message_definitions
            .iter()
            .filter(|(key, _)| symbol.is_none_or(|symbol| key.symbol == *symbol))
            .map(|(key, def)| self.message_definition_rule_expr(key, def))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(super) fn message_catalog_entries_for_symbol(
        &self,
        symbol: Option<&Symbol>,
    ) -> Vec<MessageCatalogEntry> {
        self.message_definitions
            .iter()
            .filter(|(key, _)| symbol.is_none_or(|symbol| key.symbol == *symbol))
            .map(|(key, def)| self.build_message_catalog_entry(key, def))
            .collect()
    }

    pub(super) fn message_catalog_entries_for_key(
        &self,
        key: &MessageNameKey,
    ) -> Vec<MessageCatalogEntry> {
        self.message_definitions
            .get(key)
            .map(|def| vec![self.build_message_catalog_entry(key, def)])
            .unwrap_or_default()
    }

    pub(super) fn message_tags_expr_for_symbol(&self, symbol: Option<&Symbol>) -> Expr {
        let tags = if symbol.is_some() {
            self.message_definitions
                .keys()
                .filter(|key| symbol.is_none_or(|symbol| key.symbol == *symbol))
                .map(|key| key.tag.clone())
                .collect::<Vec<_>>()
        } else {
            self.message_definitions
                .keys()
                .map(|key| key.tag.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        };

        Expr::call(
            system_expr("List"),
            tags.into_iter().map(Expr::string).collect(),
        )
    }

    pub(super) fn message_definitions_expr_for_key(&self, key: &MessageNameKey) -> Expr {
        let items = self
            .message_definitions
            .get(key)
            .map(|def| vec![self.message_definition_rule_expr(key, def)])
            .unwrap_or_default();

        Expr::call(system_expr("List"), items)
    }

    pub(super) fn message_tags_expr_for_key(&self, key: &MessageNameKey) -> Expr {
        let items = self
            .message_definitions
            .contains_key(key)
            .then(|| vec![Expr::string(key.tag.clone())])
            .unwrap_or_default();

        Expr::call(system_expr("List"), items)
    }
}
