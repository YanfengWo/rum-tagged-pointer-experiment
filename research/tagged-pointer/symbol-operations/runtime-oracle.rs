
// Research-only wrapper, not called by the default evaluator.
#[doc(hidden)]
pub fn research_unique_symbols(symbols: &[Symbol], hint: usize) -> Vec<Symbol> {
    definitions::research_unique_symbols(symbols, hint)
}
