
// Research-only export in the isolated copy; the collector is unchanged.
pub(super) fn research_unique_symbols(symbols: &[rum_core::Symbol], hint: usize) -> Vec<rum_core::Symbol> {
    let mut out = unique_symbols::UniqueSymbolCollector::with_capacity_hint(hint);
    for symbol in symbols { out.push(*symbol); }
    out.into_symbols()
}
