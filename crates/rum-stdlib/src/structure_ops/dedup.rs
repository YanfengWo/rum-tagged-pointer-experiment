use super::*;
use rum_core::SharedArgs;
use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::collections::HashSet;

const DEDUP_INDEX_THRESHOLD: usize = 16;

#[derive(FrameRootTrace)]
struct AdaptiveExprSet {
    linear: Vec<Expr>,
    indexed: Option<HashSet<Expr>>,
}

impl AdaptiveExprSet {
    fn new() -> Self {
        Self {
            linear: Vec::new(),
            indexed: None,
        }
    }

    fn insert(&mut self, value: Expr) -> bool {
        if let Some(indexed) = &mut self.indexed {
            return indexed.insert(value);
        }

        if self.linear.iter().any(|existing| existing == &value) {
            return false;
        }
        self.linear.push(value);
        if self.linear.len() > DEDUP_INDEX_THRESHOLD {
            self.indexed = Some(std::mem::take(&mut self.linear).into_iter().collect());
        }
        true
    }

    fn into_hash_set(self) -> HashSet<Expr> {
        self.indexed
            .unwrap_or_else(|| self.linear.into_iter().collect())
    }
}

#[derive(FrameRootTrace)]
struct DeleteDuplicatesTestState {
    expr_head: Expr,
    test: Expr,
    items: Vec<Expr>,
    index: usize,
    kept: Vec<Expr>,
}

pub(crate) fn eval_delete_duplicates(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    match args {
        [expr] => eval_delete_duplicates_structural(session, expr, args),
        [expr, test] => eval_delete_duplicates_with_test(session, expr, test, args),
        _ => Ok(Expr::call(head.clone(), args.to_vec())),
    }
}

fn eval_delete_duplicates_structural(
    session: &mut Session,
    expr: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_delete_duplicates_normal_call(
            session,
            "DeleteDuplicates",
            args,
        ));
    };

    Ok(Expr::call(
        expr_head.as_ref().clone(),
        stable_unique_exprs(expr_args),
    ))
}

fn eval_delete_duplicates_with_test(
    session: &mut Session,
    expr: &Expr,
    test: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_delete_duplicates_normal_call(
            session,
            "DeleteDuplicates",
            args,
        ));
    };

    resume_delete_duplicates_test_from_kept(
        session,
        DeleteDuplicatesTestState {
            expr_head: expr_head.as_ref().clone(),
            test: test.clone(),
            items: expr_args.to_vec(),
            index: 0,
            kept: Vec::new(),
        },
        0,
    )
}

pub(crate) fn eval_delete_duplicates_by(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [expr, function] = args else {
        return Ok(match args.len() {
            1 => Expr::call(head.clone(), args.to_vec()),
            _ => session.invalid_call_with_arg_range("DeleteDuplicatesBy", args, 1, 2),
        });
    };

    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_delete_duplicates_normal_call(
            session,
            "DeleteDuplicatesBy",
            args,
        ));
    };

    resume_delete_duplicates_by_from_index(
        session,
        expr_head.as_ref().clone(),
        function.clone(),
        expr_args.to_vec(),
        0,
        AdaptiveExprSet::new(),
        Vec::new(),
    )
}

fn resume_delete_duplicates_by_from_index(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    mut index: usize,
    mut keys: AdaptiveExprSet,
    mut kept: Vec<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return Ok(Expr::call(expr_head, kept));
        };

        let key = match session.evaluate(Expr::call(function.clone(), vec![item.clone()])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let items = FrozenStructureSequence::freeze(items);
                                        let keys = PersistentStructureSet::freeze(keys.into_hash_set());
                                        let kept = PersistentStructureSequence::freeze(kept);
                ResumeDeleteDuplicatesByFromIndexSuspensionFrameL178C25 { expr_head, function, items, index, keys, kept, item }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if keys.insert(key) {
            kept.push(item);
        }
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_delete_duplicates_by_from_index(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    items: FrozenStructureSequence<Expr>,
    mut index: usize,
    mut keys: PersistentStructureSet<Expr>,
    mut kept: PersistentStructureSequence<Expr>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = items.get(index).cloned() else {
            return Ok(Expr::call(expr_head, kept.to_vec()));
        };
        let key = match session.evaluate(Expr::call(function.clone(), vec![item.clone()])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedDeleteDuplicatesByFromIndexSuspensionFrameL216C25 { expr_head, function, items, index, keys, kept, item },
                );
            }
            Err(error) => return Err(error),
        };

        let (next_keys, inserted) = keys.inserted(key);
        keys = next_keys;
        if inserted {
            kept = kept.appended(item);
        }
        index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_delete_duplicates_by_with_key(
    session: &mut Session,
    expr_head: Expr,
    function: Expr,
    items: FrozenStructureSequence<Expr>,
    index: usize,
    keys: PersistentStructureSet<Expr>,
    kept: PersistentStructureSequence<Expr>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    let (keys, inserted) = keys.inserted(key);
    let kept = if inserted { kept.appended(item) } else { kept };
    resume_suspended_delete_duplicates_by_from_index(
        session,
        expr_head,
        function,
        items,
        index + 1,
        keys,
        kept,
    )
}

