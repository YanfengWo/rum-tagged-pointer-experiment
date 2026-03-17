use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxExpr {
    Integer(String),
    String(String),
    Symbol(String),
    List(Vec<SyntaxExpr>),
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Set,
    SetDelayed,
    ReplaceAll,
    Rule,
    RuleDelayed,
    Condition,
    Add,
    Subtract,
    Times,
    Divide,
    Power,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostfixOp {
    Function,
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
        }
    }

    pub(crate) fn precedence(self) -> u8 {
        65
    }
}

impl BinaryOp {
    pub fn token(self) -> &'static str {
        match self {
            Self::Set => "=",
            Self::SetDelayed => ":=",
            Self::ReplaceAll => "/.",
            Self::Rule => "->",
            Self::RuleDelayed => ":>",
            Self::Condition => "/;",
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Times => "*",
            Self::Divide => "/",
            Self::Power => "^",
        }
    }

    pub(crate) fn precedence(self) -> u8 {
        match self {
            Self::Set | Self::SetDelayed => 10,
            Self::ReplaceAll => 20,
            Self::Rule | Self::RuleDelayed => 30,
            Self::Condition => 40,
            Self::Add | Self::Subtract => 50,
            Self::Times | Self::Divide => 60,
            Self::Power => 70,
        }
    }

    pub(crate) fn is_right_associative(self) -> bool {
        matches!(
            self,
            Self::Set
                | Self::SetDelayed
                | Self::Rule
                | Self::RuleDelayed
                | Self::Condition
                | Self::Power
        )
    }
}

impl PostfixOp {
    pub fn token(self) -> &'static str {
        match self {
            Self::Function => "&",
        }
    }

    pub(crate) fn precedence(self) -> u8 {
        5
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

    match expr {
        SyntaxExpr::Integer(value) => f.write_str(value)?,
        SyntaxExpr::String(value) => write!(f, "\"{value}\"")?,
        SyntaxExpr::Symbol(value) => f.write_str(value)?,
        SyntaxExpr::List(items) => {
            f.write_str("{")?;

            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }

                format_expr(item, f, 0)?;
            }

            f.write_str("}")?;
        }
        SyntaxExpr::Slot(spec) => match spec {
            SlotSpec::Anonymous => f.write_str("#")?,
            SlotSpec::Number(value) => write!(f, "#{value}")?,
            SlotSpec::Named(value) => write!(f, "#{value}")?,
        },
        SyntaxExpr::Blank { kind, head } => {
            f.write_str(kind.token())?;

            if let Some(head) = head {
                format_expr(head, f, precedence)?;
            }
        }
        SyntaxExpr::Pattern { name, pattern } => {
            format_expr(name, f, precedence)?;
            format_expr(pattern, f, precedence)?;
        }
        SyntaxExpr::Prefix { op, expr } => {
            f.write_str(op.token())?;
            format_expr(expr, f, op.precedence())?;
        }
        SyntaxExpr::Binary { op, lhs, rhs } => {
            let lhs_precedence = op.precedence() + u8::from(op.is_right_associative());
            let rhs_precedence = op.precedence() + u8::from(!op.is_right_associative());

            format_expr(lhs, f, lhs_precedence)?;
            write!(f, " {} ", op.token())?;
            format_expr(rhs, f, rhs_precedence)?;
        }
        SyntaxExpr::Postfix { op, expr } => {
            format_expr(expr, f, op.precedence())?;
            write!(f, " {}", op.token())?;
        }
        SyntaxExpr::Call { head, args } => {
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

impl SyntaxExpr {
    fn precedence(&self) -> u8 {
        match self {
            Self::Postfix { op, .. } => op.precedence(),
            Self::Binary { op, .. } => op.precedence(),
            Self::Prefix { op, .. } => op.precedence(),
            Self::Pattern { .. } => 80,
            Self::Blank { .. } => 80,
            Self::Call { .. } => 90,
            Self::Integer(_)
            | Self::String(_)
            | Self::Symbol(_)
            | Self::List(_)
            | Self::Slot(_) => 95,
        }
    }
}
