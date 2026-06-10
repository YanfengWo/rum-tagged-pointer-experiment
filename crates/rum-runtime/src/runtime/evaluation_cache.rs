use super::*;

impl Session {
    pub(super) fn bump_evaluation_epoch(&mut self) {
        self.evaluation_epoch = self.evaluation_epoch.saturating_add(1);
        self.clear_evaluation_caches();
    }

    pub(super) fn clear_evaluation_caches(&mut self) {
        self.stable_symbol_cache.clear();
        self.symbol_call_effect_cache.clear();
        self.call_stability_cache.clear();
    }

    pub(super) fn cached_expr_is_evaluation_stable(&mut self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { head, args } => self.call_is_evaluation_stable(head, args),
            Expr::Atom(_) => self.expr_is_evaluation_stable_uncached(expr),
        }
    }

    pub(super) fn call_is_evaluation_stable(
        &mut self,
        head: &SharedExpr,
        args: &SharedArgs,
    ) -> bool {
        let key = CallStabilityCacheKey::new(head, args);
        if let Some(entry) = self.call_stability_cache.get(&key) {
            if entry.epoch == self.evaluation_epoch
                && SharedExpr::ptr_eq(&entry.head, head)
                && SharedArgs::ptr_eq(&entry.args, args)
            {
                return entry.stable;
            }
        }

        let stable = self.call_is_evaluation_stable_uncached(head, args);
        self.call_stability_cache.insert(
            key,
            CallStabilityCacheEntry {
                epoch: self.evaluation_epoch,
                head: head.clone(),
                args: args.clone(),
                stable,
            },
        );
        stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluation_cache_is_invalidated_by_new_down_value() {
        let mut session = Session::default();

        assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "f[1]");
        assert!(!session.symbol_call_effect_cache.is_empty());

        session.eval_text("f[x_] := x + 1").unwrap();

        assert_eq!(session.eval_text("f[1]").unwrap().value.to_string(), "2");
    }

    #[test]
    fn evaluation_cache_is_invalidated_by_attribute_change() {
        let mut session = Session::default();

        assert_eq!(
            session.eval_text("f[{a, b}]").unwrap().value.to_string(),
            "f[List[a, b]]"
        );
        assert!(!session.symbol_call_effect_cache.is_empty());

        session.eval_text("SetAttributes[f, Listable]").unwrap();

        assert_eq!(
            session.eval_text("f[{a, b}]").unwrap().value.to_string(),
            "List[f[a], f[b]]"
        );
    }
}
