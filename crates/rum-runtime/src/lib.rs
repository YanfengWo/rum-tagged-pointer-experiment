use std::collections::BTreeMap;

use rum_core::{BigInt, Expr, Symbol};
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
        let builtin = self.builtins.get(name)?;

        Some(SymbolInfo {
            qualified_name: name.to_string(),
            summary: Some(builtin.summary.to_string()),
            is_builtin: true,
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
            Expr::Call { head, mut args } => {
                if let Expr::Atom(rum_core::Atom::Symbol(symbol)) = head.as_ref() {
                    if symbol.name() == "Identity" && args.len() == 1 {
                        return self.evaluate(args.remove(0));
                    }
                }

                Ok(Expr::Call { head, args })
            }
            other => Ok(other),
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
    if let Some(split_at) = raw.rfind('`') {
        let context = &raw[..=split_at];
        let name = &raw[split_at + 1..];
        Expr::Atom(rum_core::Atom::Symbol(Symbol::new(context, name)))
    } else {
        Expr::symbol(raw)
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
