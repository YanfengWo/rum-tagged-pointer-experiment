use super::*;
use std::sync::{Arc, Mutex};

pub(super) type RuntimeStateRef = Arc<Mutex<RuntimeState>>;
pub(super) type CompiledClosure =
    Arc<dyn Fn(&mut Session, RuntimeStateRef) -> Result<Expr, EvalError> + Send + Sync + 'static>;

#[derive(Clone)]
pub(super) struct CodeObject {
    pub(super) source: Expr,
    pub(super) body: CompiledClosure,
    pub(super) source_template: Option<TemplateNode>,
    pub(super) slots: Vec<Symbol>,
    pub(super) const_symbols: Vec<Symbol>,
    pub(super) local_specs: Vec<LocalSpec>,
    pub(super) shape_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LocalKind {
    Module,
    With,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LocalSpec {
    pub(super) symbol: Symbol,
    pub(super) kind: LocalKind,
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeState {
    pub(super) slot_values: Vec<Expr>,
    pub(super) locals: Vec<LocalRuntime>,
    pub(super) option_scope: Option<OptionValueScope>,
}

#[derive(Clone, Debug)]
pub(super) enum LocalRuntime {
    Module { fresh: Symbol, value: Option<Expr> },
    With { value: Option<Expr> },
}

#[derive(Clone)]
pub(in crate::runtime) enum PreparedRhsTemplate {
    Substituted(Expr),
    Compiled(PreparedCompiledRhsTemplate),
}

#[derive(Clone)]
pub(in crate::runtime) struct PreparedCompiledRhsTemplate {
    pub(super) code: Arc<CodeObject>,
    pub(super) slot_values: Vec<Expr>,
}

#[derive(Clone)]
pub(super) enum TemplateNode {
    Const(SharedExpr),
    Binding(usize),
    Local(usize),
    Call {
        head: Box<TemplateNode>,
        args: Vec<TemplateNode>,
    },
}

#[derive(Clone, Copy)]
pub(super) struct LocalBinding {
    pub(super) symbol: Symbol,
    pub(super) slot: usize,
}
