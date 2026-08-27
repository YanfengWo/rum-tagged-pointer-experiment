use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

mod driver;
mod spec;
mod table_fast_path;

pub(super) use spec::{build_iteration_body_or_nested, evaluate_do_spec_expr, parse_do_spec};

impl Session {
    pub(in crate::runtime) fn with_iteration_product_scope<F>(
        &mut self,
        evaluate: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&mut Session) -> Result<Expr, EvalError>,
    {
        let parent_budget = self.iteration_product_budget.get();
        match evaluate(self) {
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, outcome| {
                    session.iteration_product_budget.set(parent_budget);
                    outcome
                });
                Err(EvalError::InputRequired { request })
            }
            outcome => {
                self.iteration_product_budget.set(parent_budget);
                outcome
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(super) enum DoSpec {
    Count(usize),
    Iterator { symbol: Symbol, values: Vec<Expr> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
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
    fn product_scope_tag(self) -> u8 {
        self as u8 + 1
    }

    fn inherits_product_from(self, enclosing_consumer_tag: u8) -> bool {
        enclosing_consumer_tag == self.product_scope_tag()
            || enclosing_consumer_tag == Self::Table.product_scope_tag()
    }

    fn accepts_bare_count(self) -> bool {
        matches!(self, Self::Do | Self::Table)
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<DoSpec>();
};
