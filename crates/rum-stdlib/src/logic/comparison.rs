use super::shared::predicate_bool_expr;
use rum_core::SharedArgs;
use rum_core::{Atom, Expr, Rational, heads};
use rum_runtime::kernel_api::{EvalError, Session, system_expr, truthiness_symbol};
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonRelation {
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ComparableValue {
    Number(Rational),
    String(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum EqualityKnownValue {
    Comparable(ComparableValue),
    Boolean(bool),
}

pub(crate) fn eval_equal(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(eval_comparison_subset(
        session,
        head,
        args,
        ComparisonRelation::Equal,
    ))
}

pub(crate) fn eval_unequal(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(eval_comparison_subset(
        session,
        head,
        args,
        ComparisonRelation::Unequal,
    ))
}

pub(crate) fn eval_less(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(eval_comparison_subset(
        session,
        head,
        args,
        ComparisonRelation::Less,
    ))
}

pub(crate) fn eval_greater(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(eval_comparison_subset(
        session,
        head,
        args,
        ComparisonRelation::Greater,
    ))
}

pub(crate) fn eval_less_equal(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(eval_comparison_subset(
        session,
        head,
        args,
        ComparisonRelation::LessEqual,
    ))
}

pub(crate) fn eval_greater_equal(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(eval_comparison_subset(
        session,
        head,
        args,
        ComparisonRelation::GreaterEqual,
    ))
}

pub(crate) fn eval_comparison_operator_wrapper(
    _session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    Ok(Expr::call(head.clone(), args.to_vec()))
}

fn eval_comparison_subset(
    _session: &mut Session,
    head: &Expr,
    args: &[Expr],
    relation: ComparisonRelation,
) -> Expr {
    if args.len() <= 1 {
        return predicate_bool_expr(true);
    }

    if relation == ComparisonRelation::Equal {
        if let Some(result) = reduce_equal_prefix_chain(args) {
            return predicate_bool_expr(result);
        }
    } else if relation == ComparisonRelation::Unequal {
        if let Some(result) = reduce_unequal_chain(args) {
            return predicate_bool_expr(result);
        }
    } else if let Some(result) = comparison_result_from_expr_identity(args, relation) {
        return predicate_bool_expr(result);
    }

    if matches!(
        relation,
        ComparisonRelation::Less
            | ComparisonRelation::Greater
            | ComparisonRelation::LessEqual
            | ComparisonRelation::GreaterEqual
    ) {
        if known_order_chain_contradiction(args, relation) {
            return predicate_bool_expr(false);
        }
        if let Some(result) = reduce_order_chain(head, args) {
            return result;
        }
    }

    let result = match relation {
        ComparisonRelation::Equal | ComparisonRelation::Unequal => args
            .iter()
            .map(comparable_value_from_expr)
            .collect::<Option<Vec<_>>>()
            .map(|values| compare_equality_values(&values, relation)),
        ComparisonRelation::Less
        | ComparisonRelation::Greater
        | ComparisonRelation::LessEqual
        | ComparisonRelation::GreaterEqual => args
            .iter()
            .map(numeric_comparable_value_from_expr)
            .collect::<Option<Vec<_>>>()
            .map(|values| compare_numeric_values(&values, relation)),
    };

    match result {
        Some(result) => predicate_bool_expr(result),
        None => Expr::call(head.clone(), args.to_vec()),
    }
}

fn reduce_equal_prefix_chain(args: &[Expr]) -> Option<bool> {
    let mut saw_known_pair = false;

    for pair in args.windows(2) {
        match adjacent_equal_truth(&pair[0], &pair[1]) {
            Some(true) => saw_known_pair = true,
            Some(false) => return Some(false),
            None => return None,
        }
    }

    saw_known_pair.then_some(true)
}

fn adjacent_equal_truth(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    if lhs == rhs {
        return Some(true);
    }

    match (truthiness_symbol(lhs), truthiness_symbol(rhs)) {
        (Some(lhs), Some(rhs)) => return Some(lhs == rhs),
        (Some(_), None) | (None, Some(_)) => return None,
        (None, None) => {}
    }

    let lhs = comparable_value_from_expr(lhs)?;
    let rhs = comparable_value_from_expr(rhs)?;
    Some(lhs == rhs)
}

fn reduce_unequal_chain(args: &[Expr]) -> Option<bool> {
    for (index, pair) in args.windows(2).enumerate() {
        match adjacent_unequal_truth(&pair[0], &pair[1]) {
            Some(true) => continue,
            Some(false) => return Some(false),
            None => {
                return known_equality_prefix_has_duplicate(&args[..=index]).then_some(false);
            }
        }
    }

    args.iter()
        .map(equality_known_value_from_expr)
        .collect::<Option<Vec<_>>>()
        .map(|values| !equality_values_have_duplicate(&values))
}

fn adjacent_unequal_truth(lhs: &Expr, rhs: &Expr) -> Option<bool> {
    if lhs == rhs {
        return Some(false);
    }

    match (truthiness_symbol(lhs), truthiness_symbol(rhs)) {
        (Some(lhs), Some(rhs)) => return Some(lhs != rhs),
        (Some(_), None) | (None, Some(_)) => return None,
        (None, None) => {}
    }

    let lhs = comparable_value_from_expr(lhs)?;
    let rhs = comparable_value_from_expr(rhs)?;
    Some(lhs != rhs)
}

fn known_equality_prefix_has_duplicate(args: &[Expr]) -> bool {
    let Some(values) = args
        .iter()
        .map(equality_known_value_from_expr)
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };

    equality_values_have_duplicate(&values)
}

fn equality_values_have_duplicate(values: &[EqualityKnownValue]) -> bool {
    values_have_duplicate(values)
}

fn reduce_order_chain(head: &Expr, args: &[Expr]) -> Option<Expr> {
    if args.len() <= 2 {
        return None;
    }

    let numeric_prefix_len = args
        .iter()
        .take_while(|arg| is_numeric_comparable_expr(arg))
        .count();
    if numeric_prefix_len == args.len() {
        return Some(system_expr(heads::structure::TRUE));
    }

    let numeric_suffix_len = args
        .iter()
        .rev()
        .take_while(|arg| is_numeric_comparable_expr(arg))
        .count();
    if numeric_prefix_len < 2 && numeric_suffix_len < 2 {
        return None;
    }

    let start = numeric_prefix_len.saturating_sub(1);
    let end = if numeric_suffix_len >= 2 {
        args.len() - numeric_suffix_len + 1
    } else {
        args.len()
    };
    Some(Expr::call(head.clone(), args[start..end].to_vec()))
}

fn known_order_chain_contradiction(args: &[Expr], relation: ComparisonRelation) -> bool {
    let mut previous = None;
    for expr in args {
        let Some(current) = numeric_comparable_value_from_expr(expr) else {
            continue;
        };

        if let Some(previous) = &previous {
            let ordering = compare_rationals(previous, &current);
            if !ordering_satisfies_relation(ordering, relation) {
                return true;
            }
        }
        previous = Some(current);
    }

    false
}

fn numeric_comparable_value_from_expr(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.to_bigint())),
        Expr::Atom(Atom::Rational(value)) => Some(value.as_ref().clone()),
        _ => None,
    }
}

