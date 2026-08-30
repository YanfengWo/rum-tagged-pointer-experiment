use super::*;

mod binary;

use binary::lower_binary_sugar;

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
        SyntaxExprKind::ComparisonChain {
            operands,
            operators,
        } => binary::lower_comparison_chain(operands, operators, expr.span(), resolve_symbol),
        SyntaxExprKind::Postfix { op, expr } => lower_postfix_sugar(*op, expr, resolve_symbol),
        SyntaxExprKind::Call { head, args } => {
            lower_head_preserving_call(head, args, resolve_symbol)
        }
    }
}

fn lower_integer_atom(value: &str) -> Result<Expr, EvalError> {
    if let Some(rational) = lower_decimal_atom(value)? {
        return Ok(rational);
    }

    let bigint =
        BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| EvalError::InvalidInteger {
            literal: value.to_owned(),
        })?;

    Ok(Expr::integer(bigint))
}

fn lower_decimal_atom(value: &str) -> Result<Option<Expr>, EvalError> {
    let (mantissa, exponent) = split_decimal_exponent(value)?;
    let Some((whole, fractional)) = mantissa.split_once('.') else {
        return Ok(None);
    };
    if (whole.is_empty() && fractional.is_empty())
        || !whole.chars().all(|ch| ch.is_ascii_digit())
        || !fractional.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(EvalError::InvalidInteger {
            literal: value.to_owned(),
        });
    }

    let whole = if whole.is_empty() { "0" } else { whole };
    let numerator_text = format!("{whole}{fractional}");
    let mut numerator = BigInt::parse_bytes(numerator_text.as_bytes(), 10).ok_or_else(|| {
        EvalError::InvalidInteger {
            literal: value.to_owned(),
        }
    })?;
    let mut denominator = BigInt::from(1_u8);
    for _ in fractional.chars() {
        denominator *= BigInt::from(10_u8);
    }

    if let Some(exponent) = exponent {
        if exponent >= 0 {
            multiply_by_power_of_ten(&mut numerator, exponent as u64);
        } else {
            multiply_by_power_of_ten(&mut denominator, exponent.unsigned_abs());
        }
    }

    Ok(Some(Expr::rational(
        Rational::new(numerator, denominator)
            .expect("decimal literal denominator is always positive"),
    )))
}

fn split_decimal_exponent(value: &str) -> Result<(&str, Option<i64>), EvalError> {
    let Some(exponent_index) = value.find(|ch| matches!(ch, 'e' | 'E')) else {
        return Ok((value, None));
    };

    let exponent =
        value[exponent_index + 1..]
            .parse::<i64>()
            .map_err(|_| EvalError::InvalidInteger {
                literal: value.to_owned(),
            })?;

    Ok((&value[..exponent_index], Some(exponent)))
}

fn multiply_by_power_of_ten(value: &mut BigInt, exponent: u64) {
    for _ in 0..exponent {
        *value *= BigInt::from(10_u8);
    }
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

fn negated_numeric_literal(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Expr::integer(-value.to_bigint())),
        Expr::Atom(Atom::Rational(value)) => Some(Expr::rational(value.negated())),
        _ => None,
    }
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
        PrefixOp::Minus => {
            if let Some(negated) = negated_numeric_literal(&lowered) {
                Ok(negated)
            } else {
                Ok(canonical_call(
                    heads::arithmetic::TIMES,
                    vec![Expr::integer(BigInt::from(-1)), lowered],
                ))
            }
        }
        PrefixOp::Not => Ok(canonical_call("Not", vec![lowered])),
        PrefixOp::PreIncrement => Ok(canonical_call("PreIncrement", vec![lowered])),
        PrefixOp::PreDecrement => Ok(canonical_call("PreDecrement", vec![lowered])),
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
        PostfixOp::Increment => Ok(canonical_call("Increment", vec![lowered])),
        PostfixOp::Decrement => Ok(canonical_call("Decrement", vec![lowered])),
        PostfixOp::Factorial => Ok(canonical_call("Factorial", vec![lowered])),
        PostfixOp::Factorial2 => Ok(canonical_call("Factorial2", vec![lowered])),
        PostfixOp::Derivative(order) => Ok(Expr::call(
            canonical_call("Derivative", vec![Expr::integer(BigInt::from(order))]),
            vec![lowered],
        )),
        PostfixOp::Optional => Ok(canonical_call("Optional", vec![lowered])),
        PostfixOp::Repeated => Ok(canonical_call("Repeated", vec![lowered])),
        PostfixOp::RepeatedNull => Ok(canonical_call("RepeatedNull", vec![lowered])),
        PostfixOp::Unset => Ok(canonical_call(heads::rules::UNSET, vec![lowered])),
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
