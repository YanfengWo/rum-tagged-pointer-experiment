use super::super::*;

const UNIQUE_SYMBOL_INDEX_THRESHOLD: usize = 16;

pub(super) struct UniqueSymbolCollector {
    symbols: Vec<Symbol>,
    index: Option<std::collections::HashSet<Symbol>>,
    capacity_hint: usize,
}

impl UniqueSymbolCollector {
    pub(super) fn with_capacity_hint(capacity_hint: usize) -> Self {
        Self {
            symbols: Vec::new(),
            index: None,
            capacity_hint,
        }
    }

    pub(super) fn push(&mut self, symbol: Symbol) {
        if let Some(index) = &mut self.index {
            if index.insert(symbol) {
                self.symbols.push(symbol);
            }
            return;
        }

        if self.symbols.iter().any(|existing| *existing == symbol) {
            return;
        }

        if self.symbols.len() >= UNIQUE_SYMBOL_INDEX_THRESHOLD {
            let capacity = self.capacity_hint.max(self.symbols.len() + 1);
            let mut index = std::collections::HashSet::with_capacity(capacity);
            index.extend(self.symbols.iter().copied());
            let inserted = index.insert(symbol);
            debug_assert!(inserted, "linear duplicate check rejected this symbol");
            self.index = Some(index);
        }
        self.symbols.push(symbol);
    }

    pub(super) fn into_symbols(self) -> Vec<Symbol> {
        self.symbols
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_first_seen_order_across_the_adaptive_index_threshold() {
        let symbols = (0..UNIQUE_SYMBOL_INDEX_THRESHOLD + 4)
            .map(|index| Symbol::from_name(&format!("collector{index}")))
            .collect::<Vec<_>>();
        let mut collector = UniqueSymbolCollector::with_capacity_hint(symbols.len());

        for symbol in &symbols {
            collector.push(*symbol);
        }
        collector.push(symbols[3]);
        collector.push(symbols[UNIQUE_SYMBOL_INDEX_THRESHOLD]);

        assert!(collector.index.is_some());
        assert_eq!(collector.into_symbols(), symbols);
    }

    #[test]
    fn keeps_small_symbol_sets_on_the_linear_path() {
        let mut collector = UniqueSymbolCollector::with_capacity_hint(4);
        for name in ["a", "b", "a", "c"] {
            collector.push(Symbol::from_name(name));
        }

        assert!(collector.index.is_none());
        assert_eq!(collector.symbols.len(), 3);
    }
}
