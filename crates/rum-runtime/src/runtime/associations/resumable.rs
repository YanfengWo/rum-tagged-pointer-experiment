use super::*;

mod aggregate;
mod key_select;
mod key_sort;
mod map;

pub(super) fn one_or_two_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; 1 or 2 arguments are expected.")
}
