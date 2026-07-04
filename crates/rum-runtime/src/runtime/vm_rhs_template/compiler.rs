use super::executor::{
    evaluate_instantiated_expr, initialize_module_locals, initialize_with_locals,
    instantiate_template, resume_compiled_compound, resume_compiled_if, resume_module_initializers,
    resume_with_initializers, set_local_value,
};
use super::model::*;
use super::safety::{
    duplicate_scoped_symbols, leaf_contains_uncompiled_scope, locals_are_safe_in_compiled_expr,
};
use super::*;
use std::sync::Arc;
#[cfg(debug_assertions)]
use std::sync::Mutex;

pub(super) fn compile_body(expr: &Expr, bindings: &SymbolExprMap) -> Option<CodeObject> {
    let mut compiler = ClosureBodyCompiler::new(bindings);
    let body = compiler.compile_eval(expr)?;
    let source_template = if compiler.local_specs.is_empty() {
        Some(compiler.compile_template(expr)?)
    } else {
        None
    };
    Some(CodeObject {
        source: expr.clone(),
        body,
        source_template,
        slots: compiler.slots,
        const_symbols: compiler.const_symbols,
        local_specs: compiler.local_specs,
        shape_count: compiler.shape_count,
    })
}

#[cfg(debug_assertions)]
pub(super) fn debug_assert_template_matches_tree_walk(
    code: &CodeObject,
    slot_values: &[Expr],
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
) {
    let Some(source_template) = &code.source_template else {
        return;
    };
    let state = Arc::new(Mutex::new(RuntimeState {
        slot_values: slot_values.to_vec(),
        locals: Vec::new(),
        option_scope: None,
    }));
    let result = instantiate_template(source_template, &state)
        .expect("compiled source template should instantiate with prepared bindings");
    let expected = substitute_rule_bindings_for_rhs(expr, bindings, may_splice_without_bindings);
    debug_assert_eq!(
        result, expected,
        "RHS closure template diverged from tree substitution"
    );
}

struct ClosureBodyCompiler<'a> {
    bindings: &'a SymbolExprMap,
    slots: Vec<Symbol>,
    const_symbols: Vec<Symbol>,
    local_specs: Vec<LocalSpec>,
    locals: Vec<LocalBinding>,
    shape_count: usize,
}

impl<'a> ClosureBodyCompiler<'a> {
    fn new(bindings: &'a SymbolExprMap) -> Self {
        Self {
            bindings,
            slots: Vec::new(),
            const_symbols: Vec::new(),
            local_specs: Vec::new(),
            locals: Vec::new(),
            shape_count: 0,
        }
    }

