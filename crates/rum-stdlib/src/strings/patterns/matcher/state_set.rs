use std::collections::HashMap;

use super::StringPatternMatchState;

// Preserve the allocation-free linear scan for the overwhelmingly common
// small state sets; wide sets pay once for expected-constant-time end lookup.
const MATCH_STATE_INDEX_THRESHOLD: usize = 16;
const NO_COLLISION: usize = usize::MAX;

pub(super) struct UniqueMatchStates {
    values: Vec<StringPatternMatchState>,
    index: Option<MatchStateEndIndex>,
    #[cfg(test)]
    work: MatchStateDedupWork,
}

impl UniqueMatchStates {
    pub(super) fn new() -> Self {
        Self {
            values: Vec::new(),
            index: None,
            #[cfg(test)]
            work: MatchStateDedupWork::default(),
        }
    }

    pub(super) fn insert(&mut self, candidate: StringPatternMatchState) -> bool {
        if self.index.is_none() && self.values.len() >= MATCH_STATE_INDEX_THRESHOLD {
            self.index = Some(MatchStateEndIndex::from_states(&self.values));
            #[cfg(test)]
            {
                self.work.index_builds += 1;
            }
        }

        let contains = if let Some(index) = &self.index {
            #[cfg(test)]
            let contains = {
                self.work.index_probes += 1;
                index.contains(&self.values, &candidate, &mut self.work)
            };
            #[cfg(not(test))]
            let contains = index.contains(&self.values, &candidate);
            contains
        } else {
            self.values.iter().any(|state| {
                #[cfg(test)]
                {
                    self.work.equality_comparisons += 1;
                }
                state == &candidate
            })
        };
        if contains {
            return false;
        }

        if let Some(index) = &mut self.index {
            index.insert(self.values.len(), candidate.end);
        }
        self.values.push(candidate);
        true
    }

    pub(super) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &StringPatternMatchState> {
        self.values.iter()
    }

    pub(super) fn into_vec(self) -> Vec<StringPatternMatchState> {
        self.values
    }
}

struct MatchStateEndIndex {
    first_by_end: HashMap<usize, usize>,
    next_collision: Vec<usize>,
}

impl MatchStateEndIndex {
    fn from_states(states: &[StringPatternMatchState]) -> Self {
        let mut index = Self {
            first_by_end: HashMap::with_capacity(states.len()),
            next_collision: Vec::with_capacity(states.len()),
        };
        for (position, state) in states.iter().enumerate() {
            index.insert(position, state.end);
        }
        index
    }

    fn contains(
        &self,
        states: &[StringPatternMatchState],
        candidate: &StringPatternMatchState,
        #[cfg(test)] work: &mut MatchStateDedupWork,
    ) -> bool {
        let Some(mut position) = self.first_by_end.get(&candidate.end).copied() else {
            return false;
        };

        loop {
            #[cfg(test)]
            {
                work.equality_comparisons += 1;
            }
            if states[position] == *candidate {
                return true;
            }
            let next = self.next_collision[position];
            if next == NO_COLLISION {
                return false;
            }
            position = next;
        }
    }

    fn insert(&mut self, position: usize, end: usize) {
        let previous = self.first_by_end.insert(end, position);
        self.next_collision.push(previous.unwrap_or(NO_COLLISION));
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MatchStateDedupWork {
    equality_comparisons: usize,
    index_probes: usize,
    index_builds: usize,
}

#[cfg(test)]
mod tests {
    use rum_core::{Expr, Symbol};
    use rum_runtime::kernel_api::SymbolExprMap;

    use super::*;

    fn state(end: usize) -> StringPatternMatchState {
        StringPatternMatchState {
            end,
            bindings: SymbolExprMap::default(),
        }
    }

    #[test]
    fn wide_distinct_ends_build_one_index_after_the_small_scan_tier() {
        let mut states = UniqueMatchStates::new();

        for end in 0..64 {
            assert!(states.insert(state(end)));
        }

        assert_eq!(states.values.len(), 64);
        assert_eq!(
            states.work,
            MatchStateDedupWork {
                equality_comparisons: 120,
                index_probes: 48,
                index_builds: 1,
            }
        );
    }

    #[test]
    fn indexed_collision_chain_compares_bindings_and_preserves_order() {
        let mut states = UniqueMatchStates::new();
        for end in 0..=MATCH_STATE_INDEX_THRESHOLD {
            assert!(states.insert(state(end)));
        }

        let mut bound = state(7);
        bound
            .bindings
            .insert(Symbol::from_name("x"), Expr::string("seven"));
        assert!(states.insert(bound.clone()));
        assert!(!states.insert(bound.clone()));

        let values = states.into_vec();
        assert_eq!(values.last(), Some(&bound));
        assert_eq!(values.iter().filter(|state| state.end == 7).count(), 2);
    }
}
