use super::*;
use rum_syntax::{MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS, Span};

pub(super) fn comparison_head(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::SameQ => Some("SameQ"),
        BinaryOp::UnsameQ => Some("UnsameQ"),
        BinaryOp::Equal => Some("Equal"),
        BinaryOp::Unequal => Some("Unequal"),
        BinaryOp::Less => Some("Less"),
        BinaryOp::Greater => Some("Greater"),
        BinaryOp::LessEqual => Some("LessEqual"),
        BinaryOp::GreaterEqual => Some("GreaterEqual"),
        _ => None,
    }
}

pub(in crate::runtime::lowering) fn lower_comparison_chain<F>(
    operands: &[SyntaxExpr],
    operators: &[BinaryOp],
    span: Span,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    debug_assert_eq!(operands.len(), operators.len() + 1);
    debug_assert!(
        operators
            .iter()
            .all(|operator| comparison_head(*operator).is_some())
    );

    if operators.len() > MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS {
        return Err(SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            span,
        }
        .into());
    }

    let Some(first_operator) = operators.first().copied() else {
        return operands.first().map_or_else(
            || Ok(canonical_call("Inequality", Vec::new())),
            |operand| lower_syntax_expr_with(operand, resolve_symbol),
        );
    };

    if operators
        .iter()
        .all(|candidate| *candidate == first_operator)
    {
        let args = operands
            .iter()
            .map(|operand| lower_syntax_expr_with(operand, resolve_symbol))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(canonical_call(
            comparison_head(first_operator).expect("comparison chain operator"),
            args,
        ));
    }

    let mut args = Vec::with_capacity(operators.len() * 2 + 1);
    for (index, operand) in operands.iter().enumerate() {
        args.push(lower_syntax_expr_with(operand, resolve_symbol)?);
        if let Some(operator) = operators.get(index) {
            args.push(system_expr(
                comparison_head(*operator).expect("comparison chain operator"),
            ));
        }
    }

    Ok(canonical_call("Inequality", args))
}
