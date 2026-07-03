use super::shared::predicate_bool_expr;
use crate::runtime::kernel_api::{EvalError, Session, system_expr, truthiness_symbol};
use rum_core::{Atom, Expr, Rational, heads};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonRelation {
    Equal,
    Unequal,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ComparableValue {
    Number(Rational),
    String(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EqualityKnownValue {
    Comparable(ComparableValue),
    Boolean(bool),
}

impl Session {
    pub(in crate::runtime) fn eval_equal(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::Equal))
    }

    pub(in crate::runtime) fn eval_unequal(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::Unequal))
    }

    pub(in crate::runtime) fn eval_less(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::Less))
    }

    pub(in crate::runtime) fn eval_greater(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::Greater))
    }

    pub(in crate::runtime) fn eval_less_equal(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::LessEqual))
    }

    pub(in crate::runtime) fn eval_greater_equal(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(self.eval_comparison_subset(head, args, ComparisonRelation::GreaterEqual))
    }

    pub(in crate::runtime) fn eval_comparison_operator_wrapper(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    fn eval_comparison_subset(
        &mut self,
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
            if let Some(result) = reduce_order_chain(head, args, relation) {
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
    values
        .iter()
        .enumerate()
        .any(|(index, lhs)| values[index + 1..].iter().any(|rhs| lhs == rhs))
}

fn reduce_order_chain(head: &Expr, args: &[Expr], relation: ComparisonRelation) -> Option<Expr> {
    if args.len() <= 2 {
        return None;
    }

    let mut kept = Vec::new();
    let mut had_known_true_pair = false;

    for (index, pair) in args.windows(2).enumerate() {
        let Some(ordering) = compare_numeric_expr_pair(&pair[0], &pair[1]) else {
            push_unique_expr(&mut kept, pair[0].clone());
            push_unique_expr(&mut kept, pair[1].clone());
            continue;
        };

        if !ordering_satisfies_relation(ordering, relation) {
            return Some(system_expr(heads::structure::FALSE));
        }

        had_known_true_pair = true;
        if index > 0 && index + 1 < args.len() {
            push_unique_expr(&mut kept, pair[0].clone());
        }
    }

    if !had_known_true_pair || kept.len() == args.len() {
        return None;
    }

    Some(match kept.len() {
        0 | 1 => system_expr(heads::structure::TRUE),
        _ => Expr::call(head.clone(), kept),
    })
}

fn known_order_chain_contradiction(args: &[Expr], relation: ComparisonRelation) -> bool {
    for (left_index, lhs) in args.iter().enumerate() {
        let Some(lhs) = numeric_comparable_value_from_expr(lhs) else {
            continue;
        };

        for rhs in args.iter().skip(left_index + 1) {
            let Some(rhs) = numeric_comparable_value_from_expr(rhs) else {
                continue;
            };
            let ordering = compare_rationals(&lhs, &rhs);
            if !ordering_satisfies_relation(ordering, relation) {
                return true;
            }
        }
    }

    false
}

fn compare_numeric_expr_pair(lhs: &Expr, rhs: &Expr) -> Option<std::cmp::Ordering> {
    let lhs = numeric_comparable_value_from_expr(lhs)?;
    let rhs = numeric_comparable_value_from_expr(rhs)?;
    Some(compare_rationals(&lhs, &rhs))
}

fn numeric_comparable_value_from_expr(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.to_bigint())),
        Expr::Atom(Atom::Rational(value)) => Some(value.as_ref().clone()),
        _ => None,
    }
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

fn push_unique_expr(values: &mut Vec<Expr>, value: Expr) {
    if values.last() != Some(&value) {
        values.push(value);
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
        ComparisonRelation::Unequal => !values
            .iter()
            .enumerate()
            .any(|(index, lhs)| values[index + 1..].iter().any(|rhs| lhs == rhs)),
        ComparisonRelation::Less
        | ComparisonRelation::Greater
        | ComparisonRelation::LessEqual
        | ComparisonRelation::GreaterEqual => {
            unreachable!("order relations use numeric comparison")
        }
    }
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
