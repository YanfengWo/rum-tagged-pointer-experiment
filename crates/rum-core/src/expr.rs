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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Rational {
    numerator: BigInt,
    denominator: BigInt,
}

impl Rational {
    pub fn new(numerator: BigInt, denominator: BigInt) -> Option<Self> {
        if denominator == BigInt::from(0_u8) {
            return None;
        }

        if numerator == BigInt::from(0_u8) {
            return Some(Self {
                numerator,
                denominator: BigInt::from(1_u8),
            });
        }

        let mut numerator = numerator;
        let mut denominator = denominator;

        if denominator < BigInt::from(0_u8) {
            numerator = -numerator;
            denominator = -denominator;
        }

        let divisor = bigint_gcd(bigint_abs(&numerator), denominator.clone());

        Some(Self {
            numerator: numerator / divisor.clone(),
            denominator: denominator / divisor,
        })
    }

    pub fn from_integer(value: BigInt) -> Self {
        Self {
            numerator: value,
            denominator: BigInt::from(1_u8),
        }
    }

    pub fn zero() -> Self {
        Self::from_integer(BigInt::from(0_u8))
    }

    pub fn one() -> Self {
        Self::from_integer(BigInt::from(1_u8))
    }

    pub fn numerator(&self) -> &BigInt {
        &self.numerator
    }

    pub fn denominator(&self) -> &BigInt {
        &self.denominator
    }

    pub fn is_zero(&self) -> bool {
        self.numerator == BigInt::from(0_u8)
    }

    pub fn is_one(&self) -> bool {
        self.numerator == BigInt::from(1_u8) && self.denominator == BigInt::from(1_u8)
    }

    pub fn is_integer(&self) -> bool {
        self.denominator == BigInt::from(1_u8)
    }

    pub fn as_integer(&self) -> Option<&BigInt> {
        self.is_integer().then_some(&self.numerator)
    }

    pub fn negated(&self) -> Self {
        Self {
            numerator: -self.numerator.clone(),
            denominator: self.denominator.clone(),
        }
    }

    pub fn reciprocal(&self) -> Option<Self> {
        Self::new(self.denominator.clone(), self.numerator.clone())
    }

    pub fn add(&self, rhs: &Self) -> Self {
        Self::new(
            &self.numerator * &rhs.denominator + &rhs.numerator * &self.denominator,
            &self.denominator * &rhs.denominator,
        )
        .expect("adding normalized rationals should keep a non-zero denominator")
    }

    pub fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.negated())
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        Self::new(
            &self.numerator * &rhs.numerator,
            &self.denominator * &rhs.denominator,
        )
        .expect("multiplying normalized rationals should keep a non-zero denominator")
    }

    pub fn div(&self, rhs: &Self) -> Option<Self> {
        Some(self.mul(&rhs.reciprocal()?))
    }

    pub fn pow(&self, exponent: &BigInt) -> Option<Self> {
        if exponent == &BigInt::from(0_u8) {
            return (!self.is_zero()).then(Self::one);
        }

        if exponent > &BigInt::from(0_u8) {
            return Some(Self::new(
                pow_bigint(&self.numerator, exponent),
                pow_bigint(&self.denominator, exponent),
            )?);
        }

        let positive = -exponent.clone();
        Some(self.reciprocal()?.pow(&positive)?)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Atom {
    Integer(BigInt),
    Rational(Rational),
    String(String),
    Symbol(Symbol),
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Rational(value) => write!(f, "{value}"),
            Self::String(value) => write_escaped_string(f, value),
            Self::Symbol(symbol) => write!(f, "{symbol}"),
        }
    }
}

fn write_escaped_string(f: &mut fmt::Formatter<'_>, value: &str) -> fmt::Result {
    f.write_str("\"")?;

    for ch in value.chars() {
        match ch {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            other if other.is_control() => write!(f, "\\u{{{:X}}}", other as u32)?,
            other => write!(f, "{other}")?,
        }
    }

    f.write_str("\"")
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

    pub fn rational(value: Rational) -> Self {
        if let Some(integer) = value.as_integer() {
            Self::integer(integer.clone())
        } else {
            Self::Atom(Atom::Rational(value))
        }
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

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_u8) {
        -value.clone()
    } else {
        value.clone()
    }
}

fn bigint_gcd(mut lhs: BigInt, mut rhs: BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);

    while rhs != zero {
        let remainder = &lhs % &rhs;
        lhs = rhs;
        rhs = remainder;
    }

    bigint_abs(&lhs)
}

fn pow_bigint(base: &BigInt, exponent: &BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);
    let one = BigInt::from(1_u8);
    let two = BigInt::from(2_u8);
    let mut result = BigInt::from(1_u8);
    let mut factor = base.clone();
    let mut remaining = exponent.clone();

    while remaining > zero {
        if &remaining % &two == one {
            result *= &factor;
        }

        remaining /= &two;

        if remaining > zero {
            factor = &factor * &factor;
        }
    }

    result
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
    fn normalizes_rational_atoms() {
        let rational = Rational::new(BigInt::from(6_i32), BigInt::from(-8_i32)).unwrap();

        assert_eq!(rational.numerator().to_string(), "-3");
        assert_eq!(rational.denominator().to_string(), "4");
        assert_eq!(Expr::rational(rational).to_string(), "-3/4");
    }

    #[test]
    fn displays_message_name_shorthand() {
        let expr = Expr::call(
            Expr::system_symbol("MessageName"),
            vec![Expr::symbol("f"), Expr::string("argx")],
        );

        assert_eq!(expr.to_string(), "f::argx");
    }

    #[test]
    fn displays_string_atoms_with_escaped_special_characters() {
        let expr = Expr::string("he said \"hi\"\n\\");
        assert_eq!(expr.to_string(), "\"he said \\\"hi\\\"\\n\\\\\"");
    }
}
