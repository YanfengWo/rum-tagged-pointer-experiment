use std::fmt;

use crate::Span;

#[derive(Clone, Debug)]
pub struct SyntaxExpr {
    pub kind: SyntaxExprKind,
    pub span: Span,
}

impl SyntaxExpr {
    pub fn new(kind: SyntaxExprKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn integer(value: impl Into<String>, span: Span) -> Self {
        Self::new(SyntaxExprKind::Integer(value.into()), span)
    }

    pub fn string(value: impl Into<String>, span: Span) -> Self {
        Self::new(SyntaxExprKind::String(value.into()), span)
    }

    pub fn symbol(value: impl Into<String>, span: Span) -> Self {
        Self::new(SyntaxExprKind::Symbol(value.into()), span)
    }

    pub fn list(items: Vec<Self>, span: Span) -> Self {
        Self::new(SyntaxExprKind::List(items), span)
    }

    pub fn association(items: Vec<Self>, span: Span) -> Self {
        Self::new(SyntaxExprKind::Association(items), span)
    }

    pub fn slot(spec: SlotSpec, span: Span) -> Self {
        Self::new(SyntaxExprKind::Slot(spec), span)
    }

    pub fn blank(kind: BlankKind, head: Option<Self>, span: Span) -> Self {
        Self::new(
            SyntaxExprKind::Blank {
                kind,
                head: head.map(Box::new),
            },
            span,
        )
    }

    pub fn pattern(name: Self, pattern: Self, span: Span) -> Self {
        Self::new(
            SyntaxExprKind::Pattern {
                name: Box::new(name),
                pattern: Box::new(pattern),
            },
            span,
        )
    }

    pub fn prefix(op: PrefixOp, expr: Self, span: Span) -> Self {
        Self::new(
            SyntaxExprKind::Prefix {
                op,
                expr: Box::new(expr),
            },
            span,
        )
    }

    pub fn binary(op: BinaryOp, lhs: Self, rhs: Self, span: Span) -> Self {
        Self::new(
            SyntaxExprKind::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            span,
        )
    }

    pub fn postfix(op: PostfixOp, expr: Self, span: Span) -> Self {
        Self::new(
            SyntaxExprKind::Postfix {
                op,
                expr: Box::new(expr),
            },
            span,
        )
    }

    pub fn call(head: Self, args: Vec<Self>, span: Span) -> Self {
        Self::new(
            SyntaxExprKind::Call {
                head: Box::new(head),
                args,
            },
            span,
        )
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    fn precedence(&self) -> u8 {
        match &self.kind {
            SyntaxExprKind::Postfix { op, .. } => op.precedence(),
            SyntaxExprKind::Binary { op, .. } => op.precedence(),
            SyntaxExprKind::Prefix { op, .. } => op.precedence(),
            SyntaxExprKind::Pattern { .. } => 85,
            SyntaxExprKind::Blank { .. } => 85,
            SyntaxExprKind::Call { .. } => 90,
            SyntaxExprKind::Integer(_)
            | SyntaxExprKind::String(_)
            | SyntaxExprKind::Symbol(_)
            | SyntaxExprKind::List(_)
            | SyntaxExprKind::Association(_)
            | SyntaxExprKind::Slot(_) => 95,
        }
    }
}

impl PartialEq for SyntaxExpr {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Eq for SyntaxExpr {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxExprKind {
    Integer(String),
    String(String),
    Symbol(String),
    List(Vec<SyntaxExpr>),
    Association(Vec<SyntaxExpr>),
    Slot(SlotSpec),
    Blank {
        kind: BlankKind,
        head: Option<Box<SyntaxExpr>>,
    },
    Pattern {
        name: Box<SyntaxExpr>,
        pattern: Box<SyntaxExpr>,
    },
    Prefix {
        op: PrefixOp,
        expr: Box<SyntaxExpr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<SyntaxExpr>,
        rhs: Box<SyntaxExpr>,
    },
    Postfix {
        op: PostfixOp,
        expr: Box<SyntaxExpr>,
    },
    Call {
        head: Box<SyntaxExpr>,
        args: Vec<SyntaxExpr>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlotSpec {
    Anonymous,
    Number(String),
    Named(String),
    Sequence,
    SequenceNumber(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlankKind {
    Blank,
    BlankSequence,
    BlankNullSequence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefixOp {
    Plus,
    Minus,
    Not,
    PreIncrement,
    PreDecrement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    CompoundExpression,
    MessageName,
    TagSet,
    Set,
    UpSet,
    AddTo,
    SubtractFrom,
    TimesBy,
    DivideBy,
    ApplyTo,
    SetDelayed,
    UpSetDelayed,
    ReplaceAll,
    ReplaceRepeated,
    Rule,
    RuleDelayed,
    Condition,
    Or,
    And,
    SameQ,
    UnsameQ,
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Pattern,
    Alternatives,
    PatternTest,
    StringExpression,
    StringJoin,
    Apply,
    PostfixApply,
    Map,
    MapAll,
    ApplyHead,
    MapApply,
    Composition,
    RightComposition,
    Add,
    Subtract,
    Times,
    NonCommutativeMultiply,
    Divide,
    Power,
    Function,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostfixOp {
    Function,
    Increment,
    Decrement,
    Factorial,
    Factorial2,
    Derivative(u32),
    Repeated,
    RepeatedNull,
    Unset,
}

impl BlankKind {
    pub fn token(self) -> &'static str {
        match self {
            Self::Blank => "_",
            Self::BlankSequence => "__",
            Self::BlankNullSequence => "___",
        }
    }
}

impl PrefixOp {
    pub fn token(self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Not => "!",
            Self::PreIncrement => "++",
            Self::PreDecrement => "--",
        }
    }

    pub(crate) fn precedence(self) -> u8 {
        match self {
            Self::Plus | Self::Minus => 65,
            Self::Not => 65,
            Self::PreIncrement | Self::PreDecrement => 80,
        }
    }
}

impl BinaryOp {
    pub fn token(self) -> &'static str {
        match self {
            Self::CompoundExpression => ";",
            Self::MessageName => "::",
            Self::TagSet => "/:",
            Self::Set => "=",
            Self::UpSet => "^=",
            Self::AddTo => "+=",
            Self::SubtractFrom => "-=",
            Self::TimesBy => "*=",
            Self::DivideBy => "/=",
            Self::ApplyTo => "//=",
            Self::SetDelayed => ":=",
            Self::UpSetDelayed => "^:=",
            Self::ReplaceAll => "/.",
            Self::ReplaceRepeated => "//.",
            Self::Rule => "->",
            Self::RuleDelayed => ":>",
            Self::Condition => "/;",
            Self::Or => "||",
            Self::And => "&&",
            Self::SameQ => "===",
            Self::UnsameQ => "=!=",
            Self::Equal => "==",
            Self::Unequal => "!=",
            Self::Less => "<",
            Self::Greater => ">",
            Self::LessEqual => "<=",
            Self::GreaterEqual => ">=",
            Self::Pattern => ":",
            Self::Alternatives => "|",
            Self::PatternTest => "?",
            Self::StringExpression => "~~",
            Self::StringJoin => "<>",
            Self::Apply => "@",
            Self::PostfixApply => "//",
            Self::Map => "/@",
            Self::MapAll => "//@",
            Self::ApplyHead => "@@",
            Self::MapApply => "@@@",
            Self::Composition => "@*",
            Self::RightComposition => "/*",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Times => "*",
            Self::NonCommutativeMultiply => "**",
            Self::Divide => "/",
            Self::Power => "^",
            Self::Function => "|->",
        }
    }

    pub(crate) fn precedence(self) -> u8 {
        match self {
            Self::CompoundExpression => 3,
            Self::MessageName => 75,
            Self::TagSet => 5,
            Self::Set | Self::UpSet | Self::SetDelayed | Self::UpSetDelayed => 10,
            Self::AddTo | Self::SubtractFrom | Self::TimesBy | Self::DivideBy | Self::ApplyTo => 10,
            Self::Function => 14,
            Self::ReplaceAll | Self::ReplaceRepeated => 20,
            Self::Rule | Self::RuleDelayed => 30,
            Self::Condition => 40,
            Self::Or => 41,
            Self::And => 42,
            Self::SameQ
            | Self::UnsameQ
            | Self::Equal
            | Self::Unequal
            | Self::Less
            | Self::Greater
            | Self::LessEqual
            | Self::GreaterEqual => 45,
            Self::Pattern => 80,
            Self::Alternatives => 82,
            Self::PatternTest => 85,
            Self::StringExpression => 35,
            Self::StringJoin => 55,
            Self::PostfixApply => 15,
            Self::Apply | Self::Map | Self::MapAll | Self::ApplyHead | Self::MapApply => 72,
            Self::Composition | Self::RightComposition => 73,
            Self::Add | Self::Subtract => 50,
            Self::Times | Self::Divide => 60,
            Self::NonCommutativeMultiply => 65,
            Self::Power => 70,
        }
    }

    pub(crate) fn is_right_associative(self) -> bool {
        matches!(
            self,
            Self::CompoundExpression
                | Self::TagSet
                | Self::Set
                | Self::UpSet
                | Self::AddTo
                | Self::SubtractFrom
                | Self::TimesBy
                | Self::DivideBy
                | Self::ApplyTo
                | Self::SetDelayed
                | Self::UpSetDelayed
                | Self::Rule
                | Self::RuleDelayed
                | Self::Condition
                | Self::Or
                | Self::And
                | Self::Apply
                | Self::Map
                | Self::MapAll
                | Self::ApplyHead
                | Self::MapApply
                | Self::Composition
                | Self::Power
                | Self::Function
        )
    }
}

impl PostfixOp {
    pub fn token(self) -> &'static str {
        match self {
            Self::Function => "&",
            Self::Increment => "++",
            Self::Decrement => "--",
            Self::Factorial => "!",
            Self::Factorial2 => "!!",
            Self::Derivative(_) => "'",
            Self::Repeated => "..",
            Self::RepeatedNull => "...",
            Self::Unset => "=.",
        }
    }

    pub(crate) fn precedence(self) -> u8 {
        match self {
            Self::Function => 15,
            Self::Increment
            | Self::Decrement
            | Self::Factorial
            | Self::Factorial2
            | Self::Derivative(_) => 80,
            Self::Repeated | Self::RepeatedNull => 76,
            Self::Unset => 10,
        }
    }
}

impl fmt::Display for SyntaxExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_expr(self, f, 0)
    }
}

fn format_expr(
    expr: &SyntaxExpr,
    f: &mut fmt::Formatter<'_>,
    parent_precedence: u8,
) -> fmt::Result {
    let precedence = expr.precedence();
    let needs_parens = precedence < parent_precedence;

    if needs_parens {
        f.write_str("(")?;
    }

    match &expr.kind {
        SyntaxExprKind::Integer(value) => f.write_str(value)?,
        SyntaxExprKind::String(value) => write_escaped_string(f, value)?,
        SyntaxExprKind::Symbol(value) => f.write_str(value)?,
        SyntaxExprKind::List(items) => {
            f.write_str("{")?;

            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }

                format_expr(item, f, 0)?;
            }

            f.write_str("}")?;
        }
        SyntaxExprKind::Association(items) => {
            f.write_str("<|")?;

            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }

                format_expr(item, f, 0)?;
            }

            f.write_str("|>")?;
        }
        SyntaxExprKind::Slot(spec) => match spec {
            SlotSpec::Anonymous => f.write_str("#")?,
            SlotSpec::Number(value) => write!(f, "#{value}")?,
            SlotSpec::Named(value) => write!(f, "#{value}")?,
            SlotSpec::Sequence => f.write_str("##")?,
            SlotSpec::SequenceNumber(value) => write!(f, "##{value}")?,
        },
        SyntaxExprKind::Blank { kind, head } => {
            f.write_str(kind.token())?;

            if let Some(head) = head {
                format_expr(head, f, precedence)?;
            }
        }
        SyntaxExprKind::Pattern { name, pattern } => {
            format_expr(name, f, precedence)?;
            format_expr(pattern, f, precedence)?;
        }
        SyntaxExprKind::Prefix { op, expr } => {
            f.write_str(op.token())?;
            format_expr(expr, f, op.precedence())?;
        }
        SyntaxExprKind::Binary { op, lhs, rhs } => {
            let lhs_precedence = op.precedence() + u8::from(op.is_right_associative());
            let rhs_precedence = op.precedence() + u8::from(!op.is_right_associative());

            format_expr(lhs, f, lhs_precedence)?;
            if matches!(op, BinaryOp::CompoundExpression) {
                write!(f, "{} ", op.token())?;
            } else {
                write!(f, " {} ", op.token())?;
            }
            format_expr(rhs, f, rhs_precedence)?;
        }
        SyntaxExprKind::Postfix { op, expr } => {
            format_expr(expr, f, op.precedence())?;
            match op {
                PostfixOp::Derivative(order) => {
                    for _ in 0..*order {
                        f.write_str("'")?;
                    }
                }
                PostfixOp::Factorial | PostfixOp::Factorial2 => f.write_str(op.token())?,
                _ => write!(f, " {}", op.token())?,
            }
        }
        SyntaxExprKind::Call { head, args } => {
            format_expr(head, f, precedence)?;
            f.write_str("[")?;

            for (index, arg) in args.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }

                format_expr(arg, f, 0)?;
            }

            f.write_str("]")?;
        }
    }

    if needs_parens {
        f.write_str(")")?;
    }

    Ok(())
}

fn write_escaped_string(f: &mut fmt::Formatter<'_>, value: &str) -> fmt::Result {
    f.write_str("\"")?;

    for ch in value.chars() {
        match ch {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\t' => f.write_str("\\t")?,
            other => write!(f, "{other}")?,
        }
    }

    f.write_str("\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_expr;

    fn z() -> Span {
        Span::default()
    }

    #[test]
    fn syntax_string_display_escapes_round_trippable_characters() {
        let expr = SyntaxExpr::string("a\"b\\c\nd\t", z());
        let rendered = expr.to_string();

        assert_eq!(rendered, "\"a\\\"b\\\\c\\nd\\t\"");
        assert_eq!(parse_expr(&rendered).unwrap(), expr);
    }
}
