use rum_core::Expr;
use rum_runtime::kernel_api::{EvalError, FrameRootTrace, FrameRootVisitor, Session};

use super::is_true_expr;

#[derive(Clone, Copy, FrameRootTrace)]
pub(crate) enum ComparatorResultPolicy {
    ExactTrue,
    Custom(fn(&Expr) -> bool),
}

impl ComparatorResultPolicy {
    fn strictly_precedes(self, value: &Expr) -> bool {
        match self {
            Self::ExactTrue => is_true_expr(value),
            Self::Custom(decision) => decision(value),
        }
    }
}

pub(crate) trait MergeSortItem {
    fn ordering_value(&self) -> &Expr;
}

pub(crate) trait MergeCompletion<T>: FrameRootTrace + Send + 'static {
    fn complete(self, items: Vec<T>) -> Expr;
}

impl MergeSortItem for Expr {
    fn ordering_value(&self) -> &Expr {
        self
    }
}

#[derive(FrameRootTrace)]
struct Run<T> {
    items: Vec<Option<T>>,
    cursor: usize,
}

impl<T> Run<T> {
    fn singleton(item: T) -> Self {
        Self {
            items: vec![Some(item)],
            cursor: 0,
        }
    }

    fn from_items(items: Vec<Option<T>>) -> Self {
        Self { items, cursor: 0 }
    }

    fn front(&self) -> Option<&T> {
        self.items.get(self.cursor)?.as_ref()
    }

    fn take_front(&mut self) -> Option<T> {
        let item = self.items.get_mut(self.cursor)?.take()?;
        self.cursor += 1;
        Some(item)
    }

    fn is_empty(&self) -> bool {
        self.cursor >= self.items.len()
    }

    fn into_items(self) -> Vec<T> {
        self.items.into_iter().skip(self.cursor).flatten().collect()
    }
}

#[derive(FrameRootTrace)]
struct MergeSortState<T, C> {
    ordering: Expr,
    result_policy: ComparatorResultPolicy,
    completion: C,
    runs: Vec<Run<T>>,
    pair_index: usize,
    merged: Vec<Option<T>>,
    next_runs: Vec<Run<T>>,
}

pub(crate) fn sort_with_comparator<T, C>(
    session: &mut Session,
    items: Vec<T>,
    ordering: Expr,
    result_policy: ComparatorResultPolicy,
    completion: C,
) -> Result<Expr, EvalError>
where
    T: MergeSortItem + Send + 'static,
    C: MergeCompletion<T>,
{
    let runs = items.into_iter().map(Run::singleton).collect();
    resume_merge_sort(
        session,
        MergeSortState {
            ordering,
            result_policy,
            completion,
            runs,
            pair_index: 0,
            merged: Vec::new(),
            next_runs: Vec::new(),
        },
    )
}

fn resume_merge_sort<T, C>(
    session: &mut Session,
    mut state: MergeSortState<T, C>,
) -> Result<Expr, EvalError>
where
    T: MergeSortItem + Send + 'static,
    C: MergeCompletion<T>,
{
    loop {
        if state.runs.len() <= 1 && state.next_runs.is_empty() {
            let sorted = state.runs.pop().map(Run::into_items).unwrap_or_default();
            return Ok(state.completion.complete(sorted));
        }

        if state.pair_index >= state.runs.len() {
            state.runs = std::mem::take(&mut state.next_runs);
            state.pair_index = 0;
            continue;
        }

        let right_index = state.pair_index + 1;
        if right_index >= state.runs.len() {
            state.drain_left();
            state.finish_pair();
            continue;
        }
        if state.runs[state.pair_index].is_empty() {
            state.drain_right();
            state.finish_pair();
            continue;
        }
        if state.runs[right_index].is_empty() {
            state.drain_left();
            state.finish_pair();
            continue;
        }

        let comparison = state.comparison_expression();
        let value = match session.evaluate(comparison) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        state.apply_comparison(&value);
                        resume_merge_sort(session, state)
                    },
                );
            }
            Err(error) => return Err(error),
        };
        state.apply_comparison(&value);
    }
}

impl<T, C> MergeSortState<T, C>
where
    T: MergeSortItem,
{
    fn comparison_expression(&self) -> Expr {
        let left = self.runs[self.pair_index]
            .front()
            .expect("left merge run should have a candidate")
            .ordering_value()
            .clone();
        let right = self.runs[self.pair_index + 1]
            .front()
            .expect("right merge run should have a candidate")
            .ordering_value()
            .clone();
        Expr::call(self.ordering.clone(), vec![right, left])
    }

    fn apply_comparison(&mut self, value: &Expr) {
        if self.result_policy.strictly_precedes(value) {
            let item = self.runs[self.pair_index + 1]
                .take_front()
                .expect("right merge candidate should still exist");
            self.merged.push(Some(item));
        } else {
            let item = self.runs[self.pair_index]
                .take_front()
                .expect("left merge candidate should still exist");
            self.merged.push(Some(item));
        }
    }

    fn drain_left(&mut self) {
        while let Some(item) = self.runs[self.pair_index].take_front() {
            self.merged.push(Some(item));
        }
    }

    fn drain_right(&mut self) {
        while let Some(item) = self.runs[self.pair_index + 1].take_front() {
            self.merged.push(Some(item));
        }
    }

    fn finish_pair(&mut self) {
        self.next_runs
            .push(Run::from_items(std::mem::take(&mut self.merged)));
        self.pair_index += 2;
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<ComparatorResultPolicy>();
    assert_traces::<MergeSortState<Expr, Expr>>();
    assert_traces::<Run<Expr>>();
};
