use super::*;

struct LeftChainLink<'a> {
    op: BinaryOp,
    rhs: &'a SyntaxExpr,
}

enum LeftChainAccumulator {
    Opaque(Expr),
    Additive(Vec<Expr>),
    Multiplicative(Vec<Expr>),
    RightComposition(Vec<Expr>),
    Variadic { head: &'static str, args: Vec<Expr> },
}

impl LeftChainAccumulator {
    fn into_expr(self) -> Expr {
        match self {
            Self::Opaque(expr) => expr,
            Self::Additive(args) => canonical_call(heads::arithmetic::PLUS, args),
            Self::Multiplicative(args) => canonical_call(heads::arithmetic::TIMES, args),
            Self::RightComposition(args) => canonical_call("RightComposition", args),
            Self::Variadic { head, args } => canonical_call(head, args),
        }
    }

    fn append_additive(self, rhs: Expr) -> Self {
        match self {
            Self::Additive(mut args) => {
                args.push(rhs);
                Self::Additive(args)
            }
            other => Self::Additive(vec![other.into_expr(), rhs]),
        }
    }

    fn append_multiplicative(self, rhs: Expr) -> Self {
        match self {
            Self::Multiplicative(mut args) => {
                args.push(rhs);
                Self::Multiplicative(args)
            }
            other => Self::Multiplicative(vec![other.into_expr(), rhs]),
        }
    }

    fn divide(self, rhs: Expr) -> Self {
        match self {
            Self::Opaque(lhs) => divide_opaque_accumulator(lhs, rhs),
            Self::Multiplicative(mut args) => {
                args.push(reciprocal(rhs));
                Self::Multiplicative(args)
            }
            other => divide_opaque_accumulator(other.into_expr(), rhs),
        }
    }

    fn append_right_composition(self, rhs: Expr) -> Self {
        match self {
            Self::RightComposition(mut args) => {
                args.push(rhs);
                Self::RightComposition(args)
            }
            other => Self::RightComposition(vec![other.into_expr(), rhs]),
        }
    }

    fn append_variadic(self, head: &'static str, rhs: Expr) -> Self {
        match self {
            Self::Variadic {
                head: active_head,
                mut args,
            } if active_head == head => {
                args.push(rhs);
                Self::Variadic { head, args }
            }
            other => Self::Variadic {
                head,
                args: vec![other.into_expr(), rhs],
            },
        }
    }
}

pub(super) fn is_iterative_left_chain_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Times
            | BinaryOp::Divide
            | BinaryOp::Dot
            | BinaryOp::NonCommutativeMultiply
            | BinaryOp::StringJoin
            | BinaryOp::MessageName
            | BinaryOp::ReplaceAll
            | BinaryOp::ReplaceRepeated
            | BinaryOp::Pattern
            | BinaryOp::Alternatives
            | BinaryOp::PatternTest
            | BinaryOp::StringExpression
            | BinaryOp::PostfixApply
            | BinaryOp::RightComposition
    )
}

pub(super) fn lower_left_chain<F>(
    op: BinaryOp,
    lhs: &SyntaxExpr,
    rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let (base, links) = collect_left_spine(op, lhs, rhs);
    let mut accumulator =
        LeftChainAccumulator::Opaque(lower_syntax_expr_with(base, resolve_symbol)?);

    for link in links {
        let lowered_rhs = lower_syntax_expr_with(link.rhs, resolve_symbol)?;
        accumulator =
            apply_left_chain_link(link.op, accumulator, lowered_rhs, link.rhs, resolve_symbol)?;
    }

    Ok(accumulator.into_expr())
}

fn collect_left_spine<'a>(
    root_op: BinaryOp,
    mut lhs: &'a SyntaxExpr,
    rhs: &'a SyntaxExpr,
) -> (&'a SyntaxExpr, Vec<LeftChainLink<'a>>) {
    let mut links = vec![LeftChainLink { op: root_op, rhs }];

    loop {
        match &lhs.kind {
            SyntaxExprKind::Binary {
                op,
                lhs: next_lhs,
                rhs: next_rhs,
            } if is_iterative_left_chain_op(*op) => {
                links.push(LeftChainLink {
                    op: *op,
                    rhs: next_rhs,
                });
                lhs = next_lhs;
            }
            _ => break,
        }
    }

    links.reverse();
    (lhs, links)
}

