use std::collections::HashMap;

use super::*;

const GROUP_INDEX_THRESHOLD: usize = 16;
const GROUP_INDEX_MIN_REMAINING: usize = 32;

pub(super) fn group_index_worthwhile(input_len: usize) -> bool {
    input_len > GROUP_INDEX_THRESHOLD + GROUP_INDEX_MIN_REMAINING
}

pub(super) fn existing_group_index(
    grouped: &[(Expr, Rational)],
    index: &mut Option<HashMap<Expr, usize>>,
    key: &Expr,
    indexing_enabled: bool,
) -> Option<usize> {
    if index.is_none() && indexing_enabled && grouped.len() >= GROUP_INDEX_THRESHOLD {
        let mut built = HashMap::with_capacity(grouped.len() + 1);
        built.extend(
            grouped
                .iter()
                .enumerate()
                .map(|(position, (key, _))| (key.clone(), position)),
        );
        *index = Some(built);
    }

    match index {
        Some(index) => index.get(key).copied(),
        None => grouped.iter().position(|(candidate, _)| candidate == key),
    }
}

pub(super) fn record_group_index(
    grouped: &[(Expr, Rational)],
    index: &mut Option<HashMap<Expr, usize>>,
    position: usize,
) {
    if let Some(index) = index {
        index.insert(grouped[position].0.clone(), position);
    }
}
