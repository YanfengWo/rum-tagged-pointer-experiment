mod resume_frames;
use resume_frames::*;

use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct CallArgumentPlan {
    expr: Expr,
    mode: CallArgumentMode,
    allow_sequence_splice: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallArgumentMode {
    Evaluate,
    Hold,
    StrippedUnevaluated,
}

pub(super) struct PreparedCallArguments {
    pub(super) evaluated_args: SharedArgs,
    pub(super) unevaluated_restore: Option<Box<UnevaluatedRestorePlan>>,
}

#[derive(FrameRootTrace)]
pub(super) struct UnevaluatedRestorePlan {
    pub(super) prepared_args: SharedArgs,
    pub(super) wrappers: Vec<(usize, Expr)>,
}

#[derive(Clone, FrameRootTrace)]
struct PreparedArgumentAccumulator {
    prefix: SharedArgs,
    tail: Option<Arc<PreparedArgumentNode>>,
    len: usize,
}

#[derive(FrameRootTrace)]
struct PreparedArgumentNode {
    previous: Option<Arc<PreparedArgumentNode>>,
    value: Expr,
}

#[derive(Clone, FrameRootTrace)]
struct UnevaluatedRestoreAccumulator {
    prefix: Arc<[(usize, Expr)]>,
    tail: Option<Arc<UnevaluatedRestoreNode>>,
    len: usize,
}

#[derive(FrameRootTrace)]
struct UnevaluatedRestoreNode {
    previous: Option<Arc<UnevaluatedRestoreNode>>,
    value: (usize, Expr),
}

#[derive(Clone, FrameRootTrace)]
struct SuspendedCallArgumentPreparation {
    head: Expr,
    attributes: AttributeSet,
    args: SharedArgs,
    cursor: usize,
    prepared: PreparedArgumentAccumulator,
    unevaluated_restores: Option<UnevaluatedRestoreAccumulator>,
}

struct SuspendedPreparedCallArguments {
    evaluated_args: PreparedArgumentAccumulator,
    unevaluated_restores: Option<UnevaluatedRestoreAccumulator>,
}

impl PreparedArgumentAccumulator {
    fn freeze(values: Vec<Expr>) -> Self {
        let len = values.len();
        Self {
            prefix: SharedArgs::new(values),
            tail: None,
            len,
        }
    }

    fn appended(&self, value: Expr) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: Some(Arc::new(PreparedArgumentNode {
                previous: self.tail.clone(),
                value,
            })),
            len: self.len + 1,
        }
    }

    fn into_vec(self) -> Vec<Expr> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail;
        while let Some(current) = node {
            let current = Arc::try_unwrap(current)
                .unwrap_or_else(|_| panic!("completed call argument state must be unique"));
            suffix.push(current.value);
            node = current.previous;
        }
        suffix.reverse();

        let mut values = self.prefix.into_vec();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

impl UnevaluatedRestoreAccumulator {
    fn freeze(values: Vec<(usize, Expr)>) -> Self {
        let len = values.len();
        Self {
            prefix: Arc::from(values),
            tail: None,
            len,
        }
    }

    fn appended(&self, value: (usize, Expr)) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: Some(Arc::new(UnevaluatedRestoreNode {
                previous: self.tail.clone(),
                value,
            })),
            len: self.len + 1,
        }
    }

    fn into_vec(self) -> Vec<(usize, Expr)> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail;
        while let Some(current) = node {
            let current = Arc::try_unwrap(current)
                .unwrap_or_else(|_| panic!("completed Unevaluated restore state must be unique"));
            suffix.push(current.value);
            node = current.previous;
        }
        suffix.reverse();

        let mut values = self.prefix.iter().cloned().collect::<Vec<_>>();
        values.reserve(suffix.len());
        values.extend(suffix);
        values
    }
}

