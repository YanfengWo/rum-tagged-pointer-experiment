use super::*;

impl Drop for LexicalProfileContextGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        LEXICAL_CONTEXT_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        LEXICAL_CONTEXT_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

impl Drop for LexicalProfileArgSiteGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        ARG_PREFILTER_SITE_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        ARG_PREFILTER_SITE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

impl Drop for ExpressionRebuildSourceGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
        EXPRESSION_REBUILD_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

#[inline]
pub(in crate::runtime) fn lexical_profile_context(
    context: &'static str,
) -> LexicalProfileContextGuard {
    if !lexical_profile_shapes_enabled() && !lexical_profile_sources_enabled() {
        return LexicalProfileContextGuard { active: false };
    }

    LEXICAL_CONTEXT_STACK.with(|stack| {
        stack.borrow_mut().push(context);
    });
    LEXICAL_CONTEXT_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    LexicalProfileContextGuard { active: true }
}

#[inline]
pub(in crate::runtime) fn lexical_profile_arg_prefilter_site(
    parent_head: &Expr,
    index: usize,
) -> LexicalProfileArgSiteGuard {
    if !lexical_profile_shapes_enabled() {
        return LexicalProfileArgSiteGuard { active: false };
    }

    let site = PresenceArgSite {
        parent: presence_root_shape(parent_head),
        index: index.min(usize::from(u16::MAX)) as u16,
    };
    ARG_PREFILTER_SITE_STACK.with(|stack| {
        stack.borrow_mut().push(site);
    });
    ARG_PREFILTER_SITE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    LexicalProfileArgSiteGuard { active: true }
}

#[inline]
pub(in crate::runtime) fn expression_rebuild_source_symbol(
    symbol: Symbol,
) -> ExpressionRebuildSourceGuard {
    if !lexical_profile_shapes_enabled()
        && !lexical_profile_provenance_enabled()
        && !rule_rhs_timing_enabled()
    {
        return ExpressionRebuildSourceGuard { active: false };
    }

    EXPRESSION_REBUILD_SOURCE_STACK.with(|stack| {
        stack.borrow_mut().push(symbol);
    });
    EXPRESSION_REBUILD_SOURCE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
    ExpressionRebuildSourceGuard { active: true }
}