fn resume_delete_duplicates_test_from_kept(
    session: &mut Session,
    mut state: DeleteDuplicatesTestState,
    mut kept_index: usize,
) -> Result<Expr, EvalError> {
    loop {
        if state.index >= state.items.len() {
            return Ok(Expr::call(state.expr_head, state.kept));
        }

        if kept_index >= state.kept.len() {
            state.kept.push(state.items[state.index].clone());
            state.index += 1;
            kept_index = 0;
            continue;
        }

        let result = eval_binary_predicate(
            session,
            &state.test,
            &state.items[state.index],
            &state.kept[kept_index],
        );
        let value = match result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => {
                let items = FrozenStructureSequence::freeze(state.items);
                                        let kept = PersistentStructureSequence::freeze(state.kept);
                ResumeDeleteDuplicatesTestFromKeptSuspensionFrameL292C25 { state__expr_head: state.expr_head, state__test: state.test, state__index: state.index, items, kept, kept_index }
                },
                );
            }
            Err(error) => return Err(error),
        };

        if is_true_expr(&value) {
            state.index += 1;
            kept_index = 0;
        } else {
            kept_index += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_delete_duplicates_test_from_kept(
    session: &mut Session,
    expr_head: Expr,
    test: Expr,
    items: FrozenStructureSequence<Expr>,
    mut index: usize,
    mut kept: PersistentStructureSequence<Expr>,
    mut kept_index: usize,
) -> Result<Expr, EvalError> {
    loop {
        if index >= items.len() {
            return Ok(Expr::call(expr_head, kept.to_vec()));
        }
        if kept_index >= kept.len() {
            kept = kept.appended(items.get(index).expect("item should exist").clone());
            index += 1;
            kept_index = 0;
            continue;
        }

        let result = eval_binary_predicate(
            session,
            &test,
            items.get(index).expect("item should exist"),
            kept.get(kept_index).expect("kept item should exist"),
        );
        let value = match result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return rum_runtime::resume_or_suspend!(
                    session,
                    Err(EvalError::InputRequired { request }),
                    ok(value) => Ok(value),
                    suspend => ResumeSuspendedDeleteDuplicatesTestFromKeptSuspensionFrameL352C25 { expr_head, test, items, index, kept, kept_index },
                );
            }
            Err(error) => return Err(error),
        };

        if is_true_expr(&value) {
            index += 1;
            kept_index = 0;
        } else {
            kept_index += 1;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_suspended_delete_duplicates_test_with_value(
    session: &mut Session,
    expr_head: Expr,
    test: Expr,
    items: FrozenStructureSequence<Expr>,
    mut index: usize,
    kept: PersistentStructureSequence<Expr>,
    kept_index: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    let kept_index = if is_true_expr(&value) {
        index += 1;
        0
    } else {
        kept_index + 1
    };
    resume_suspended_delete_duplicates_test_from_kept(
        session, expr_head, test, items, index, kept, kept_index,
    )
}

fn invalid_delete_duplicates_normal_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn stable_unique_exprs(items: &[Expr]) -> Vec<Expr> {
    let mut remaining = items.iter();
    let mut kept = Vec::new();
    for item in remaining.by_ref() {
        if kept.iter().any(|existing| existing == item) {
            continue;
        }
        kept.push(item.clone());
        if kept.len() > DEDUP_INDEX_THRESHOLD {
            return stable_unique_exprs_indexed(kept, remaining);
        }
    }
    kept
}

#[inline(never)]
fn stable_unique_exprs_indexed<'a>(
    mut kept: Vec<Expr>,
    remaining: impl Iterator<Item = &'a Expr>,
) -> Vec<Expr> {
    let mut seen: HashSet<Expr> = kept.iter().cloned().collect();
    for item in remaining {
        if seen.insert(item.clone()) {
            kept.push(item.clone());
        }
    }
    kept
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<AdaptiveExprSet>();
    assert_traces::<DeleteDuplicatesTestState>();
};

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDeleteDuplicatesByFromIndexSuspensionFrameL178C25 {
    pub(super) expr_head: rum_core::Expr,
    pub(super) function: rum_core::Expr,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) keys: crate::structure_ops::suspended_state::PersistentStructureSet<rum_core::Expr>,
    pub(super) kept:
        crate::structure_ops::suspended_state::PersistentStructureSequence<rum_core::Expr>,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteDuplicatesByFromIndexSuspensionFrameL178C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut expr_head = self.expr_head.clone();
        #[allow(unused_mut)]
        let mut function = self.function.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut keys = self.keys.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_delete_duplicates_by_with_key(
                    session, expr_head, function, items, index, keys, kept, item, key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedDeleteDuplicatesByFromIndexSuspensionFrameL216C25 {
    pub(super) expr_head: rum_core::Expr,
    pub(super) function: rum_core::Expr,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) keys: crate::structure_ops::suspended_state::PersistentStructureSet<rum_core::Expr>,
    pub(super) kept:
        crate::structure_ops::suspended_state::PersistentStructureSequence<rum_core::Expr>,
    pub(super) item: rum_core::Expr,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteDuplicatesByFromIndexSuspensionFrameL216C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut expr_head = self.expr_head.clone();
        #[allow(unused_mut)]
        let mut function = self.function.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut keys = self.keys.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut item = self.item.clone();
        let resumed = match outcome {
            Ok(key) => (|| {
                resume_suspended_delete_duplicates_by_with_key(
                    session, expr_head, function, items, index, keys, kept, item, key,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeDeleteDuplicatesTestFromKeptSuspensionFrameL292C25 {
    pub(super) state__expr_head: rum_core::Expr,
    pub(super) state__test: rum_core::Expr,
    pub(super) state__index: usize,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) kept:
        crate::structure_ops::suspended_state::PersistentStructureSequence<rum_core::Expr>,
    pub(super) kept_index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeDeleteDuplicatesTestFromKeptSuspensionFrameL292C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut state__expr_head = self.state__expr_head.clone();
        #[allow(unused_mut)]
        let mut state__test = self.state__test.clone();
        #[allow(unused_mut)]
        let mut state__index = self.state__index.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut kept_index = self.kept_index.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_duplicates_test_with_value(
                    session,
                    state__expr_head,
                    state__test,
                    items,
                    state__index,
                    kept,
                    kept_index,
                    value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}

#[derive(rum_runtime::kernel_api::FrameRootTrace)]
pub(super) struct ResumeSuspendedDeleteDuplicatesTestFromKeptSuspensionFrameL352C25 {
    pub(super) expr_head: rum_core::Expr,
    pub(super) test: rum_core::Expr,
    pub(super) items:
        crate::structure_ops::suspended_state::FrozenStructureSequence<rum_core::Expr>,
    pub(super) index: usize,
    pub(super) kept:
        crate::structure_ops::suspended_state::PersistentStructureSequence<rum_core::Expr>,
    pub(super) kept_index: usize,
}

impl rum_runtime::kernel_api::SuspensionFrame
    for ResumeSuspendedDeleteDuplicatesTestFromKeptSuspensionFrameL352C25
{
    fn resume(
        self: std::sync::Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> rum_runtime::kernel_api::FrameAction {
        #[allow(unused_mut)]
        let mut expr_head = self.expr_head.clone();
        #[allow(unused_mut)]
        let mut test = self.test.clone();
        #[allow(unused_mut)]
        let mut items = self.items.clone();
        #[allow(unused_mut)]
        let mut index = self.index.clone();
        #[allow(unused_mut)]
        let mut kept = self.kept.clone();
        #[allow(unused_mut)]
        let mut kept_index = self.kept_index.clone();
        let resumed = match outcome {
            Ok(value) => (|| {
                resume_suspended_delete_duplicates_test_with_value(
                    session, expr_head, test, items, index, kept, kept_index, value,
                )
            })(),
            Err(error) => Err(error),
        };
        rum_runtime::kernel_api::FrameAction::complete(resumed)
    }
}