fn plan_call_argument_preparation(
    attributes: AttributeSet,
    index: usize,
    arg: Expr,
) -> CallArgumentPlan {
    let hold = attributes.hold_attribute();
    let blocks_evaluate_override = attributes.blocks_evaluate_override();

    if hold.holds_argument(index) {
        if blocks_evaluate_override {
            return CallArgumentPlan {
                expr: arg,
                mode: CallArgumentMode::Hold,
                allow_sequence_splice: false,
            };
        }

        if let Some(inner) = unevaluated_wrapper_arg(&arg) {
            return CallArgumentPlan {
                expr: inner.clone(),
                mode: CallArgumentMode::StrippedUnevaluated,
                allow_sequence_splice: true,
            };
        }

        if let Some(inner) = evaluate_wrapper_arg(&arg) {
            return CallArgumentPlan {
                expr: inner.clone(),
                mode: CallArgumentMode::Evaluate,
                allow_sequence_splice: true,
            };
        }

        return CallArgumentPlan {
            expr: arg,
            mode: CallArgumentMode::Hold,
            allow_sequence_splice: true,
        };
    }

    if let Some(inner) = unevaluated_wrapper_arg(&arg) {
        return CallArgumentPlan {
            expr: inner.clone(),
            mode: CallArgumentMode::StrippedUnevaluated,
            allow_sequence_splice: true,
        };
    }

    CallArgumentPlan {
        expr: arg,
        mode: CallArgumentMode::Evaluate,
        allow_sequence_splice: true,
    }
}

fn append_prepared_argument(
    prepared_args: &mut Vec<Expr>,
    value: Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) {
    if allow_sequence_splice && !sequence_hold {
        splice_sequence_argument(prepared_args, value);
    } else {
        prepared_args.push(value);
    }
}

fn append_prepared_list_argument(
    prepared_args: &mut Vec<Expr>,
    value: Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) {
    if allow_sequence_splice && !sequence_hold {
        match value {
            Expr::Call { head, args }
                if is_system_symbol_name(head.as_ref(), heads::functions::SEQUENCE) =>
            {
                prepared_args.extend(
                    args.into_iter()
                        .filter(|item| !is_system_symbol_name(item, "Nothing")),
                );
            }
            other => append_prepared_list_item(prepared_args, other),
        }
    } else {
        append_prepared_list_item(prepared_args, value);
    }
}

fn append_prepared_list_item(prepared_args: &mut Vec<Expr>, value: Expr) {
    if !is_system_symbol_name(&value, "Nothing") {
        prepared_args.push(value);
    }
}

fn append_suspended_prepared_argument(
    mut prepared: PreparedArgumentAccumulator,
    value: Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
    list_drops_nothing: bool,
) -> PreparedArgumentAccumulator {
    if allow_sequence_splice && !sequence_hold {
        if let Some(items) = sequence_items(&value) {
            for item in items {
                if !list_drops_nothing || !is_nothing_symbol(item) {
                    prepared = prepared.appended(item.clone());
                }
            }
            return prepared;
        }
    }

    if !list_drops_nothing || !is_nothing_symbol(&value) {
        prepared = prepared.appended(value);
    }
    prepared
}

fn finish_suspended_prepared_call(state: SuspendedPreparedCallArguments) -> Expr {
    let evaluated_args = state.evaluated_args.into_vec();
    let restores = state
        .unevaluated_restores
        .map(UnevaluatedRestoreAccumulator::into_vec)
        .unwrap_or_default()
        .into_iter()
        .map(|(position, wrapper)| {
            Expr::call(
                system_expr("InternalContinuationUnevaluatedRestore"),
                vec![Expr::integer(BigInt::from(position)), wrapper],
            )
        })
        .collect::<Vec<_>>();
    Expr::call(
        system_expr("InternalContinuationPreparedCallArguments"),
        vec![
            Expr::call(system_expr("List"), evaluated_args),
            Expr::call(system_expr("List"), restores),
        ],
    )
}

