use std::collections::BTreeMap;

use rum_core::{Atom, BigInt, Expr, Symbol};
use rum_syntax::{
    BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxError, SyntaxExpr, parse_expr,
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinSpec {
    pub name: &'static str,
    pub summary: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct Session {
    builtins: BTreeMap<String, BuiltinSpec>,
    definitions: BTreeMap<Symbol, SymbolDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalOutcome {
    pub value: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolInfo {
    pub qualified_name: String,
    pub summary: Option<String>,
    pub is_builtin: bool,
    pub has_own_value: bool,
    pub has_delayed_value: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolDefinition {
    pub own_value: Option<Expr>,
    pub delayed_own_value: Option<Expr>,
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error("invalid integer literal `{literal}`")]
    InvalidInteger { literal: String },
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_builtin(&mut self, spec: BuiltinSpec) {
        self.builtins.insert(spec.name.to_string(), spec);
    }

    pub fn inspect(&self, name: &str) -> Option<SymbolInfo> {
        let symbol = symbol_from_raw_name(name);
        let builtin = self.builtins.get(symbol.name());
        let definition = self.definitions.get(&symbol);

        if builtin.is_none() && definition.is_none() {
            return None;
        }

        Some(SymbolInfo {
            qualified_name: symbol.to_string(),
            summary: builtin.map(|builtin| builtin.summary.to_string()),
            is_builtin: builtin.is_some(),
            has_own_value: definition.and_then(|def| def.own_value.as_ref()).is_some(),
            has_delayed_value: definition
                .and_then(|def| def.delayed_own_value.as_ref())
                .is_some(),
        })
    }

    pub fn eval_text(&mut self, input: &str) -> Result<EvalOutcome, EvalError> {
        let syntax = parse_expr(input)?;
        let expr = lower_syntax_expr(&syntax)?;
        let value = self.evaluate(expr)?;
        Ok(EvalOutcome { value })
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => self.evaluate_symbol(symbol),
            Expr::Call { head, args } => self.evaluate_call(*head, args),
            other => Ok(other),
        }
    }

    fn evaluate_symbol(&mut self, symbol: Symbol) -> Result<Expr, EvalError> {
        let definition = self.definitions.get(&symbol).cloned();

        match definition {
            Some(SymbolDefinition {
                own_value: Some(value),
                ..
            }) => self.evaluate(value),
            Some(SymbolDefinition {
                delayed_own_value: Some(value),
                ..
            }) => self.evaluate(value),
            _ => Ok(Expr::Atom(Atom::Symbol(symbol))),
        }
    }

    fn evaluate_call(&mut self, head: Expr, args: Vec<Expr>) -> Result<Expr, EvalError> {
        if let Some(result) = self.try_evaluate_special_form(&head, args.as_slice())? {
            return Ok(result);
        }

        let evaluated_head = self.evaluate(head)?;
        let evaluated_args = args
            .into_iter()
            .map(|arg| self.evaluate(arg))
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(result) =
            self.try_evaluate_non_held_builtin(&evaluated_head, evaluated_args.as_slice())?
        {
            return Ok(result);
        }

        Ok(Expr::call(evaluated_head, evaluated_args))
    }

    fn try_evaluate_special_form(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        match symbol.name() {
            "Set" => self.eval_set(args).map(Some),
            "SetDelayed" => self.eval_set_delayed(args).map(Some),
            _ => Ok(None),
        }
    }

    fn try_evaluate_non_held_builtin(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        match symbol.name() {
            "Identity" if args.len() == 1 => self.evaluate(args[0].clone()).map(Some),
            _ => Ok(None),
        }
    }

    fn eval_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("Set"), args.to_vec()));
        }

        let lhs = args[0].clone();
        let rhs = self.evaluate(args[1].clone())?;

        if let Some(symbol) = expr_as_symbol(&lhs).cloned() {
            let definition = self.definitions.entry(symbol).or_default();
            definition.own_value = Some(rhs.clone());
            definition.delayed_own_value = None;
            Ok(rhs)
        } else {
            Ok(Expr::call(Expr::symbol("Set"), vec![lhs, rhs]))
        }
    }

    fn eval_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(Expr::call(Expr::symbol("SetDelayed"), args.to_vec()));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();

        if let Some(symbol) = expr_as_symbol(&lhs).cloned() {
            let definition = self.definitions.entry(symbol).or_default();
            definition.own_value = None;
            definition.delayed_own_value = Some(rhs);
            Ok(Expr::symbol("Null"))
        } else {
            Ok(Expr::call(Expr::symbol("SetDelayed"), vec![lhs, rhs]))
        }
    }
}

