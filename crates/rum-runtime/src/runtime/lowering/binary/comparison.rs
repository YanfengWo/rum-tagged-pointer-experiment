use super::*;

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

pub(super) fn lower_comparison_chain<F>(
    op: BinaryOp,
    lhs: &SyntaxExpr,
    rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let mut operands = Vec::new();
    let mut ops = Vec::new();
    collect_comparison_chain(op, lhs, rhs, &mut operands, &mut ops);

    if ops.len() == 1 {
        return Ok(canonical_call(
            comparison_head(op).expect("checked comparison op"),
            vec![
                lower_syntax_expr_with(lhs, resolve_symbol)?,
                lower_syntax_expr_with(rhs, resolve_symbol)?,
            ],
        ));
    }

    if ops.iter().all(|candidate| *candidate == ops[0]) {
        let args = operands
            .into_iter()
            .map(|operand| lower_syntax_expr_with(operand, resolve_symbol))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(canonical_call(
            comparison_head(ops[0]).expect("comparison chain op"),
            args,
        ));
    }

    let lowered_operands = operands
        .into_iter()
        .map(|operand| lower_syntax_expr_with(operand, resolve_symbol))
        .collect::<Result<Vec<_>, _>>()?;
    let comparisons = ops
        .into_iter()
        .zip(lowered_operands.windows(2))
        .map(|(chain_op, pair)| {
            canonical_call(
                comparison_head(chain_op).expect("comparison chain op"),
                vec![pair[0].clone(), pair[1].clone()],
            )
        })
        .collect();

    Ok(canonical_call("And", comparisons))
}

enum ComparisonWork<'a> {
    Visit(&'a SyntaxExpr),
    EmitOp(BinaryOp),
}

fn collect_comparison_chain<'a>(
    root_op: BinaryOp,
    lhs: &'a SyntaxExpr,
    rhs: &'a SyntaxExpr,
    operands: &mut Vec<&'a SyntaxExpr>,
    ops: &mut Vec<BinaryOp>,
) {
    let mut work = vec![
        ComparisonWork::Visit(rhs),
        ComparisonWork::EmitOp(root_op),
        ComparisonWork::Visit(lhs),
    ];

    while let Some(item) = work.pop() {
        match item {
            ComparisonWork::Visit(expr) => match &expr.kind {
                SyntaxExprKind::Binary { op, lhs, rhs } if comparison_head(*op).is_some() => {
                    work.push(ComparisonWork::Visit(rhs));
                    work.push(ComparisonWork::EmitOp(*op));
                    work.push(ComparisonWork::Visit(lhs));
                }
                _ => operands.push(expr),
            },
            ComparisonWork::EmitOp(op) => ops.push(op),
        }
    }
}
