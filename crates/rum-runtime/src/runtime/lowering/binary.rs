use super::*;

mod comparison;

use comparison::{comparison_head, lower_comparison_chain};

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

pub(super) fn lower_binary_sugar<F>(
    op: BinaryOp,
    lhs: &SyntaxExpr,
    rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    if comparison_head(op).is_some() {
        return lower_comparison_chain(op, lhs, rhs, resolve_symbol);
    }

    if matches!(op, BinaryOp::CompoundExpression) {
        return lower_compound_expression_sugar(lhs, rhs, resolve_symbol);
    }

    let lowered_lhs = lower_syntax_expr_with(lhs, resolve_symbol)?;
    let lowered_rhs = lower_syntax_expr_with(rhs, resolve_symbol)?;

    match op {
        BinaryOp::MessageName => Ok(message_name_expr(
            match lowered_lhs {
                Expr::Atom(Atom::Symbol(symbol)) => symbol,
                other => {
                    return Ok(canonical_call(
                        heads::messages::MESSAGE_NAME,
                        vec![other, lower_message_tag_expr(rhs, resolve_symbol)?],
                    ));
                }
            },
            lower_message_tag_string(rhs),
        )),
        BinaryOp::TagSet => lower_tag_set_sugar(lowered_lhs, lowered_rhs),
        BinaryOp::Set => Ok(canonical_call(
            heads::rules::SET,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::UpSet => Ok(canonical_call("UpSet", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::AddTo => Ok(canonical_call("AddTo", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::SubtractFrom => Ok(canonical_call(
            "SubtractFrom",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::TimesBy => Ok(canonical_call("TimesBy", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::DivideBy => Ok(canonical_call("DivideBy", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::ApplyTo => Ok(canonical_call("ApplyTo", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::SetDelayed => Ok(canonical_call(
            heads::rules::SET_DELAYED,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::UpSetDelayed => Ok(canonical_call(
            "UpSetDelayed",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceAll => Ok(canonical_call(
            heads::rules::REPLACE_ALL,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceRepeated => Ok(canonical_call(
            heads::rules::REPLACE_REPEATED,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Rule => Ok(canonical_call(
            heads::rules::RULE,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::RuleDelayed => Ok(canonical_call(
            heads::rules::RULE_DELAYED,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Condition => Ok(canonical_call(
            heads::patterns::CONDITION,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Or => Ok(canonical_call("Or", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::And => Ok(canonical_call("And", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::Pattern => {
            if matches!(&lowered_lhs, Expr::Atom(Atom::Symbol(_))) {
                Ok(canonical_call(
                    heads::patterns::PATTERN,
                    vec![lowered_lhs, lowered_rhs],
                ))
            } else {
                Ok(canonical_call("Optional", vec![lowered_lhs, lowered_rhs]))
            }
        }
        BinaryOp::Alternatives => Ok(canonical_call(
            "Alternatives",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::PatternTest => Ok(canonical_call(
            "PatternTest",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::StringExpression => Ok(canonical_call(
            "StringExpression",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::StringJoin => Ok(canonical_call("StringJoin", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::Apply => Ok(Expr::call(lowered_lhs, vec![lowered_rhs])),
        BinaryOp::PostfixApply => Ok(Expr::call(lowered_rhs, vec![lowered_lhs])),
        BinaryOp::Map => Ok(canonical_call("Map", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::MapAll => Ok(canonical_call("MapAll", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::ApplyHead => Ok(canonical_call("Apply", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::MapApply => Ok(canonical_call("MapApply", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::Composition => Ok(canonical_call(
            "Composition",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::RightComposition => Ok(canonical_call(
            "RightComposition",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Add => Ok(canonical_call(
            heads::arithmetic::PLUS,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Subtract => Ok(canonical_call(
            heads::arithmetic::PLUS,
            vec![
                lowered_lhs,
                canonical_call(
                    heads::arithmetic::TIMES,
                    vec![Expr::integer(BigInt::from(-1)), lowered_rhs],
                ),
            ],
        )),
        BinaryOp::Times => Ok(canonical_call(
            heads::arithmetic::TIMES,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Dot => Ok(canonical_call("Dot", vec![lowered_lhs, lowered_rhs])),
        BinaryOp::NonCommutativeMultiply => Ok(canonical_call(
            "NonCommutativeMultiply",
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Divide => {
            if let Some(quotient) = divided_numeric_literals(&lowered_lhs, &lowered_rhs) {
                Ok(quotient)
            } else {
                Ok(canonical_call(
                    heads::arithmetic::TIMES,
                    vec![
                        lowered_lhs,
                        canonical_call(
                            heads::arithmetic::POWER,
                            vec![lowered_rhs, Expr::integer(BigInt::from(-1))],
                        ),
                    ],
                ))
            }
        }
        BinaryOp::Power => Ok(canonical_call(
            heads::arithmetic::POWER,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Function => Ok(canonical_call(
            heads::functions::FUNCTION,
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::SameQ
        | BinaryOp::UnsameQ
        | BinaryOp::Equal
        | BinaryOp::Unequal
        | BinaryOp::Less
        | BinaryOp::Greater
        | BinaryOp::LessEqual
        | BinaryOp::GreaterEqual => unreachable!("comparison ops handled before lowering operands"),
        BinaryOp::CompoundExpression => {
            unreachable!("compound expression handled before lowering operands")
        }
    }
}

/// Lowers a `;` spine iteratively: the right-nested syntax chain is walked in
/// a loop, so statement count no longer consumes lowering recursion depth.
/// The output is byte-identical to the recursive form — one flat n-ary
/// `CompoundExpression` with the items in source order.
fn lower_compound_expression_sugar<F>(
    lhs: &SyntaxExpr,
    rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let mut items = Vec::new();
    let mut lhs = lhs;
    let mut rhs = rhs;

    loop {
        let lowered = lower_syntax_expr_with(lhs, resolve_symbol)?;
        push_compound_expression_item(&mut items, lowered);

        match &rhs.kind {
            SyntaxExprKind::Binary {
                op: BinaryOp::CompoundExpression,
                lhs: next_lhs,
                rhs: next_rhs,
            } => {
                lhs = next_lhs;
                rhs = next_rhs;
            }
            _ => break,
        }
    }

    let lowered_tail = lower_syntax_expr_with(rhs, resolve_symbol)?;
    push_compound_expression_item(&mut items, lowered_tail);

    Ok(canonical_call("CompoundExpression", items))
}

fn push_compound_expression_item(items: &mut Vec<Expr>, expr: Expr) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(&head, "CompoundExpression") => {
            items.extend(args);
        }
        other => items.push(other),
    }
}

pub(super) fn lower_message_tag_expr<F>(
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    match &expr.kind {
        SyntaxExprKind::Symbol(value) => Ok(Expr::string(value.clone())),
        SyntaxExprKind::String(value) => Ok(Expr::string(value.clone())),
        _ => lower_syntax_expr_with(expr, resolve_symbol),
    }
}

pub(super) fn lower_message_tag_string(expr: &SyntaxExpr) -> String {
    match &expr.kind {
        SyntaxExprKind::Symbol(value) | SyntaxExprKind::String(value) => value.clone(),
        _ => expr.to_string(),
    }
}

pub(super) fn lower_tag_set_sugar(tag: Expr, rhs: Expr) -> Result<Expr, EvalError> {
    match rhs {
        Expr::Call { head, args }
            if is_system_symbol_name(&head, heads::rules::SET) && args.len() == 2 =>
        {
            Ok(canonical_call(
                heads::rules::TAG_SET,
                vec![tag, args[0].clone(), args[1].clone()],
            ))
        }
        Expr::Call { head, args }
            if is_system_symbol_name(&head, heads::rules::SET_DELAYED) && args.len() == 2 =>
        {
            Ok(canonical_call(
                heads::rules::TAG_SET_DELAYED,
                vec![tag, args[0].clone(), args[1].clone()],
            ))
        }
        Expr::Call { head, args }
            if is_system_symbol_name(&head, heads::rules::UNSET) && args.len() == 1 =>
        {
            Ok(canonical_call(
                heads::rules::TAG_UNSET,
                vec![tag, args[0].clone()],
            ))
        }
        _ => Ok(canonical_call(heads::rules::TAG_SET, vec![tag, rhs])),
    }
}