pub(super) fn prepared_call_arguments_from_continuation_expr(
    expr: Expr,
) -> Result<PreparedCallArguments, Expr> {
    let Expr::Call { head, args } = expr else {
        return Err(expr);
    };
    if !is_system_symbol_name(&head, "InternalContinuationPreparedCallArguments") {
        return Err(Expr::Call { head, args });
    }
    let mut parts = args.into_iter();
    let evaluated_expr = parts
        .next()
        .expect("prepared-call continuation stores evaluated arguments");
    let restores_expr = parts
        .next()
        .expect("prepared-call continuation stores Unevaluated wrappers");
    assert!(parts.next().is_none());

    let Expr::Call {
        head: evaluated_head,
        args: evaluated_args,
    } = evaluated_expr
    else {
        panic!("prepared-call arguments should be stored as a List")
    };
    assert!(is_system_symbol_name(&evaluated_head, "List"));
    let Expr::Call {
        head: restores_head,
        args: restores,
    } = restores_expr
    else {
        panic!("prepared-call restore entries should be stored as a List")
    };
    assert!(is_system_symbol_name(&restores_head, "List"));

    let wrappers = restores
        .into_iter()
        .map(|entry| {
            let Expr::Call { head, args } = entry else {
                panic!("prepared-call restore entry should be a call")
            };
            assert!(is_system_symbol_name(
                &head,
                "InternalContinuationUnevaluatedRestore"
            ));
            let mut fields = args.into_iter();
            let position = fields
                .next()
                .as_ref()
                .and_then(expr_as_non_negative_usize)
                .expect("prepared-call restore position should be non-negative");
            let wrapper = fields
                .next()
                .expect("prepared-call restore entry stores its wrapper");
            assert!(fields.next().is_none());
            (position, wrapper)
        })
        .collect::<Vec<_>>();
    let unevaluated_restore = (!wrappers.is_empty()).then(|| {
        Box::new(UnevaluatedRestorePlan {
            prepared_args: evaluated_args.clone(),
            wrappers,
        })
    });
    Ok(PreparedCallArguments {
        evaluated_args,
        unevaluated_restore,
    })
}

fn prepared_argument_splices(
    value: &Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) -> bool {
    allow_sequence_splice && !sequence_hold && sequence_items(value).is_some()
}

fn prepared_list_argument_drops_nothing(
    value: &Expr,
    allow_sequence_splice: bool,
    sequence_hold: bool,
) -> bool {
    if is_nothing_symbol(value) {
        return true;
    }

    allow_sequence_splice
        && !sequence_hold
        && sequence_items(value).is_some_and(|items| items.iter().any(is_nothing_symbol))
}

fn list_args_drop_nothing(head: &Expr, args: &[Expr]) -> bool {
    is_system_symbol_name(head, heads::structure::LIST) && args.iter().any(is_nothing_symbol)
}

fn is_nothing_symbol(expr: &Expr) -> bool {
    is_system_symbol_name(expr, "Nothing")
}

fn attribute_normalization_is_noop(head: &Expr, args: &[Expr], attributes: AttributeSet) -> bool {
    if list_args_drop_nothing(head, args) {
        return false;
    }

    if attributes.contains(Attribute::Flat)
        && args
            .iter()
            .any(|arg| call_items_with_head(arg, head).is_some())
    {
        return false;
    }

    if attributes.contains(Attribute::Orderless)
        && args
            .windows(2)
            .any(|pair| compare_expr_canonical(&pair[0], &pair[1]).is_gt())
    {
        return false;
    }

    true
}

impl Session {
    pub(super) fn prepare_arguments_for_call(
        &mut self,
        head: Expr,
        attributes: AttributeSet,
        args: SharedArgs,
    ) -> Result<SharedArgs, EvalError> {
        self.prepare_arguments_for_call_with_unevaluated_restore(head, attributes, args)
            .map(|prepared| prepared.evaluated_args)
    }

