use super::*;

mod api;
mod engine;
mod split_trim;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StringPatternMatchState {
    pub(super) end: usize,
    pub(super) bindings: SymbolExprMap,
}

pub(super) use api::{
    collect_string_cases, split_string_value, string_contains_q, string_count_matches,
    string_delete_matches, string_ends_q, string_free_q, string_match_q, string_position_expr,
    string_starts_q, trim_string_value,
};
pub(super) use engine::{
    best_pattern_match_at_with_bindings, best_pattern_match_state, pattern_match_text,
};