fn is_numeric_comparable_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Atom(Atom::Integer(_) | Atom::Rational(_)))
}

fn ordering_satisfies_relation(ordering: std::cmp::Ordering, relation: ComparisonRelation) -> bool {
    match relation {
        ComparisonRelation::Less => ordering == std::cmp::Ordering::Less,
        ComparisonRelation::Greater => ordering == std::cmp::Ordering::Greater,
        ComparisonRelation::LessEqual => ordering != std::cmp::Ordering::Greater,
        ComparisonRelation::GreaterEqual => ordering != std::cmp::Ordering::Less,
        ComparisonRelation::Equal | ComparisonRelation::Unequal => false,
    }
}

fn comparison_result_from_expr_identity(
    args: &[Expr],
    relation: ComparisonRelation,
) -> Option<bool> {
    match relation {
        ComparisonRelation::Equal => args
            .windows(2)
            .all(|pair| pair[0] == pair[1])
            .then_some(true),
        ComparisonRelation::Unequal => None,
        ComparisonRelation::Less
        | ComparisonRelation::Greater
        | ComparisonRelation::LessEqual
        | ComparisonRelation::GreaterEqual => None,
    }
}

fn comparable_value_from_expr(expr: &Expr) -> Option<ComparableValue> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(ComparableValue::Number(Rational::from_integer(
            value.to_bigint(),
        ))),
        Expr::Atom(Atom::Rational(value)) => Some(ComparableValue::Number(value.as_ref().clone())),
        Expr::Atom(Atom::String(value)) => Some(ComparableValue::String(value.clone())),
        _ => None,
    }
}

fn equality_known_value_from_expr(expr: &Expr) -> Option<EqualityKnownValue> {
    if let Some(value) = truthiness_symbol(expr) {
        return Some(EqualityKnownValue::Boolean(value));
    }

    comparable_value_from_expr(expr).map(EqualityKnownValue::Comparable)
}

fn compare_equality_values(values: &[ComparableValue], relation: ComparisonRelation) -> bool {
    match relation {
        ComparisonRelation::Equal => values.windows(2).all(|pair| pair[0] == pair[1]),
        ComparisonRelation::Unequal => !values_have_duplicate(values),
        ComparisonRelation::Less
        | ComparisonRelation::Greater
        | ComparisonRelation::LessEqual
        | ComparisonRelation::GreaterEqual => {
            unreachable!("order relations use numeric comparison")
        }
    }
}

fn values_have_duplicate<T: Eq + Hash>(values: &[T]) -> bool {
    const INDEX_THRESHOLD: usize = 16;

    if values.len() <= INDEX_THRESHOLD {
        return values
            .iter()
            .enumerate()
            .any(|(index, lhs)| values[index + 1..].iter().any(|rhs| lhs == rhs));
    }

    let mut seen = HashSet::with_capacity(values.len());
    values.iter().any(|value| !seen.insert(value))
}

fn compare_numeric_values(values: &[Rational], relation: ComparisonRelation) -> bool {
    values
        .windows(2)
        .all(|pair| ordering_satisfies_relation(compare_rationals(&pair[0], &pair[1]), relation))
}

fn compare_rationals(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}
