use super::*;

mod driver;
mod spec;
mod table_fast_path;

pub(super) use spec::{build_iteration_body_or_nested, evaluate_do_spec_expr, parse_do_spec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Vec<Expr> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IterationConsumer {
    Do,
    Product,
    Sum,
    Table,
}

impl DoSpec {
    pub(super) fn iteration_count(&self) -> usize {
        match self {
            Self::Count(count) => *count,
            Self::Iterator { values, .. } => values.len(),
        }
    }
}

impl IterationConsumer {
    fn accepts_bare_count(self) -> bool {
        matches!(self, Self::Do | Self::Table)
    }
}
