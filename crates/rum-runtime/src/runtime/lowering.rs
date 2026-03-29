use super::*;

pub fn lower_syntax_expr(expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    lower_syntax_expr_with(expr, &mut |raw| {
        resolve_unqualified_symbol(raw).or_else(|| Some(symbol_from_raw_name(raw)))
    })
}

pub(super) fn lower_syntax_expr_with<F>(
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    // Keep a sharp boundary here:
    // - direct syntax that already matches runtime Expr shape should preserve its head
    // - parser-only sugar should be rewritten into canonical system-head forms
    match &expr.kind {
        SyntaxExprKind::Integer(value) => lower_integer_atom(value),
        SyntaxExprKind::String(value) => Ok(Expr::string(value.clone())),
        SyntaxExprKind::Symbol(value) => Ok(lower_symbol(value, resolve_symbol)),
        SyntaxExprKind::List(items) => lower_list_sugar(items, resolve_symbol),
        SyntaxExprKind::Association(items) => lower_association_sugar(items, resolve_symbol),
        SyntaxExprKind::Slot(spec) => lower_slot_sugar(spec),
        SyntaxExprKind::Blank { kind, head } => {
            lower_blank_sugar(*kind, head.as_deref(), resolve_symbol)
        }
        SyntaxExprKind::Pattern { name, pattern } => {
            lower_pattern_sugar(name, pattern, resolve_symbol)
        }
        SyntaxExprKind::Prefix { op, expr } => lower_prefix_sugar(*op, expr, resolve_symbol),
        SyntaxExprKind::Binary { op, lhs, rhs } => {
            lower_binary_sugar(*op, lhs, rhs, resolve_symbol)
        }
        SyntaxExprKind::Postfix { op, expr } => lower_postfix_sugar(*op, expr, resolve_symbol),
        SyntaxExprKind::Call { head, args } => {
            lower_head_preserving_call(head, args, resolve_symbol)
        }
    }
}

fn lower_integer_atom(value: &str) -> Result<Expr, EvalError> {
    let bigint =
        BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| EvalError::InvalidInteger {
            literal: value.to_owned(),
        })?;

    Ok(Expr::integer(bigint))
}

fn lower_syntax_args<F>(args: &[SyntaxExpr], resolve_symbol: &mut F) -> Result<Vec<Expr>, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    args.iter()
        .map(|arg| lower_syntax_expr_with(arg, resolve_symbol))
        .collect()
}

fn lower_head_preserving_call<F>(
    head: &SyntaxExpr,
    args: &[SyntaxExpr],
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered_head = lower_syntax_expr_with(head, resolve_symbol)?;
    let lowered_args = lower_syntax_args(args, resolve_symbol)?;
    Ok(Expr::call(lowered_head, lowered_args))
}

fn canonical_call(head: &'static str, args: Vec<Expr>) -> Expr {
    Expr::call(system_expr(head), args)
}

// Parser sugar lowers into canonical runtime heads.
fn lower_list_sugar<F>(items: &[SyntaxExpr], resolve_symbol: &mut F) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Ok(canonical_call(
        heads::structure::LIST,
        lower_syntax_args(items, resolve_symbol)?,
    ))
}

fn lower_association_sugar<F>(
    items: &[SyntaxExpr],
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Ok(canonical_call(
        "Association",
        lower_syntax_args(items, resolve_symbol)?,
    ))
}

fn lower_pattern_sugar<F>(
    name: &SyntaxExpr,
    pattern: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Ok(canonical_call(
        heads::patterns::PATTERN,
        vec![
            lower_syntax_expr_with(name, resolve_symbol)?,
            lower_syntax_expr_with(pattern, resolve_symbol)?,
        ],
    ))
}

pub(super) fn lower_slot_sugar(spec: &SlotSpec) -> Result<Expr, EvalError> {
    let (head, arg) = match spec {
        SlotSpec::Anonymous => (heads::functions::SLOT, Expr::integer(BigInt::from(1_u8))),
        SlotSpec::Number(value) => (heads::functions::SLOT, lower_integer_atom(value)?),
        SlotSpec::Named(value) => (heads::functions::SLOT, Expr::string(value.clone())),
        SlotSpec::Sequence => (
            heads::functions::SLOT_SEQUENCE,
            Expr::integer(BigInt::from(1_u8)),
        ),
        SlotSpec::SequenceNumber(value) => {
            (heads::functions::SLOT_SEQUENCE, lower_integer_atom(value)?)
        }
    };

    Ok(canonical_call(head, vec![arg]))
}

pub(super) fn lower_blank_sugar<F>(
    kind: BlankKind,
    head: Option<&SyntaxExpr>,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let head_symbol = match kind {
        BlankKind::Blank => heads::patterns::BLANK,
        BlankKind::BlankSequence => heads::patterns::BLANK_SEQUENCE,
        BlankKind::BlankNullSequence => heads::patterns::BLANK_NULL_SEQUENCE,
    };

    let args = match head {
        Some(head) => vec![lower_syntax_expr_with(head, resolve_symbol)?],
        None => Vec::new(),
    };

    Ok(canonical_call(head_symbol, args))
}

pub(super) fn lower_prefix_sugar<F>(
    op: PrefixOp,
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered = lower_syntax_expr_with(expr, resolve_symbol)?;

    match op {
        PrefixOp::Plus => Ok(lowered),
        PrefixOp::Minus => Ok(canonical_call(
            heads::arithmetic::TIMES,
            vec![Expr::integer(BigInt::from(-1)), lowered],
        )),
    }
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
    let lowered_lhs = lower_syntax_expr_with(lhs, resolve_symbol)?;
    let lowered_rhs = lower_syntax_expr_with(rhs, resolve_symbol)?;

    match op {
        BinaryOp::CompoundExpression => {
            let mut items = Vec::new();
            push_compound_expression_item(&mut items, lowered_lhs);
            push_compound_expression_item(&mut items, lowered_rhs);
            Ok(canonical_call("CompoundExpression", items))
        }
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
        BinaryOp::SetDelayed => Ok(canonical_call(
            heads::rules::SET_DELAYED,
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
        BinaryOp::Apply => Ok(Expr::call(lowered_lhs, vec![lowered_rhs])),
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
        BinaryOp::Divide => Ok(canonical_call(
            heads::arithmetic::TIMES,
            vec![
                lowered_lhs,
                canonical_call(
                    heads::arithmetic::POWER,
                    vec![lowered_rhs, Expr::integer(BigInt::from(-1))],
                ),
            ],
        )),
        BinaryOp::Power => Ok(canonical_call(
            heads::arithmetic::POWER,
            vec![lowered_lhs, lowered_rhs],
        )),
    }
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

pub(super) fn lower_postfix_sugar<F>(
    op: PostfixOp,
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered = lower_syntax_expr_with(expr, resolve_symbol)?;

    match op {
        PostfixOp::Function => Ok(canonical_call(heads::functions::FUNCTION, vec![lowered])),
        PostfixOp::Unset => Ok(canonical_call(heads::rules::UNSET, vec![lowered])),
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

pub(super) fn lower_symbol<F>(raw: &str, resolve_symbol: &mut F) -> Expr
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Expr::Atom(Atom::Symbol(
        resolve_symbol(raw).unwrap_or_else(|| symbol_from_raw_name(raw)),
    ))
}
