use super::model::*;
use super::*;
use std::sync::{Arc, Mutex};

pub(super) fn execute_compiled_body(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
) -> Result<Expr, EvalError> {
    let state = runtime_state(session, &prepared.code, prepared.slot_values, option_scope);
    (prepared.code.body)(session, state)
}

fn runtime_state(
    session: &mut Session,
    code: &CodeObject,
    slot_values: Vec<Expr>,
    option_scope: Option<OptionValueScope>,
) -> RuntimeStateRef {
    let locals = code
        .local_specs
        .iter()
        .map(|spec| match spec.kind {
            LocalKind::Module => LocalRuntime::Module {
                fresh: session.allocate_fresh_symbol(&spec.symbol),
                value: None,
            },
            LocalKind::With => LocalRuntime::With { value: None },
        })
        .collect();
    Arc::new(Mutex::new(RuntimeState {
        slot_values,
        locals,
        option_scope,
    }))
}

pub(super) fn instantiate_template(
    template: &TemplateNode,
    state: &RuntimeStateRef,
) -> Option<Expr> {
    match template {
        TemplateNode::Const(expr) => Some(expr.as_ref().clone()),
        TemplateNode::Binding(slot) => {
            let state = state
                .lock()
                .expect("compiled RHS state lock should not poison");
            state.slot_values.get(*slot).cloned()
        }
        TemplateNode::Local(slot) => local_expr(state, *slot),
        TemplateNode::Call { head, args } => {
            let head = instantiate_template(head, state)?;
            let mut output_args = Vec::with_capacity(args.len());
            for arg in args {
                let value = instantiate_template(arg, state)?;
                if let Some(items) = sequence_items(&value) {
                    output_args.extend(items.iter().cloned());
                } else {
                    output_args.push(value);
                }
            }
            Some(Expr::call(head, output_args))
        }
    }
}

fn local_expr(state: &RuntimeStateRef, slot: usize) -> Option<Expr> {
    let state = state
        .lock()
        .expect("compiled RHS state lock should not poison");
    match state.locals.get(slot)? {
        LocalRuntime::Module { fresh, .. } => Some(Expr::Atom(Atom::Symbol(*fresh))),
        LocalRuntime::With { value } => value.clone(),
    }
}

pub(super) fn set_local_value(state: &RuntimeStateRef, slot: usize, value: Expr) {
    let mut state = state
        .lock()
        .expect("compiled RHS state lock should not poison");
    match state
        .locals
        .get_mut(slot)
        .expect("compiled RHS local slot should exist")
    {
        LocalRuntime::Module {
            value: local_value, ..
        }
        | LocalRuntime::With { value: local_value } => *local_value = Some(value),
    }
}

pub(super) fn evaluate_instantiated_expr(
    session: &mut Session,
    state: RuntimeStateRef,
    expr: Expr,
) -> Result<Expr, EvalError> {
    if expr_is_self_evaluating_atom(&expr) {
        return Ok(expr);
    }

    let option_scope = {
        let state = state
            .lock()
            .expect("compiled RHS state lock should not poison");
        state.option_scope.clone()
    };
    let dynamic_depth = push_module_local_bindings(session, &state);
    let result = session.evaluate_with_option_value_scope(option_scope, expr);
    match result {
        Ok(value) => {
            truncate_dynamic_locals(session, dynamic_depth);
            Ok(value)
        }
        Err(EvalError::InputRequired { request }) => {
            session.push_suspension_frame(move |session, outcome| {
                truncate_dynamic_locals(session, dynamic_depth);
                outcome
            });
            Err(EvalError::InputRequired { request })
        }
        Err(error) => {
            truncate_dynamic_locals(session, dynamic_depth);
            Err(error)
        }
    }
}

fn push_module_local_bindings(session: &mut Session, state: &RuntimeStateRef) -> Option<usize> {
    let locals = {
        let state = state
            .lock()
            .expect("compiled RHS state lock should not poison");
        state
            .locals
            .iter()
            .filter_map(|local| match local {
                LocalRuntime::Module { fresh, value } => Some((*fresh, value.clone())),
                LocalRuntime::With { .. } => None,
            })
            .collect::<Vec<_>>()
    };

    let mut depth = None;
    for (symbol, value) in locals {
        let pushed_depth = session.push_dynamic_own_value_binding(symbol, value);
        depth.get_or_insert(pushed_depth);
    }
    depth
}