fn apply_left_chain_link<F>(
    op: BinaryOp,
    accumulator: LeftChainAccumulator,
    lowered_rhs: Expr,
    raw_rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<LeftChainAccumulator, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    match op {
        BinaryOp::Add => return Ok(accumulator.append_additive(lowered_rhs)),
        BinaryOp::Subtract => {
            return Ok(accumulator.append_additive(canonical_call(
                heads::arithmetic::TIMES,
                vec![Expr::integer(BigInt::from(-1)), lowered_rhs],
            )));
        }
        BinaryOp::Times => return Ok(accumulator.append_multiplicative(lowered_rhs)),
        BinaryOp::Divide => return Ok(accumulator.divide(lowered_rhs)),
        BinaryOp::RightComposition => {
            return Ok(accumulator.append_right_composition(lowered_rhs));
        }
        BinaryOp::Dot => return Ok(accumulator.append_variadic("Dot", lowered_rhs)),
        BinaryOp::StringJoin => {
            return Ok(accumulator.append_variadic("StringJoin", lowered_rhs));
        }
        BinaryOp::Alternatives => {
            return Ok(accumulator.append_variadic("Alternatives", lowered_rhs));
        }
        BinaryOp::StringExpression => {
            return Ok(accumulator.append_variadic("StringExpression", lowered_rhs));
        }
        _ => {}
    }

    let accumulator = accumulator.into_expr();
    match op {
        BinaryOp::MessageName => Ok(LeftChainAccumulator::Opaque(message_name_expr(
            match accumulator {
                Expr::Atom(Atom::Symbol(symbol)) => symbol,
                other => {
                    return Ok(LeftChainAccumulator::Opaque(canonical_call(
                        heads::messages::MESSAGE_NAME,
                        vec![other, lower_message_tag_expr(raw_rhs, resolve_symbol)?],
                    )));
                }
            },
            lower_message_tag_string(raw_rhs),
        ))),
        BinaryOp::ReplaceAll => Ok(LeftChainAccumulator::Opaque(canonical_call(
            heads::rules::REPLACE_ALL,
            vec![accumulator, lowered_rhs],
        ))),
        BinaryOp::ReplaceRepeated => Ok(LeftChainAccumulator::Opaque(canonical_call(
            heads::rules::REPLACE_REPEATED,
            vec![accumulator, lowered_rhs],
        ))),
        BinaryOp::Pattern => {
            if matches!(&accumulator, Expr::Atom(Atom::Symbol(_))) {
                Ok(LeftChainAccumulator::Opaque(canonical_call(
                    heads::patterns::PATTERN,
                    vec![accumulator, lowered_rhs],
                )))
            } else {
                Ok(LeftChainAccumulator::Opaque(canonical_call(
                    "Optional",
                    vec![accumulator, lowered_rhs],
                )))
            }
        }
        BinaryOp::PatternTest => Ok(LeftChainAccumulator::Opaque(canonical_call(
            "PatternTest",
            vec![accumulator, lowered_rhs],
        ))),
        BinaryOp::PostfixApply => Ok(LeftChainAccumulator::Opaque(Expr::call(
            lowered_rhs,
            vec![accumulator],
        ))),
        BinaryOp::NonCommutativeMultiply => Ok(LeftChainAccumulator::Opaque(canonical_call(
            "NonCommutativeMultiply",
            vec![accumulator, lowered_rhs],
        ))),
        BinaryOp::Add
        | BinaryOp::Subtract
        | BinaryOp::Times
        | BinaryOp::Divide
        | BinaryOp::Dot
        | BinaryOp::StringJoin
        | BinaryOp::Alternatives
        | BinaryOp::StringExpression
        | BinaryOp::RightComposition => {
            unreachable!("variadic-family link handled before materializing")
        }
        _ => unreachable!("non-iterative operator in left-chain fold"),
    }
}

fn divide_opaque_accumulator(lhs: Expr, rhs: Expr) -> LeftChainAccumulator {
    if let Some(quotient) = divided_numeric_literals(&lhs, &rhs) {
        LeftChainAccumulator::Opaque(quotient)
    } else {
        LeftChainAccumulator::Multiplicative(vec![lhs, reciprocal(rhs)])
    }
}

fn reciprocal(expr: Expr) -> Expr {
    canonical_call(
        heads::arithmetic::POWER,
        vec![expr, Expr::integer(BigInt::from(-1))],
    )
}

fn numeric_literal_rational(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.to_bigint())),
        Expr::Atom(Atom::Rational(value)) => Some(value.as_ref().clone()),
        _ => None,
    }
}

fn divided_numeric_literals(lhs: &Expr, rhs: &Expr) -> Option<Expr> {
    Some(Expr::rational(
        numeric_literal_rational(lhs)?.div(&numeric_literal_rational(rhs)?)?,
    ))
}
