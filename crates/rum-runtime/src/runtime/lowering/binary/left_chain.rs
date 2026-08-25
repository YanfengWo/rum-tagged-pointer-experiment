use super::*;

struct LeftChainLink<'a> {
    op: BinaryOp,
    rhs: &'a SyntaxExpr,
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
    let mut accumulator = lower_syntax_expr_with(base, resolve_symbol)?;

    for link in links {
        let lowered_rhs = lower_syntax_expr_with(link.rhs, resolve_symbol)?;
        accumulator =
            apply_left_chain_link(link.op, accumulator, lowered_rhs, link.rhs, resolve_symbol)?;
    }

    Ok(accumulator)
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
    accumulator: Expr,
    lowered_rhs: Expr,
    raw_rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    match op {
        BinaryOp::MessageName => Ok(message_name_expr(
            match accumulator {
                Expr::Atom(Atom::Symbol(symbol)) => symbol,
                other => {
                    return Ok(canonical_call(
                        heads::messages::MESSAGE_NAME,
                        vec![other, lower_message_tag_expr(raw_rhs, resolve_symbol)?],
                    ));
                }
            },
            lower_message_tag_string(raw_rhs),
        )),
        BinaryOp::ReplaceAll => Ok(canonical_call(
            heads::rules::REPLACE_ALL,
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::ReplaceRepeated => Ok(canonical_call(
            heads::rules::REPLACE_REPEATED,
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::Pattern => {
            if matches!(&accumulator, Expr::Atom(Atom::Symbol(_))) {
                Ok(canonical_call(
                    heads::patterns::PATTERN,
                    vec![accumulator, lowered_rhs],
                ))
            } else {
                Ok(canonical_call("Optional", vec![accumulator, lowered_rhs]))
            }
        }
        BinaryOp::Alternatives => Ok(canonical_call(
            "Alternatives",
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::PatternTest => Ok(canonical_call(
            "PatternTest",
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::StringExpression => Ok(canonical_call(
            "StringExpression",
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::StringJoin => Ok(canonical_call("StringJoin", vec![accumulator, lowered_rhs])),
        BinaryOp::PostfixApply => Ok(Expr::call(lowered_rhs, vec![accumulator])),
        BinaryOp::RightComposition => Ok(canonical_call(
            "RightComposition",
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::Add => Ok(canonical_call(
            heads::arithmetic::PLUS,
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::Subtract => Ok(canonical_call(
            heads::arithmetic::PLUS,
            vec![
                accumulator,
                canonical_call(
                    heads::arithmetic::TIMES,
                    vec![Expr::integer(BigInt::from(-1)), lowered_rhs],
                ),
            ],
        )),
        BinaryOp::Times => Ok(canonical_call(
            heads::arithmetic::TIMES,
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::Dot => Ok(canonical_call("Dot", vec![accumulator, lowered_rhs])),
        BinaryOp::NonCommutativeMultiply => Ok(canonical_call(
            "NonCommutativeMultiply",
            vec![accumulator, lowered_rhs],
        )),
        BinaryOp::Divide => {
            if let Some(quotient) = divided_numeric_literals(&accumulator, &lowered_rhs) {
                Ok(quotient)
            } else {
                Ok(canonical_call(
                    heads::arithmetic::TIMES,
                    vec![
                        accumulator,
                        canonical_call(
                            heads::arithmetic::POWER,
                            vec![lowered_rhs, Expr::integer(BigInt::from(-1))],
                        ),
                    ],
                ))
            }
        }
        _ => unreachable!("non-iterative operator in left-chain fold"),
    }
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
