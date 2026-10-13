
// Safe opaque-text comparator, preserving qualified-name byte ordering.
impl SymbolText {
    pub fn cmp_qualified_text(&self, other: &Self) -> std::cmp::Ordering {
        if std::ptr::eq(self, other) { return std::cmp::Ordering::Equal; }
        let lhs_context = if self.context == GLOBAL_CONTEXT || self.context == SYSTEM_CONTEXT { "" } else { self.context };
        let rhs_context = if other.context == GLOBAL_CONTEXT || other.context == SYSTEM_CONTEXT { "" } else { other.context };
        if lhs_context == rhs_context { return self.name.as_bytes().cmp(other.name.as_bytes()); }
        compare_concatenated_bytes([lhs_context.as_bytes(),self.name.as_bytes()], [rhs_context.as_bytes(),other.name.as_bytes()])
    }
}
