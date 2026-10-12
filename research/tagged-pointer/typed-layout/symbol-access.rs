// Additive safe API proposal, appended only to an isolated copy of expr.rs.
// No field layout casts, raw pointer operations, new lookup or allocation.
impl Symbol {
    /// The existing permanently interned context/name handle.
    #[inline]
    pub fn interned_text(&self) -> &'static SymbolText { self.text }

    /// Rebuild from an authentic opaque text handle. Derive id from the handle
    /// instead of accepting an external numeric id; identity remains non-zero.
    #[inline]
    pub fn from_interned_text(text: &'static SymbolText, identity: Option<NonZeroU64>) -> Self {
        Self { id: text.id, identity, text }
    }
}
impl SymbolText {
    /// Observe the immutable id; this does not construct a Symbol or acquire ownership.
    #[inline]
    pub fn interned_id(&self) -> u32 { self.id }
}
#[cfg(test)]
mod tagged_symbol_access_tests {
    use super::*;
    fn roundtrip(s:Symbol) {
        let n=Symbol::from_interned_text(s.interned_text(),s.identity().and_then(NonZeroU64::new));
        assert_eq!(n,s); assert_eq!(n.interned_id(),s.interned_id());
        assert_eq!(n.cmp(&s),Ordering::Equal);
    }
    #[test] fn plain_and_context_handles_roundtrip() {
        roundtrip(Symbol::new("Global`","x")); roundtrip(Symbol::new("Other`","x"));
    }
    #[test] fn generated_full_identity_roundtrips() {
        roundtrip(Symbol::with_identity("Global`","x",1));
        roundtrip(Symbol::with_identity("Global`","x",u64::MAX));
    }
    #[test] fn rebuilding_does_not_merge_generated_and_plain() {
        let s=Symbol::new("Global`","x");
        let g=Symbol::from_interned_text(s.interned_text(),NonZeroU64::new(1));
        assert_ne!(s,g); assert_eq!(s.interned_id(),g.interned_id());
    }
    #[test] fn text_id_observation_agrees_with_every_symbol_kind() {
        for s in [Symbol::new("Global`","x"),Symbol::new("Other`","x"),Symbol::with_identity("Global`","x",u64::MAX)] {
            assert_eq!(s.interned_text().interned_id(),s.interned_id());
        }
    }
}
