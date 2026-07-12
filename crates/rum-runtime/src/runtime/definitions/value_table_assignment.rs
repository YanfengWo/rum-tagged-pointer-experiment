use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum ValueTableKind {
    OwnValues,
    DownValues,
    UpValues,
    SubValues,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AssignedValueTableRule {
    lhs: Expr,
    rhs: Expr,
    delayed: bool,
}

impl ValueTableKind {
    fn inspector_name(self) -> &'static str {
        match self {
            Self::OwnValues => "OwnValues",
            Self::DownValues => "DownValues",
            Self::UpValues => "UpValues",
            Self::SubValues => "SubValues",
        }
    }

    fn definition_target(self, symbol: Symbol) -> TaggedDefinitionTarget {
        match self {
            Self::OwnValues => TaggedDefinitionTarget::OwnValue(symbol),
            Self::DownValues => TaggedDefinitionTarget::DownValue(symbol),
            Self::UpValues => TaggedDefinitionTarget::UpValue(symbol),
            Self::SubValues => TaggedDefinitionTarget::SubValue(symbol),
        }
    }
}

impl Session {
    pub(in crate::runtime) fn normalize_value_table_target_expr(
        &self,
        expr: &Expr,
    ) -> Option<(ValueTableKind, Symbol)> {
        let Expr::Call { head, args } = expr else {
            return None;
        };
        if args.len() != 1 {
            return None;
        }

        let inspector = expr_as_symbol(head)?;
        if inspector.context() != SYSTEM_CONTEXT {
            return None;
        }

        let kind = match inspector.name() {
            "OwnValues" => ValueTableKind::OwnValues,
            "DownValues" => ValueTableKind::DownValues,
            "UpValues" => ValueTableKind::UpValues,
            "SubValues" => ValueTableKind::SubValues,
            _ => return None,
        };
        let target = *expr_as_symbol(&args[0])?;
        Some((kind, target))
    }

    /// Replaces one value table after validating the complete serialized table.
    /// Invalid assignments use the dedicated `<Inspector>::setval` diagnostic.
    pub(in crate::runtime) fn replace_value_table_from_expr(
        &mut self,
        kind: ValueTableKind,
        target: Symbol,
        rhs: &Expr,
    ) -> bool {
        let rules = match parse_assigned_value_table(kind, target, rhs) {
            Ok(rules) => rules,
            Err(message) => {
                self.push_message(kind.inspector_name(), "setval", message);
                return false;
            }
        };

        // Parsing and target-kind validation above are deliberately complete before
        // this first mutation, so a bad later rule can never leave a partial table.
        self.clear_value_table(kind, &target);
        for rule in rules {
            self.store_definition_target(
                kind.definition_target(target),
                rule.lhs,
                rule.rhs,
                rule.delayed,
            );
        }
        true
    }

    fn clear_value_table(&mut self, kind: ValueTableKind, target: &Symbol) {
        let Some(definition) = self.definitions.get_mut(target) else {
            return;
        };

        let (changed, removed_up_values) = match kind {
            ValueTableKind::OwnValues => {
                let removed_immediate = definition.own_value.take().is_some();
                let removed_delayed = definition.delayed_own_value.take().is_some();
                (removed_immediate || removed_delayed, false)
            }
            ValueTableKind::DownValues => {
                let changed = !definition.down_values.is_empty();
                definition.down_values.clear();
                definition.refresh_down_value_shallow_signatures(target);
                (changed, false)
            }
            ValueTableKind::UpValues => {
                let changed = !definition.up_values.is_empty();
                definition.up_values.clear();
                (changed, changed)
            }
            ValueTableKind::SubValues => {
                let changed = !definition.sub_values.is_empty();
                definition.sub_values.clear();
                (changed, false)
            }
        };
        let definition_is_empty = definition.is_empty();

        if definition_is_empty {
            self.definitions.remove(target);
        }
        if removed_up_values {
            self.refresh_has_up_value_definitions();
        }
        if changed {
            if kind == ValueTableKind::OwnValues {
                self.installed_system_own_values.remove(target);
            }
            self.bump_evaluation_epoch_for_symbol(target);
        }
    }
}

fn parse_assigned_value_table(
    kind: ValueTableKind,
    target: Symbol,
    rhs: &Expr,
) -> Result<Vec<AssignedValueTableRule>, String> {
    let Expr::Call { head, args } = rhs else {
        return Err(format!(
            "{}[{}] must be assigned a List of Rule or RuleDelayed entries; received {}.",
            kind.inspector_name(),
            target,
            rhs
        ));
    };
    if !is_system_symbol_name(head, "List") {
        return Err(format!(
            "{}[{}] must be assigned a List of Rule or RuleDelayed entries; received {}.",
            kind.inspector_name(),
            target,
            rhs
        ));
    }

    let expected_target = kind.definition_target(target);
    let mut rules = Vec::with_capacity(args.len());
    for rule_expr in args {
        let Expr::Call {
            head: rule_head,
            args: rule_args,
        } = rule_expr
        else {
            return Err(malformed_value_table_rule_message(kind, target, rule_expr));
        };
        if rule_args.len() != 2 {
            return Err(malformed_value_table_rule_message(kind, target, rule_expr));
        }

        let delayed = if is_system_symbol_name(rule_head, "RuleDelayed") {
            true
        } else if is_system_symbol_name(rule_head, "Rule") {
            false
        } else {
            return Err(malformed_value_table_rule_message(kind, target, rule_expr));
        };

        let lhs = strip_hold_pattern(&rule_args[0]);
        if classify_tagged_definition(&target, lhs) != Ok(expected_target.clone()) {
            return Err(format!(
                "Cannot assign rule {} to {}[{}]: its left-hand side {} does not define the target's {} table.",
                rule_expr,
                kind.inspector_name(),
                target,
                lhs,
                kind.inspector_name()
            ));
        }

        // Inspectors serialize every entry as RuleDelayed, even an immediate own
        // value. Its rhs is already evaluated, so the round-trip safely restores it
        // as a delayed own value.
        rules.push(AssignedValueTableRule {
            lhs: lhs.clone(),
            rhs: rule_args[1].clone(),
            delayed,
        });
    }

    Ok(rules)
}

fn malformed_value_table_rule_message(kind: ValueTableKind, target: Symbol, rule: &Expr) -> String {
    format!(
        "Cannot assign entry {} to {}[{}]: expected Rule[lhs, rhs] or RuleDelayed[lhs, rhs].",
        rule,
        kind.inspector_name(),
        target
    )
}
