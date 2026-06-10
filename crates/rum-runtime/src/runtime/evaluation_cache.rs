use super::*;

impl Session {
    pub(super) fn bump_evaluation_epoch(&mut self) {
        self.evaluation_epoch = self.evaluation_epoch.saturating_add(1);
        self.clear_evaluation_caches();
    }

    pub(super) fn clear_evaluation_caches(&mut self) {
        self.stable_symbol_cache.clear();
        self.symbol_call_effect_cache.clear();
    }

    pub(super) fn cached_expr_is_evaluation_stable(&mut self, expr: &Expr) -> bool {
        self.expr_is_evaluation_stable_uncached(expr)
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
