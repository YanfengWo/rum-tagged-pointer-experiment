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
    match expr {
        SyntaxExpr::Integer(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            Ok(Expr::integer(bigint))
        }
        SyntaxExpr::String(value) => Ok(Expr::string(value.clone())),
        SyntaxExpr::Symbol(value) => Ok(lower_symbol(value, resolve_symbol)),
        SyntaxExpr::List(items) => {
            let lowered_items = items
                .iter()
                .map(|item| lower_syntax_expr_with(item, resolve_symbol))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::call(system_expr("List"), lowered_items))
        }
        SyntaxExpr::Slot(spec) => lower_slot(spec),
        SyntaxExpr::Blank { kind, head } => lower_blank(*kind, head.as_deref(), resolve_symbol),
        SyntaxExpr::Pattern { name, pattern } => Ok(Expr::call(
            system_expr("Pattern"),
            vec![
                lower_syntax_expr_with(name, resolve_symbol)?,
                lower_syntax_expr_with(pattern, resolve_symbol)?,
            ],
        )),
        SyntaxExpr::Prefix { op, expr } => lower_prefix(*op, expr, resolve_symbol),
        SyntaxExpr::Binary { op, lhs, rhs } => lower_binary(*op, lhs, rhs, resolve_symbol),
        SyntaxExpr::Postfix { op, expr } => lower_postfix(*op, expr, resolve_symbol),
        SyntaxExpr::Call { head, args } => {
            let lowered_head = lower_syntax_expr_with(head, resolve_symbol)?;
            let lowered_args = args
                .iter()
                .map(|arg| lower_syntax_expr_with(arg, resolve_symbol))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::call(lowered_head, lowered_args))
        }
    }
}

pub(super) fn lower_slot(spec: &SlotSpec) -> Result<Expr, EvalError> {
    let (head, arg) = match spec {
        SlotSpec::Anonymous => ("Slot", Expr::integer(BigInt::from(1_u8))),
        SlotSpec::Number(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            ("Slot", Expr::integer(bigint))
        }
        SlotSpec::Named(value) => ("Slot", Expr::string(value.clone())),
        SlotSpec::Sequence => ("SlotSequence", Expr::integer(BigInt::from(1_u8))),
        SlotSpec::SequenceNumber(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            ("SlotSequence", Expr::integer(bigint))
        }
    };

    Ok(Expr::call(system_expr(head), vec![arg]))
}

pub(super) fn lower_blank<F>(
    kind: BlankKind,
    head: Option<&SyntaxExpr>,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let head_symbol = match kind {
        BlankKind::Blank => "Blank",
        BlankKind::BlankSequence => "BlankSequence",
        BlankKind::BlankNullSequence => "BlankNullSequence",
    };

    let args = match head {
        Some(head) => vec![lower_syntax_expr_with(head, resolve_symbol)?],
        None => Vec::new(),
    };

    Ok(Expr::call(system_expr(head_symbol), args))
}

pub(super) fn lower_prefix<F>(
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
        PrefixOp::Minus => Ok(Expr::call(
            system_expr("Times"),
            vec![Expr::integer(BigInt::from(-1)), lowered],
        )),
    }
}

pub(super) fn lower_binary<F>(
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
        BinaryOp::MessageName => Ok(message_name_expr(
            match lowered_lhs {
                Expr::Atom(Atom::Symbol(symbol)) => symbol,
                other => {
                    return Ok(Expr::call(
                        system_expr("MessageName"),
                        vec![other, lower_message_tag_expr(rhs, resolve_symbol)?],
                    ));
                }
            },
            lower_message_tag_string(rhs),
        )),
        BinaryOp::TagSet => lower_tag_set_expr(lowered_lhs, lowered_rhs),
        BinaryOp::Set => Ok(Expr::call(
            system_expr("Set"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::SetDelayed => Ok(Expr::call(
            system_expr("SetDelayed"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceAll => Ok(Expr::call(
            system_expr("ReplaceAll"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceRepeated => Ok(Expr::call(
            system_expr("ReplaceRepeated"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Rule => Ok(Expr::call(
            system_expr("Rule"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::RuleDelayed => Ok(Expr::call(
            system_expr("RuleDelayed"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Condition => Ok(Expr::call(
            system_expr("Condition"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Add => Ok(Expr::call(
            system_expr("Plus"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Subtract => Ok(Expr::call(
            system_expr("Plus"),
            vec![
                lowered_lhs,
                Expr::call(
                    system_expr("Times"),
                    vec![Expr::integer(BigInt::from(-1)), lowered_rhs],
                ),
            ],
        )),
        BinaryOp::Times => Ok(Expr::call(
            system_expr("Times"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Divide => Ok(Expr::call(
            system_expr("Times"),
            vec![
                lowered_lhs,
                Expr::call(
                    system_expr("Power"),
                    vec![lowered_rhs, Expr::integer(BigInt::from(-1))],
                ),
            ],
        )),
        BinaryOp::Power => Ok(Expr::call(
            system_expr("Power"),
            vec![lowered_lhs, lowered_rhs],
        )),
    }
}

pub(super) fn lower_message_tag_expr<F>(
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    match expr {
        SyntaxExpr::Symbol(value) => Ok(Expr::string(value.clone())),
        SyntaxExpr::String(value) => Ok(Expr::string(value.clone())),
        _ => lower_syntax_expr_with(expr, resolve_symbol),
    }
}

pub(super) fn lower_message_tag_string(expr: &SyntaxExpr) -> String {
    match expr {
        SyntaxExpr::Symbol(value) | SyntaxExpr::String(value) => value.clone(),
        other => other.to_string(),
    }
}

pub(super) fn lower_postfix<F>(
    op: PostfixOp,
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered = lower_syntax_expr_with(expr, resolve_symbol)?;

    match op {
        PostfixOp::Function => Ok(Expr::call(system_expr("Function"), vec![lowered])),
        PostfixOp::Unset => Ok(Expr::call(system_expr("Unset"), vec![lowered])),
    }
}

pub(super) fn lower_tag_set_expr(tag: Expr, rhs: Expr) -> Result<Expr, EvalError> {
    match rhs {
        Expr::Call { head, args } if is_system_symbol_name(&head, "Set") && args.len() == 2 => {
            Ok(Expr::call(
                system_expr("TagSet"),
                vec![tag, args[0].clone(), args[1].clone()],
            ))
        }
        Expr::Call { head, args }
            if is_system_symbol_name(&head, "SetDelayed") && args.len() == 2 =>
        {
            Ok(Expr::call(
                system_expr("TagSetDelayed"),
                vec![tag, args[0].clone(), args[1].clone()],
            ))
        }
        Expr::Call { head, args } if is_system_symbol_name(&head, "Unset") && args.len() == 1 => {
            Ok(Expr::call(
                system_expr("TagUnset"),
                vec![tag, args[0].clone()],
            ))
        }
        _ => Ok(Expr::call(system_expr("TagSet"), vec![tag, rhs])),
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