    fn compile_eval(&mut self, expr: &Expr) -> Option<CompiledClosure> {
        self.shape_count += 1;
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "If") => self.compile_if(args),
            Expr::Call { head, args } if is_system_symbol_name(head, "CompoundExpression") => {
                self.compile_compound(args)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "Return") => {
                self.compile_return(args)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "Set") => {
                self.compile_set(args).or_else(|| self.compile_leaf(expr))
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Module") && args.len() == 2 =>
            {
                self.compile_module(args)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
                self.compile_with(args)
            }
            Expr::Call { head, .. } if is_system_symbol_name(head, "Function") => None,
            _ => self.compile_leaf(expr),
        }
    }

    fn compile_if(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        if !(2..=4).contains(&args.len()) {
            return None;
        }
        let condition = self.compile_eval(&args[0])?;
        let true_branch = self.compile_eval(&args[1])?;
        let false_branch = if args.len() >= 3 {
            Some(self.compile_eval(&args[2])?)
        } else {
            None
        };
        let unknown_branch = if args.len() == 4 {
            Some(self.compile_eval(&args[3])?)
        } else {
            None
        };
        let residual_templates = args
            .iter()
            .map(|arg| self.compile_template(arg))
            .collect::<Option<Vec<_>>>()?;

        Some(Arc::new(move |session, state| {
            let result = condition(session, state.clone());
            let value = session.suspend_with_frame_lazy(result, || {
                let true_branch = true_branch.clone();
                let false_branch = false_branch.clone();
                let unknown_branch = unknown_branch.clone();
                let residual_templates = residual_templates.clone();
                let state = state.clone();
                move |session, condition| {
                    resume_compiled_if(
                        session,
                        state,
                        condition,
                        true_branch,
                        false_branch,
                        unknown_branch,
                        residual_templates,
                    )
                }
            })?;
            resume_compiled_if(
                session,
                state,
                value,
                true_branch.clone(),
                false_branch.clone(),
                unknown_branch.clone(),
                residual_templates.clone(),
            )
        }))
    }

    fn compile_compound(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        let items = args
            .iter()
            .map(|arg| self.compile_eval(arg))
            .collect::<Option<Vec<_>>>()?;
        let items = Arc::new(items);
        Some(Arc::new(move |session, state| {
            resume_compiled_compound(session, items.clone(), state, 0)
        }))
    }

    fn compile_return(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        match args.as_slice() {
            [] => Some(Arc::new(|_, _| {
                Err(EvalError::Return {
                    value: system_expr("Null"),
                })
            })),
            [value] => {
                let value = self.compile_eval(value)?;
                Some(Arc::new(move |session, state| {
                    let result = value(session, state.clone());
                    let value = session.suspend_with_frame_lazy(result, || {
                        move |_, value| Err(EvalError::Return { value })
                    })?;
                    Err(EvalError::Return { value })
                }))
            }
            _ => None,
        }
    }

    fn compile_set(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        let [lhs, rhs] = args.as_slice() else {
            return None;
        };
        let symbol = expr_as_symbol(lhs)?;
        let local = self.visible_local(*symbol)?;
        let rhs = self.compile_eval(rhs)?;
        Some(Arc::new(move |session, state| {
            let result = rhs(session, state.clone());
            let value = session.suspend_with_frame_lazy(result, || {
                let state = state.clone();
                move |_, value| {
                    set_local_value(&state, local.slot, value.clone());
                    Ok(value)
                }
            })?;
            set_local_value(&state, local.slot, value.clone());
            Ok(value)
        }))
    }

    fn compile_module(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        let items = parse_module_binding_list(&args[0])?;
        if items.is_empty() || duplicate_scoped_symbols(items.iter().map(|(symbol, _)| *symbol)) {
            return None;
        }
        let local_symbols = items.iter().map(|(symbol, _)| *symbol).collect::<Vec<_>>();
        if !locals_are_safe_in_compiled_expr(&args[1], &local_symbols) {
            return None;
        }

        let local_base = self.locals.len();
        let mut local_slots = Vec::with_capacity(items.len());
        for (symbol, _) in &items {
            let slot = self.push_local(*symbol, LocalKind::Module);
            local_slots.push((slot, *symbol));
        }

        let mut initializers = Vec::new();
        for (index, (_, init)) in items.iter().enumerate() {
            if let Some(init) = init {
                initializers.push((local_slots[index].0, self.compile_eval(init)?));
            }
        }
        let body = self.compile_eval(&args[1])?;
        self.locals.truncate(local_base);

        let initializers = Arc::new(initializers);
        Some(Arc::new(move |session, state| {
            initialize_module_locals(session, &state, &local_slots);
            resume_module_initializers(session, initializers.clone(), body.clone(), state, 0)
        }))
    }

    fn compile_with(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        let items = parse_with_binding_list(&args[0])?;
        if items.is_empty() || duplicate_scoped_symbols(items.iter().map(|(symbol, _)| *symbol)) {
            return None;
        }

        let local_base = self.locals.len();
        let mut initializers = Vec::with_capacity(items.len());
        for (symbol, rhs) in &items {
            let rhs = self.compile_eval(rhs)?;
            let slot = self.push_local(*symbol, LocalKind::With);
            initializers.push((slot, rhs));
        }
        let body = self.compile_eval(&args[1])?;
        self.locals.truncate(local_base);

        let initializers = Arc::new(initializers);
        Some(Arc::new(move |session, state| {
            initialize_with_locals(&state, &initializers);
            resume_with_initializers(session, initializers.clone(), body.clone(), state, 0)
        }))
    }

    fn compile_leaf(&mut self, expr: &Expr) -> Option<CompiledClosure> {
        if leaf_contains_uncompiled_scope(expr) {
            return None;
        }
        let template = self.compile_template(expr)?;
        Some(Arc::new(move |session, state| {
            let expr = instantiate_template(&template, &state)
                .expect("compiled leaf template should instantiate with prepared bindings");
            evaluate_instantiated_expr(session, state, expr)
        }))
    }

    fn compile_template(&mut self, expr: &Expr) -> Option<TemplateNode> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) if self.visible_local(*symbol).is_some() => {
                Some(TemplateNode::Local(
                    self.visible_local(*symbol)
                        .expect("checked local should remain visible")
                        .slot,
                ))
            }
            Expr::Atom(Atom::Symbol(symbol)) if self.bindings.contains_key(symbol) => {
                Some(TemplateNode::Binding(self.binding_slot(*symbol)))
            }
            Expr::Atom(Atom::Symbol(symbol)) => {
                self.record_const_symbol(*symbol);
                Some(TemplateNode::Const(SharedExpr::from(expr.clone())))
            }
            Expr::Atom(_) => Some(TemplateNode::Const(SharedExpr::from(expr.clone()))),
            Expr::Call { head, args } => {
                let head = Box::new(self.compile_template(head)?);
                let args = args
                    .iter()
                    .map(|arg| self.compile_template(arg))
                    .collect::<Option<Vec<_>>>()?;
                Some(TemplateNode::Call { head, args })
            }
        }
    }

    fn push_local(&mut self, symbol: Symbol, kind: LocalKind) -> usize {
        let slot = self.local_specs.len();
        self.local_specs.push(LocalSpec { symbol, kind });
        self.locals.push(LocalBinding { symbol, slot });
        slot
    }

    fn visible_local(&self, symbol: Symbol) -> Option<LocalBinding> {
        self.locals
            .iter()
            .rev()
            .find(|local| local.symbol == symbol)
            .copied()
    }

    fn binding_slot(&mut self, symbol: Symbol) -> usize {
        if let Some(index) = self.slots.iter().position(|slot| *slot == symbol) {
            index
        } else {
            let index = self.slots.len();
            self.slots.push(symbol);
            index
        }
    }

    fn record_const_symbol(&mut self, symbol: Symbol) {
        if !self.const_symbols.contains(&symbol) {
            self.const_symbols.push(symbol);
        }
    }
}

pub(super) fn code_compatible_with_bindings(code: &CodeObject, bindings: &SymbolExprMap) -> bool {
    code.slots
        .iter()
        .all(|symbol| bindings.contains_key(symbol))
        && code
            .const_symbols
            .iter()
            .all(|symbol| !bindings.contains_key(symbol))
}

pub(super) fn slot_values_for_code(
    code: &CodeObject,
    bindings: &SymbolExprMap,
) -> Option<Vec<Expr>> {
    if code
        .const_symbols
        .iter()
        .any(|symbol| bindings.contains_key(symbol))
    {
        return None;
    }

    let mut slot_values = Vec::with_capacity(code.slots.len());
    for symbol in &code.slots {
        slot_values.push(bindings.get(symbol)?.clone());
    }
    Some(slot_values)
}
