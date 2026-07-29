use super::super::*;

#[cfg(test)]
use std::cell::RefCell;

impl Session {
    pub(in crate::runtime) fn try_incremental_append_to_plain_list(
        &mut self,
        target_expr: &Expr,
        item: &Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let Some(target) = expr_as_symbol(target_expr).copied() else {
            return Ok(None);
        };
        let Some(cache_key) = self.incremental_append_plain_list_cache_key(&target) else {
            return Ok(None);
        };

        if !self.incremental_append_environment_is_unobservable(&target)
            || !self.list_has_default_runtime_semantics()
            || self.symbol_is_protected(&target)
            || is_system_symbol_name(item, "Nothing")
            || expr_contains_symbol(item, &target)
        {
            return Ok(None);
        }

        let (item_is_stable, item_bloom) = self.cached_expr_stability_with_bloom(item);
        if !item_is_stable {
            return Ok(None);
        }

        if self.interrupt_armed {
            self.poll_host_interrupt_if_due()?;
        }
        if self.symbol_is_protected(&target) {
            return Ok(None);
        }

        let Some(mut proof) = self.take_call_stability_proof_for_key(cache_key) else {
            return Ok(None);
        };

        let (updated_head, updated_args, updated) = {
            let Some(definition) = self.definitions.get_mut(&target) else {
                return Ok(None);
            };
            let Some(mut value_guard) = DefinitionValueRollback::take(&mut definition.own_value)
            else {
                return Ok(None);
            };
            let Expr::Call { head, args } = value_guard.value_mut() else {
                return Ok(None);
            };
            if !is_system_symbol_name(head, heads::structure::LIST) {
                return Ok(None);
            }

            {
                let list_items = args.make_mut();
                if list_items.try_reserve(1).is_err() {
                    return Ok(None);
                }
                let append_guard = AppendedItemRollback::push(list_items, item.clone());
                if take_incremental_append_failure(&target) {
                    return Err(EvalError::Cancelled);
                }
                append_guard.commit();
            }

            let updated_head = head.clone();
            let updated_args = args.clone();
            let updated = Expr::Call {
                head: updated_head.clone(),
                args: updated_args.clone(),
            };
            value_guard.commit();
            (updated_head, updated_args, updated)
        };

        self.bump_evaluation_epoch_for_symbol(&target);
        proof.bloom.union_assign(item_bloom);
        self.reinsert_call_stability_proof(&updated_head, &updated_args, proof);

        Ok(Some(updated))
    }

    fn incremental_append_plain_list_cache_key(
        &self,
        target: &Symbol,
    ) -> Option<CallStabilityCacheKey> {
        if self.incremental_append_has_dynamic_own_value(target)
            || self.removed_symbols.contains(target)
        {
            return None;
        }

        let current = self
            .definitions
            .get(target)
            .and_then(|definition| definition.own_value.as_ref())?;
        let Expr::Call { head, args } = current else {
            return None;
        };
        is_system_symbol_name(head, heads::structure::LIST)
            .then(|| CallStabilityCacheKey::new(head, args))
    }

    fn incremental_append_environment_is_unobservable(&self, target: &Symbol) -> bool {
        !self.has_up_value_definitions
            && !self.tracing_enabled
            && self.monitor_scopes.is_empty()
            && self.current_time_deadline.is_none()
            && self.current_memory_limit.is_none()
            && self.eval_deadline.is_none()
            && self.pending_input_request.is_none()
            && self.pending_eval_request.is_none()
            && self.pending_suspension.is_none()
            && !self.incremental_append_has_dynamic_own_value(target)
    }

    fn list_has_default_runtime_semantics(&self) -> bool {
        let list = Symbol::system(heads::structure::LIST);
        self.attribute_set_for_symbol(&list) == default_system_attribute_set(&list)
            && self
                .definitions
                .get(&list)
                .is_none_or(SymbolDefinition::is_empty)
            && !self.incremental_append_has_dynamic_own_value(&list)
            && !self.removed_symbols.contains(&list)
            && !self.builtins.contains_key(&list)
    }

    fn incremental_append_has_dynamic_own_value(&self, symbol: &Symbol) -> bool {
        self.dynamic_own_values
            .iter()
            .any(|frame| frame.symbol == *symbol)
    }

    #[cfg(test)]
    pub(in crate::runtime) fn fail_next_incremental_append_after_value_take(
        &mut self,
        target: Symbol,
    ) {
        INCREMENTAL_APPEND_FAILURE_TARGET.with(|pending| {
            *pending.borrow_mut() = Some(target);
        });
    }
}

struct DefinitionValueRollback<'a> {
    slot: &'a mut Option<Expr>,
    value: Option<Expr>,
}

impl<'a> DefinitionValueRollback<'a> {
    fn take(slot: &'a mut Option<Expr>) -> Option<Self> {
        let value = slot.take()?;
        Some(Self {
            slot,
            value: Some(value),
        })
    }

    fn value_mut(&mut self) -> &mut Expr {
        self.value
            .as_mut()
            .expect("an active definition rollback guard owns its value")
    }

    fn commit(mut self) {
        *self.slot = self.value.take();
    }
}

impl Drop for DefinitionValueRollback<'_> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            *self.slot = Some(value);
        }
    }
}

struct AppendedItemRollback<'a> {
    args: &'a mut Vec<Expr>,
    committed: bool,
}

impl<'a> AppendedItemRollback<'a> {
    fn push(args: &'a mut Vec<Expr>, item: Expr) -> Self {
        args.push(item);
        Self {
            args,
            committed: false,
        }
    }

    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for AppendedItemRollback<'_> {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.args.pop();
        }
    }
}

#[cfg(test)]
thread_local! {
    static INCREMENTAL_APPEND_FAILURE_TARGET: RefCell<Option<Symbol>> = const {
        RefCell::new(None)
    };
}

#[cfg(test)]
fn take_incremental_append_failure(target: &Symbol) -> bool {
    INCREMENTAL_APPEND_FAILURE_TARGET.with(|pending| {
        let mut pending = pending.borrow_mut();
        if pending
            .as_ref()
            .is_some_and(|candidate| candidate == target)
        {
            pending.take();
            true
        } else {
            false
        }
    })
}

#[cfg(not(test))]
#[inline]
fn take_incremental_append_failure(_target: &Symbol) -> bool {
    false
}