    pub(super) fn prepare_arguments_for_call_with_unevaluated_restore(
        &mut self,
        head: Expr,
        attributes: AttributeSet,
        args: SharedArgs,
    ) -> Result<PreparedCallArguments, EvalError> {
        let runtime_profile = runtime_path_timing_start("prepare_arguments.total");
        let result = (|| {
            let sequence_hold = attributes.sequence_hold();
            let list_drops_nothing = is_system_symbol_name(&head, heads::structure::LIST);
            let mut built: Option<Vec<Expr>> = None;
            let mut unevaluated_restores: Option<Vec<(usize, Expr)>> = None;

            for index in 0..args.len() {
                let runtime_profile_plan = runtime_path_timing_start("prepare_arguments.plan_arg");
                let raw_arg = args[index].clone();
                let plan = plan_call_argument_preparation(attributes, index, raw_arg.clone());
                record_profile_runtime_path_timing(runtime_profile_plan, RuntimeProfileOutcome::Ok);

                let prepared_arg = match plan.mode {
                    CallArgumentMode::Evaluate => {
                        let allow_sequence_splice = plan.allow_sequence_splice;
                        let runtime_profile_evaluate =
                            runtime_path_timing_start("prepare_arguments.evaluate_arg");
                        let evaluated_result = self.evaluate_atom_fast_path(plan.expr);
                        record_profile_runtime_path_timing(
                            runtime_profile_evaluate,
                            runtime_profile_outcome_for_expr_result(&evaluated_result),
                        );
                        match evaluated_result {
                            Ok(value) => value,
                            Err(EvalError::InputRequired { request }) => {
                                let prepared =
                                    built.take().unwrap_or_else(|| args[..index].to_vec());
                                let state = SuspendedCallArgumentPreparation {
                                    head,
                                    attributes,
                                    args,
                                    cursor: index + 1,
                                    prepared: PreparedArgumentAccumulator::freeze(prepared),
                                    unevaluated_restores: unevaluated_restores
                                        .map(UnevaluatedRestoreAccumulator::freeze),
                                };
                                crate::resume_or_suspend!(insert self, PrepareArgumentsForCallWithUnevaluatedRestoreSuspensionFrameL373C60 { state, allow_sequence_splice });
                                return Err(EvalError::InputRequired { request });
                            }
                            Err(error) => return Err(error),
                        }
                    }
                    CallArgumentMode::Hold => plan.expr,
                    CallArgumentMode::StrippedUnevaluated => {
                        let splices = prepared_argument_splices(
                            &plan.expr,
                            plan.allow_sequence_splice,
                            sequence_hold,
                        );
                        let drops_nothing = list_drops_nothing
                            && prepared_list_argument_drops_nothing(
                                &plan.expr,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        if !splices && !drops_nothing {
                            let prepared_position = built.as_ref().map_or(index, Vec::len);
                            unevaluated_restores
                                .get_or_insert_with(Vec::new)
                                .push((prepared_position, raw_arg));
                        }
                        plan.expr
                    }
                };

                let splices = prepared_argument_splices(
                    &prepared_arg,
                    plan.allow_sequence_splice,
                    sequence_hold,
                );
                let drops_nothing = list_drops_nothing
                    && prepared_list_argument_drops_nothing(
                        &prepared_arg,
                        plan.allow_sequence_splice,
                        sequence_hold,
                    );
                let changed = splices || drops_nothing || prepared_arg != args[index];

                match (built.as_mut(), changed) {
                    (None, false) => {}
                    (None, true) => {
                        let mut prepared_args = args[..index].to_vec();
                        let runtime_profile_append =
                            runtime_path_timing_start("prepare_arguments.append_arg");
                        if list_drops_nothing {
                            append_prepared_list_argument(
                                &mut prepared_args,
                                prepared_arg,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        } else {
                            append_prepared_argument(
                                &mut prepared_args,
                                prepared_arg,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        }
                        record_profile_runtime_path_timing(
                            runtime_profile_append,
                            RuntimeProfileOutcome::Ok,
                        );
                        built = Some(prepared_args);
                    }
                    (Some(prepared_args), _) => {
                        let runtime_profile_append =
                            runtime_path_timing_start("prepare_arguments.append_arg");
                        if list_drops_nothing {
                            append_prepared_list_argument(
                                prepared_args,
                                prepared_arg,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        } else {
                            append_prepared_argument(
                                prepared_args,
                                prepared_arg,
                                plan.allow_sequence_splice,
                                sequence_hold,
                            );
                        }
                        record_profile_runtime_path_timing(
                            runtime_profile_append,
                            RuntimeProfileOutcome::Ok,
                        );
                    }
                }
            }

            let evaluated_args = built.map(SharedArgs::new).unwrap_or(args);
            let unevaluated_restore = unevaluated_restores.map(|wrappers| {
                Box::new(UnevaluatedRestorePlan {
                    prepared_args: evaluated_args.clone(),
                    wrappers,
                })
            });

            Ok(PreparedCallArguments {
                evaluated_args,
                unevaluated_restore,
            })
        })();
        record_profile_runtime_path_timing(
            runtime_profile,
            runtime_profile_outcome_for_shared_args_result(&result),
        );
        result
    }

    fn resume_suspended_call_after_argument(
        &mut self,
        mut state: SuspendedCallArgumentPreparation,
        value: Expr,
        allow_sequence_splice: bool,
    ) -> Result<SuspendedPreparedCallArguments, EvalError> {
        let list_drops_nothing = is_system_symbol_name(&state.head, heads::structure::LIST);
        state.prepared = append_suspended_prepared_argument(
            state.prepared,
            value,
            allow_sequence_splice,
            state.attributes.sequence_hold(),
            list_drops_nothing,
        );
        self.resume_suspended_call_argument_preparation(state)
    }

    fn resume_suspended_call_argument_preparation(
        &mut self,
        mut state: SuspendedCallArgumentPreparation,
    ) -> Result<SuspendedPreparedCallArguments, EvalError> {
        let sequence_hold = state.attributes.sequence_hold();
        let list_drops_nothing = is_system_symbol_name(&state.head, heads::structure::LIST);

        loop {
            let Some(arg) = state.args.get(state.cursor).cloned() else {
                return Ok(SuspendedPreparedCallArguments {
                    evaluated_args: state.prepared,
                    unevaluated_restores: state.unevaluated_restores,
                });
            };

            let index = state.cursor;
            let raw_arg = arg.clone();
            let runtime_profile_plan = runtime_path_timing_start("prepare_arguments.plan_arg");
            let plan = plan_call_argument_preparation(state.attributes, index, arg);
            record_profile_runtime_path_timing(runtime_profile_plan, RuntimeProfileOutcome::Ok);

            let prepared_arg = match plan.mode {
                CallArgumentMode::Evaluate => {
                    let allow_sequence_splice = plan.allow_sequence_splice;
                    let runtime_profile_evaluate =
                        runtime_path_timing_start("prepare_arguments.evaluate_arg");
                    let evaluated_result = self.evaluate_atom_fast_path(plan.expr);
                    record_profile_runtime_path_timing(
                        runtime_profile_evaluate,
                        runtime_profile_outcome_for_expr_result(&evaluated_result),
                    );
                    match evaluated_result {
                        Ok(value) => value,
                        Err(EvalError::InputRequired { request }) => {
                            state.cursor += 1;
                            crate::resume_or_suspend!(insert self, ResumeSuspendedCallArgumentPreparationSuspensionFrameL558C56 { state, allow_sequence_splice });
                            return Err(EvalError::InputRequired { request });
                        }
                        Err(error) => return Err(error),
                    }
                }
                CallArgumentMode::Hold => plan.expr,
                CallArgumentMode::StrippedUnevaluated => {
                    let splices = prepared_argument_splices(
                        &plan.expr,
                        plan.allow_sequence_splice,
                        sequence_hold,
                    );
                    let drops_nothing = list_drops_nothing
                        && prepared_list_argument_drops_nothing(
                            &plan.expr,
                            plan.allow_sequence_splice,
                            sequence_hold,
                        );
                    if !splices && !drops_nothing {
                        let entry = (state.prepared.len, raw_arg);
                        state.unevaluated_restores = Some(
                            state
                                .unevaluated_restores
                                .take()
                                .unwrap_or_else(
                                    || UnevaluatedRestoreAccumulator::freeze(Vec::new()),
                                )
                                .appended(entry),
                        );
                    }
                    plan.expr
                }
            };

            let runtime_profile_append = runtime_path_timing_start("prepare_arguments.append_arg");
            state.prepared = append_suspended_prepared_argument(
                state.prepared,
                prepared_arg,
                plan.allow_sequence_splice,
                sequence_hold,
                list_drops_nothing,
            );
            state.cursor += 1;
            record_profile_runtime_path_timing(runtime_profile_append, RuntimeProfileOutcome::Ok);
        }
    }

    pub(super) fn resume_call_from_prepared_args(
        &mut self,
        head: Expr,
        prepared_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        self.resume_call_from_prepared_args_with_unevaluated_restore(
            head,
            PreparedCallArguments {
                evaluated_args: SharedArgs::new(prepared_args),
                unevaluated_restore: None,
            },
        )
    }

    fn resume_call_from_prepared_args_with_unevaluated_restore(
        &mut self,
        head: Expr,
        prepared: PreparedCallArguments,
    ) -> Result<Expr, EvalError> {
        let plan = self.build_call_dispatch_plan(&head);
        let state = CallDispatchState {
            raw_args: SharedArgs::default(),
            evaluated_args: Some(self.normalize_call_args_shared(
                &head,
                prepared.evaluated_args,
                plan.attributes,
            )),
        };

        self.resume_call_with_dispatch_plan(head, plan, state, 0, prepared.unevaluated_restore)
    }

    pub(super) fn attribute_set_for_head(&self, head: &Expr) -> AttributeSet {
        if let Some(function) = parse_function(head) {
            return function.attributes();
        }

        let Some(symbol) = expr_as_symbol(head) else {
            return AttributeSet::empty();
        };

        self.attribute_set_for_symbol(symbol)
    }

    pub(super) fn cached_attribute_set_for_head(&mut self, head: &Expr) -> AttributeSet {
        if let Some(function) = parse_function(head) {
            return function.attributes();
        }

        let Some(symbol) = expr_as_symbol(head) else {
            return AttributeSet::empty();
        };

        self.cached_attribute_set_for_symbol(symbol)
    }

    pub(super) fn normalize_call_args_by_attributes(
        &self,
        head: &Expr,
        args: Vec<Expr>,
        attributes: AttributeSet,
    ) -> Vec<Expr> {
        let runtime_profile_total = runtime_path_timing_start("prepare_arguments.normalize.total");
        let mut normalized_args = if attributes.contains(Attribute::Flat) {
            let runtime_profile_flat =
                runtime_path_timing_start("prepare_arguments.normalize.flat");
            let result = flatten_flat_call_args(head, args);
            record_profile_runtime_path_timing(runtime_profile_flat, RuntimeProfileOutcome::Ok);
            result
        } else {
            args
        };

        if attributes.contains(Attribute::Orderless) {
            let runtime_profile_orderless =
                runtime_path_timing_start("prepare_arguments.normalize.orderless");
            normalized_args.sort_by(compare_expr_canonical);
            record_profile_runtime_path_timing(
                runtime_profile_orderless,
                RuntimeProfileOutcome::Ok,
            );
        }

        if is_system_symbol_name(head, heads::structure::LIST) {
            normalized_args.retain(|arg| !is_nothing_symbol(arg));
        }

        record_profile_runtime_path_timing(runtime_profile_total, RuntimeProfileOutcome::Ok);
        normalized_args
    }

    pub(super) fn normalize_call_args_shared(
        &self,
        head: &Expr,
        args: SharedArgs,
        attributes: AttributeSet,
    ) -> SharedArgs {
        if attribute_normalization_is_noop(head, args.as_ref(), attributes) {
            return args;
        }

        SharedArgs::new(self.normalize_call_args_by_attributes(
            head,
            Expr::unwrap_shared_args(args),
            attributes,
        ))
    }
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

fn runtime_profile_outcome_for_shared_args_result<T>(
    result: &Result<T, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_keeps_held_arguments_verbatim_under_holdallcomplete() {
        let arg = Expr::call(system_expr("Evaluate"), vec![Expr::integer(1_i64.into())]);
        let attributes = AttributeSet::from_attribute(Attribute::HoldAllComplete);
        let plan = plan_call_argument_preparation(attributes, 0, arg.clone());

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: arg,
                mode: CallArgumentMode::Hold,
                allow_sequence_splice: false,
            }
        );
    }

    #[test]
    fn planner_respects_evaluate_wrapper_for_held_arguments() {
        let inner = Expr::Atom(Atom::Symbol(Symbol::from_name("x")));
        let arg = Expr::call(system_expr("Evaluate"), vec![inner.clone()]);
        let attributes = AttributeSet::from_attribute(Attribute::HoldAll);
        let plan = plan_call_argument_preparation(attributes, 0, arg);

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: inner,
                mode: CallArgumentMode::Evaluate,
                allow_sequence_splice: true,
            }
        );
    }

    #[test]
    fn planner_allows_raw_sequence_splicing_for_held_arguments() {
        let arg = Expr::call(
            system_expr("Sequence"),
            vec![Expr::integer(1_i64.into()), Expr::integer(2_i64.into())],
        );
        let attributes = AttributeSet::from_attribute(Attribute::HoldAll);
        let plan = plan_call_argument_preparation(attributes, 0, arg.clone());

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: arg,
                mode: CallArgumentMode::Hold,
                allow_sequence_splice: true,
            }
        );
    }

    #[test]
    fn planner_respects_unevaluated_wrapper_for_nonheld_arguments() {
        let inner = Expr::Atom(Atom::Symbol(Symbol::from_name("x")));
        let arg = Expr::call(system_expr("Unevaluated"), vec![inner.clone()]);
        let plan = plan_call_argument_preparation(AttributeSet::empty(), 0, arg);

        assert_eq!(
            plan,
            CallArgumentPlan {
                expr: inner,
                mode: CallArgumentMode::StrippedUnevaluated,
                allow_sequence_splice: true,
            }
        );
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<PreparedArgumentAccumulator>();
    assert_traces::<UnevaluatedRestoreAccumulator>();
    assert_traces::<SuspendedCallArgumentPreparation>();
    assert_traces::<UnevaluatedRestorePlan>();
};