pub fn lower_syntax_expr(expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    match expr {
        SyntaxExpr::Integer(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            Ok(Expr::integer(bigint))
        }
        SyntaxExpr::String(value) => Ok(Expr::string(value.clone())),
        SyntaxExpr::Symbol(value) => Ok(lower_symbol(value)),
        SyntaxExpr::List(items) => {
            let lowered_items = items
                .iter()
                .map(lower_syntax_expr)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::call(Expr::symbol("List"), lowered_items))
        }
        SyntaxExpr::Slot(spec) => lower_slot(spec),
        SyntaxExpr::Blank { kind, head } => lower_blank(*kind, head.as_deref()),
        SyntaxExpr::Pattern { name, pattern } => Ok(Expr::call(
            Expr::symbol("Pattern"),
            vec![lower_syntax_expr(name)?, lower_syntax_expr(pattern)?],
        )),
        SyntaxExpr::Prefix { op, expr } => lower_prefix(*op, expr),
        SyntaxExpr::Binary { op, lhs, rhs } => lower_binary(*op, lhs, rhs),
        SyntaxExpr::Postfix { op, expr } => lower_postfix(*op, expr),
        SyntaxExpr::Call { head, args } => {
            let lowered_head = lower_syntax_expr(head)?;
            let lowered_args = args
                .iter()
                .map(lower_syntax_expr)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::call(lowered_head, lowered_args))
        }
    }
}

fn lower_slot(spec: &SlotSpec) -> Result<Expr, EvalError> {
    let arg = match spec {
        SlotSpec::Anonymous => Expr::integer(BigInt::from(1_u8)),
        SlotSpec::Number(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            Expr::integer(bigint)
        }
        SlotSpec::Named(value) => Expr::string(value.clone()),
    };

    Ok(Expr::call(Expr::symbol("Slot"), vec![arg]))
}

fn lower_blank(kind: BlankKind, head: Option<&SyntaxExpr>) -> Result<Expr, EvalError> {
    let head_symbol = match kind {
        BlankKind::Blank => "Blank",
        BlankKind::BlankSequence => "BlankSequence",
        BlankKind::BlankNullSequence => "BlankNullSequence",
    };

    let args = match head {
        Some(head) => vec![lower_syntax_expr(head)?],
        None => Vec::new(),
    };

    Ok(Expr::call(Expr::symbol(head_symbol), args))
}

fn lower_prefix(op: PrefixOp, expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    let lowered = lower_syntax_expr(expr)?;

    match op {
        PrefixOp::Plus => Ok(lowered),
        PrefixOp::Minus => Ok(Expr::call(
            Expr::symbol("Times"),
            vec![Expr::integer(BigInt::from(-1)), lowered],
        )),
    }
}

