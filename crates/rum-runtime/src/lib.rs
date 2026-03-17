use std::collections::BTreeMap;

use rum_core::{BigInt, Expr, Symbol};
use rum_syntax::{SyntaxError, SyntaxExpr, parse_expr};
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
}
