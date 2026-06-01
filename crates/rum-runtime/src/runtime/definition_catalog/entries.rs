use super::*;

impl Session {
    pub(in crate::runtime) fn build_definition_catalog_entries(
        &self,
        symbol: &Symbol,
        definition: &SymbolDefinition,
    ) -> Vec<DefinitionCatalogEntry> {
        let mut entries = Vec::new();

        if let Some(rhs) = &definition.own_value {
            entries.push(DefinitionCatalogEntry {
                kind: DefinitionKind::OwnValue,
                lhs: Expr::Atom(Atom::Symbol(symbol.clone())),
                rhs: rhs.clone(),
                delayed: false,
            });
        }

        if let Some(rhs) = &definition.delayed_own_value {
            entries.push(DefinitionCatalogEntry {
                kind: DefinitionKind::DelayedOwnValue,
                lhs: Expr::Atom(Atom::Symbol(symbol.clone())),
                rhs: rhs.clone(),
                delayed: true,
            });
        }

        entries.extend(
            definition
                .up_values
                .iter()
                .map(|up_value| DefinitionCatalogEntry {
                    kind: DefinitionKind::UpValue,
                    lhs: up_value.lhs.clone(),
                    rhs: up_value.rhs.clone(),
                    delayed: up_value.delayed,
                }),
        );

        entries.extend(
            definition
                .down_values
                .iter()
                .map(|down_value| DefinitionCatalogEntry {
                    kind: DefinitionKind::DownValue,
                    lhs: down_value.lhs.clone(),
                    rhs: down_value.rhs.clone(),
                    delayed: down_value.delayed,
                }),
        );

        entries.extend(
            definition
                .sub_values
                .iter()
                .map(|sub_value| DefinitionCatalogEntry {
                    kind: DefinitionKind::SubValue,
                    lhs: sub_value.lhs.clone(),
                    rhs: sub_value.rhs.clone(),
                    delayed: sub_value.delayed,
                }),
        );

        entries
    }

    pub(in crate::runtime) fn definition_catalog_entries_for_symbol(
        &self,
        symbol: &Symbol,
    ) -> Vec<DefinitionCatalogEntry> {
        self.definitions
            .get(symbol)
            .map(|definition| self.build_definition_catalog_entries(symbol, definition))
            .unwrap_or_default()
    }

    pub(in crate::runtime) fn definition_rule_expr(entry: &DefinitionCatalogEntry) -> Expr {
        Expr::call(
            system_expr("RuleDelayed"),
            vec![
                Expr::call(system_expr("HoldPattern"), vec![entry.lhs.clone()]),
                entry.rhs.clone(),
            ],
        )
    }

    pub(in crate::runtime) fn own_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let items = self
            .definition_catalog_entries_for_symbol(symbol)
            .into_iter()
            .filter(|entry| {
                matches!(
                    entry.kind,
                    DefinitionKind::OwnValue | DefinitionKind::DelayedOwnValue
                )
            })
            .map(|entry| Self::definition_rule_expr(&entry))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(in crate::runtime) fn down_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let items = self
            .definition_catalog_entries_for_symbol(symbol)
            .into_iter()
            .filter(|entry| entry.kind == DefinitionKind::DownValue)
            .map(|entry| Self::definition_rule_expr(&entry))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(in crate::runtime) fn up_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let items = self
            .definition_catalog_entries_for_symbol(symbol)
            .into_iter()
            .filter(|entry| entry.kind == DefinitionKind::UpValue)
            .map(|entry| Self::definition_rule_expr(&entry))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    pub(in crate::runtime) fn sub_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let items = self
            .definition_catalog_entries_for_symbol(symbol)
            .into_iter()
            .filter(|entry| entry.kind == DefinitionKind::SubValue)
            .map(|entry| Self::definition_rule_expr(&entry))
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }
}
