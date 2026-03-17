use std::fmt;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

pub const GLOBAL_CONTEXT: &str = "Global`";

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    context: String,
    name: String,
}

impl Symbol {
    pub fn new(context: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            context: context.into(),
            name: name.into(),
        }
    }

    pub fn from_name(name: impl Into<String>) -> Self {
        Self::new(GLOBAL_CONTEXT, name)
    }

    pub fn context(&self) -> &str {
        &self.context
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn qualified_name(&self) -> String {
        if self.context == GLOBAL_CONTEXT {
            self.name.clone()
        } else {
            format!("{}{}", self.context, self.name)
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.qualified_name())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Atom {
    Integer(BigInt),
    String(String),
    Symbol(Symbol),
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
            Self::Symbol(symbol) => write!(f, "{symbol}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expr {
    Atom(Atom),
    Call { head: Box<Expr>, args: Vec<Expr> },
}

impl Expr {
    pub fn integer(value: BigInt) -> Self {
        Self::Atom(Atom::Integer(value))
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::Atom(Atom::String(value.into()))
    }

    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Atom(Atom::Symbol(Symbol::from_name(name)))
    }

    pub fn contextual_symbol(context: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Atom(Atom::Symbol(Symbol::new(context, name)))
    }

    pub fn call(head: Expr, args: Vec<Expr>) -> Self {
        Self::Call {
            head: Box::new(head),
            args,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atom(atom) => write!(f, "{atom}"),
            Self::Call { head, args } => {
                write!(f, "{head}[")?;

                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        f.write_str(", ")?;
                    }

                    write!(f, "{arg}")?;
                }

                f.write_str("]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_global_symbol_without_context_prefix() {
        let symbol = Symbol::from_name("x");
        assert_eq!(symbol.to_string(), "x");
    }

    #[test]
    fn displays_nested_call_expression() {
        let expr = Expr::call(
            Expr::symbol("f"),
            vec![Expr::integer(BigInt::from(42_u32)), Expr::symbol("x")],
        );

        assert_eq!(expr.to_string(), "f[42, x]");
    }
}