fn truncate_dynamic_locals(session: &mut Session, depth: Option<usize>) {
    if let Some(depth) = depth {
        session.truncate_dynamic_own_value_bindings(depth);
    }
}

pub(super) fn resume_compiled_if(
    session: &mut Session,
    state: RuntimeStateRef,
    condition: Expr,
    true_branch: CompiledClosure,
    false_branch: Option<CompiledClosure>,
    unknown_branch: Option<CompiledClosure>,
    residual_templates: Vec<TemplateNode>,
) -> Result<Expr, EvalError> {
    match truthiness_symbol(&condition) {
        Some(true) => true_branch(session, state),
        Some(false) => {
            if let Some(false_branch) = false_branch {
                false_branch(session, state)
            } else {
                Ok(system_expr("Null"))
            }
        }
        None => {
            if let Some(unknown_branch) = unknown_branch {
                return unknown_branch(session, state);
            }
            let mut args = Vec::with_capacity(residual_templates.len());
            args.push(condition);
            for template in residual_templates.iter().skip(1) {
                args.push(instantiate_template(template, &state).expect(
                    "compiled If residual templates should instantiate with current bindings",
                ));
            }
            Ok(Expr::call(system_expr("If"), args))
        }
    }
}

pub(super) fn resume_compiled_compound(
    session: &mut Session,
    items: Arc<Vec<CompiledClosure>>,
    state: RuntimeStateRef,
    index: usize,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        return Ok(system_expr("Null"));
    };
    let result = item(session, state.clone());
    let value = session.suspend_with_frame_lazy(result, || {
        let items = items.clone();
        let state = state.clone();
        move |session, value| {
            if index + 1 >= items.len() {
                Ok(value)
            } else {
                resume_compiled_compound(session, items, state, index + 1)
            }
        }
    })?;

    if index + 1 >= items.len() {
        Ok(value)
    } else {
        resume_compiled_compound(session, items, state, index + 1)
    }
}

pub(super) fn initialize_module_locals(
    session: &mut Session,
    state: &RuntimeStateRef,
    local_slots: &[(usize, Symbol)],
) {
    let mut state = state
        .lock()
        .expect("compiled RHS state lock should not poison");
    for (slot, symbol) in local_slots {
        state.locals[*slot] = LocalRuntime::Module {
            fresh: session.allocate_fresh_symbol(symbol),
            value: None,
        };
    }
}

pub(super) fn resume_module_initializers(
    session: &mut Session,
    initializers: Arc<Vec<(usize, CompiledClosure)>>,
    body: CompiledClosure,
    state: RuntimeStateRef,
    index: usize,
) -> Result<Expr, EvalError> {
    let Some((slot, init)) = initializers.get(index).cloned() else {
        return body(session, state);
    };
    let result = init(session, state.clone());
    let value = session.suspend_with_frame_lazy(result, || {
        let initializers = initializers.clone();
        let body = body.clone();
        let state = state.clone();
        move |session, value| {
            set_local_value(&state, slot, value);
            resume_module_initializers(session, initializers, body, state, index + 1)
        }
    })?;
    set_local_value(&state, slot, value);
    resume_module_initializers(session, initializers, body, state, index + 1)
}

pub(super) fn initialize_with_locals(
    state: &RuntimeStateRef,
    initializers: &Arc<Vec<(usize, CompiledClosure)>>,
) {
    let mut state = state
        .lock()
        .expect("compiled RHS state lock should not poison");
    for (slot, _) in initializers.iter() {
        state.locals[*slot] = LocalRuntime::With { value: None };
    }
}

pub(super) fn resume_with_initializers(
    session: &mut Session,
    initializers: Arc<Vec<(usize, CompiledClosure)>>,
    body: CompiledClosure,
    state: RuntimeStateRef,
    index: usize,
) -> Result<Expr, EvalError> {
    let Some((slot, init)) = initializers.get(index).cloned() else {
        return body(session, state);
    };
    let result = init(session, state.clone());
    let value = session.suspend_with_frame_lazy(result, || {
        let initializers = initializers.clone();
        let body = body.clone();
        let state = state.clone();
        move |session, value| {
            set_local_value(&state, slot, value);
            resume_with_initializers(session, initializers, body, state, index + 1)
        }
    })?;
    set_local_value(&state, slot, value);
    resume_with_initializers(session, initializers, body, state, index + 1)
}
