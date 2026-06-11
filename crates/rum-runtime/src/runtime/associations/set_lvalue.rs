use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum AssociationSetLValueTarget {
    FunctionCall {
        symbol: Symbol,
        selectors: Vec<Expr>,
    },
    Part {
        symbol: Symbol,
        selectors: Vec<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AssociationSetUpdate {
    Updated(Expr),
    MissingKey(Expr),
    Unchanged,
}

impl AssociationSetLValueTarget {
    fn symbol(&self) -> &Symbol {
        match self {
            Self::FunctionCall { symbol, .. } | Self::Part { symbol, .. } => symbol,
        }
    }

    fn selectors(&self) -> &[Expr] {
        match self {
            Self::FunctionCall { selectors, .. } | Self::Part { selectors, .. } => selectors,
        }
    }
}

impl Session {
    pub(in crate::runtime) fn try_set_association_lvalue(
        &mut self,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(target) = self.normalize_association_set_lvalue_target(&lhs) else {
            return Ok(None);
        };

        if self.association_lvalue_current(target.symbol()).is_none() {
            return Ok(None);
        }

        if !self.ensure_symbol_not_protected_for_write("Set", target.symbol()) {
            return Ok(Some(rhs));
        }

        let value =
            self.resume_association_set_lvalue_selector_evaluation(target, rhs, Vec::new(), 0)?;
        Ok(Some(value))
    }

    fn normalize_association_set_lvalue_target(
        &self,
        lhs: &Expr,
    ) -> Option<AssociationSetLValueTarget> {
        let Expr::Call { head, args } = lhs else {
            return None;
        };

        if is_system_symbol_name(head, "Part") && args.len() >= 2 {
            let symbol = self.normalize_symbol_target_expr(&args[0], true)?;
            return Some(AssociationSetLValueTarget::Part {
                symbol,
                selectors: args[1..].to_vec(),
            });
        }

        let symbol = expr_as_symbol(head)?.clone();
        (!args.is_empty()).then_some(AssociationSetLValueTarget::FunctionCall {
            symbol,
            selectors: args.to_vec(),
        })
    }

    fn association_lvalue_current(&self, symbol: &Symbol) -> Option<Expr> {
        let current = self
            .definitions
            .get(symbol)
            .and_then(|definition| definition.own_value.clone())?;
        association_entry_forms_from_expr(&current)?;
        Some(current)
    }

    fn resume_association_set_lvalue_selector_evaluation(
        &mut self,
        target: AssociationSetLValueTarget,
        rhs: Expr,
        mut evaluated_selectors: Vec<Expr>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= target.selectors().len() {
            return self.finish_association_set_lvalue(target, evaluated_selectors, rhs);
        }

        let selector = target.selectors()[index].clone();
        let selector_result = self.evaluate(selector);
        let selector = self.suspend_with_frame(selector_result, {
            let target = target.clone();
            let rhs = rhs.clone();
            let evaluated_selectors = evaluated_selectors.clone();
            move |session, selector| {
                let mut evaluated_selectors = evaluated_selectors;
                evaluated_selectors.push(selector);
                session.resume_association_set_lvalue_selector_evaluation(
                    target,
                    rhs,
                    evaluated_selectors,
                    index + 1,
                )
            }
        })?;

        evaluated_selectors.push(selector);
        self.resume_association_set_lvalue_selector_evaluation(
            target,
            rhs,
            evaluated_selectors,
            index + 1,
        )
    }

    fn finish_association_set_lvalue(
        &mut self,
        target: AssociationSetLValueTarget,
        evaluated_selectors: Vec<Expr>,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(current) = self.association_lvalue_current(target.symbol()) else {
            return Ok(rhs);
        };

        let update = match target {
            AssociationSetLValueTarget::FunctionCall { .. } => {
                set_association_key_path(current, &evaluated_selectors, rhs.clone())
            }
            AssociationSetLValueTarget::Part { .. } => {
                set_association_part_path(current, &evaluated_selectors, rhs.clone())
            }
        };

        match update {
            AssociationSetUpdate::Updated(updated) => {
                let target_symbol = *target.symbol();
                let definition = self.definitions.entry(target_symbol).or_default();
                definition.own_value = Some(updated);
                definition.delayed_own_value = None;
                self.bump_evaluation_epoch_for_symbol(&target_symbol);
            }
            AssociationSetUpdate::MissingKey(key) => {
                self.push_message(
                    "Set",
                    "kval",
                    format!("The value for the key {key} does not exist."),
                );
            }
            AssociationSetUpdate::Unchanged => {}
        }

        Ok(rhs)
    }
}

fn set_association_part_path(current: Expr, selectors: &[Expr], rhs: Expr) -> AssociationSetUpdate {
    if let Some(keys) = association_part_key_path(selectors) {
        return set_association_key_path(current, &keys, rhs);
    }

    AssociationSetUpdate::Unchanged
}

fn association_part_key_path(selectors: &[Expr]) -> Option<Vec<Expr>> {
    selectors
        .iter()
        .map(association_part_key_selector)
        .collect()
}

fn association_part_key_selector(selector: &Expr) -> Option<Expr> {
    if matches!(selector, Expr::Atom(Atom::String(_))) {
        return Some(selector.clone());
    }

    match selector {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn set_association_key_path(current: Expr, selectors: &[Expr], rhs: Expr) -> AssociationSetUpdate {
    let Some((key, rest)) = selectors.split_first() else {
        return AssociationSetUpdate::Unchanged;
    };

    let Some(mut entries) = association_entry_forms_from_expr(&current) else {
        return AssociationSetUpdate::Unchanged;
    };

    if rest.is_empty() {
        upsert_immediate_association_entry(&mut entries, key.clone(), rhs);
        return AssociationSetUpdate::Updated(render_association_entry_forms_preserving_heads(
            entries,
        ));
    }

    let Some(index) = entries.iter().position(|entry| entry.key == *key) else {
        return AssociationSetUpdate::MissingKey(key.clone());
    };

    match set_association_key_path(entries[index].value.clone(), rest, rhs) {
        AssociationSetUpdate::Updated(updated_child) => {
            entries[index].value = updated_child;
            entries[index].delayed = false;
            AssociationSetUpdate::Updated(render_association_entry_forms_preserving_heads(entries))
        }
        other => other,
    }
}

fn upsert_immediate_association_entry(
    entries: &mut Vec<AssociationEntryForm>,
    key: Expr,
    rhs: Expr,
) {
    let updated = AssociationEntryForm {
        key: key.clone(),
        value: rhs,
        delayed: false,
    };

    if let Some(index) = entries.iter().position(|entry| entry.key == key) {
        entries[index] = updated;
    } else {
        entries.push(updated);
    }
}
