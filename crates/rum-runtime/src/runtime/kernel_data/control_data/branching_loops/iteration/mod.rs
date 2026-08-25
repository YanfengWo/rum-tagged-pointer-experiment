use super::*;

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
        let parent_product = self.iteration_product_budget.get();
        match evaluate(self) {
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, outcome| {
                    session.iteration_product_budget.set(parent_product);
                    outcome
                });
                Err(EvalError::InputRequired { request })
            }
            outcome => {
                self.iteration_product_budget.set(parent_product);
                outcome
            }
        }
    }
}

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
