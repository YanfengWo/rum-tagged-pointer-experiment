use super::*;

mod aggregate;
mod key_select;
mod key_sort;
mod map;

pub(crate) use self::{
    aggregate::{eval_counts_by, eval_merge},
    key_select::eval_key_select,
    key_sort::{eval_key_sort_by, resume_key_sort_with_ordering_from_index},
    map::eval_association_map,
};

pub(super) fn one_or_two_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; 1 or 2 arguments are expected.")
}