fn lower_binary(op: BinaryOp, lhs: &SyntaxExpr, rhs: &SyntaxExpr) -> Result<Expr, EvalError> {
    let lowered_lhs = lower_syntax_expr(lhs)?;
    let lowered_rhs = lower_syntax_expr(rhs)?;

    match op {
        BinaryOp::Set => Ok(Expr::call(
            Expr::symbol("Set"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::SetDelayed => Ok(Expr::call(
            Expr::symbol("SetDelayed"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceAll => Ok(Expr::call(
            Expr::symbol("ReplaceAll"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Rule => Ok(Expr::call(
            Expr::symbol("Rule"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::RuleDelayed => Ok(Expr::call(
            Expr::symbol("RuleDelayed"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Condition => Ok(Expr::call(
            Expr::symbol("Condition"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Add => Ok(Expr::call(
            Expr::symbol("Plus"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Subtract => Ok(Expr::call(
            Expr::symbol("Plus"),
            vec![
                lowered_lhs,
                Expr::call(
                    Expr::symbol("Times"),
                    vec![Expr::integer(BigInt::from(-1)), lowered_rhs],
                ),
            ],
        )),
        BinaryOp::Times => Ok(Expr::call(
            Expr::symbol("Times"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Divide => Ok(Expr::call(
            Expr::symbol("Times"),
            vec![
                lowered_lhs,
                Expr::call(
                    Expr::symbol("Power"),
                    vec![lowered_rhs, Expr::integer(BigInt::from(-1))],
                ),
            ],
        )),
        BinaryOp::Power => Ok(Expr::call(
            Expr::symbol("Power"),
            vec![lowered_lhs, lowered_rhs],
        )),
    }
}

fn lower_postfix(op: PostfixOp, expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    let lowered = lower_syntax_expr(expr)?;

    match op {
        PostfixOp::Function => Ok(Expr::call(Expr::symbol("Function"), vec![lowered])),
    }
}

fn lower_symbol(raw: &str) -> Expr {
    Expr::Atom(Atom::Symbol(symbol_from_raw_name(raw)))
}

fn symbol_from_raw_name(raw: &str) -> Symbol {
    if let Some(split_at) = raw.rfind('`') {
        let context = &raw[..=split_at];
        let name = &raw[split_at + 1..];
        Symbol::new(context, name)
    } else {
        Symbol::from_name(raw)
    }
}

fn expr_as_symbol(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_big_integer_into_core_expr() {
        let lowered = lower_syntax_expr(&SyntaxExpr::Integer(
            "123456789012345678901234567890".to_string(),
        ))
        .unwrap();

        assert_eq!(lowered.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn evaluates_identity_call() {
        let mut session = Session::new();
        let outcome = session
            .eval_text("Identity[123456789012345678901234567890]")
            .unwrap();

        assert_eq!(outcome.value.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn set_assigns_simple_symbol_and_lookup_works() {
        let mut session = Session::new();

        let assigned = session.eval_text("x = 42").unwrap();
        assert_eq!(assigned.value.to_string(), "42");

        let looked_up = session.eval_text("x").unwrap();
        assert_eq!(looked_up.value.to_string(), "42");

        let info = session.inspect("x").unwrap();
        assert!(info.has_own_value);
        assert!(!info.has_delayed_value);
    }

    #[test]
    fn setdelayed_stores_rhs_without_immediate_evaluation() {
        let mut session = Session::new();

        assert_eq!(
            session.eval_text("y := x").unwrap().value.to_string(),
            "Null"
        );
        assert_eq!(session.eval_text("x = 9").unwrap().value.to_string(), "9");
        assert_eq!(session.eval_text("y").unwrap().value.to_string(), "9");

        let info = session.inspect("y").unwrap();
        assert!(!info.has_own_value);
        assert!(info.has_delayed_value);
    }

    #[test]
    fn set_with_non_symbol_lhs_remains_unevaluated_for_now() {
        let mut session = Session::new();
        let outcome = session.eval_text("f[x] = 1").unwrap();

        assert_eq!(outcome.value.to_string(), "Set[f[x], 1]");
    }

    #[test]
    fn lowers_named_pattern_assignment() {
        let syntax = parse_expr("x_Integer := f[x]").unwrap();
        let lowered = lower_syntax_expr(&syntax).unwrap();

        assert_eq!(
            lowered.to_string(),
            "SetDelayed[Pattern[x, Blank[Integer]], f[x]]"
        );
    }

    #[test]
    fn lowers_slots_and_pure_functions() {
        let syntax = parse_expr("#1 + #2 &").unwrap();
        let lowered = lower_syntax_expr(&syntax).unwrap();

        assert_eq!(lowered.to_string(), "Function[Plus[Slot[1], Slot[2]]]");
    }
}
