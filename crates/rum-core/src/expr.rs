use std::fmt;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

pub const GLOBAL_CONTEXT: &str = "Global`";
pub const SYSTEM_CONTEXT: &str = "System`";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    pub fn system(name: impl Into<String>) -> Self {
        Self::new(SYSTEM_CONTEXT, name)
    }

    pub fn context(&self) -> &str {
        &self.context
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fully_qualified_name(&self) -> String {
        format!("{}{}", self.context, self.name)
    }

    pub fn qualified_name(&self) -> String {
        if self.context == GLOBAL_CONTEXT || self.context == SYSTEM_CONTEXT {
            self.name.clone()
        } else {
            self.fully_qualified_name()
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

    pub fn system_symbol(name: impl Into<String>) -> Self {
        Self::Atom(Atom::Symbol(Symbol::system(name)))
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
                if let Some(rendered) = format_message_name_call(head, args) {
                    return f.write_str(&rendered);
                }

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

fn format_message_name_call(head: &Expr, args: &[Expr]) -> Option<String> {
    let Expr::Atom(Atom::Symbol(symbol)) = head else {
        return None;
    };

    if symbol.context() != SYSTEM_CONTEXT || symbol.name() != "MessageName" || args.len() != 2 {
        return None;
    }

    let tag = match &args[1] {
        Expr::Atom(Atom::String(tag)) => tag.clone(),
        Expr::Atom(Atom::Symbol(tag)) => tag.name().to_string(),
        _ => return None,
    };

    Some(format!("{}::{}", args[0], tag))
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
    fn displays_system_symbol_without_context_prefix() {
        let symbol = Symbol::system("If");
        assert_eq!(symbol.to_string(), "If");
        assert_eq!(symbol.fully_qualified_name(), "System`If");
    }

    #[test]
    fn displays_nested_call_expression() {
        let expr = Expr::call(
            Expr::symbol("f"),
            vec![Expr::integer(BigInt::from(42_u32)), Expr::symbol("x")],
        );

        assert_eq!(expr.to_string(), "f[42, x]");
    }

    #[test]
    fn displays_message_name_shorthand() {
        let expr = Expr::call(
            Expr::system_symbol("MessageName"),
            vec![Expr::symbol("f"), Expr::string("argx")],
        );

        assert_eq!(expr.to_string(), "f::argx");
    }
}
