use super::*;

mod arrays;
mod basic;
mod collections;
mod numeric;
mod ordering;
mod truth;

fn exact_numeric_value(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.clone())),
        Expr::Atom(Atom::Rational(value)) => Some(value.as_ref().clone()),
        _ => None,
    }
}

fn string_atom_value(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Atom(Atom::String(value)) => Some(value),
        _ => None,
    }
}

fn exact_integer_value(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.clone()),
        _ => None,
    }
}

fn expr_has_system_head(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Call { head, .. } => is_system_symbol_name(head, name),
        Expr::Atom(_) => false,
    }
}

fn list_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

fn two_list_args(args: &[Expr]) -> Option<(&[Expr], &[Expr])> {
    let [lhs, rhs] = args else {
        return None;
    };

    Some((list_items(lhs)?, list_items(rhs)?))
}

fn expr_set(items: &[Expr]) -> std::collections::HashSet<Expr> {
    items.iter().cloned().collect()
}

fn expr_counts(items: &[Expr]) -> std::collections::HashMap<Expr, usize> {
    let mut counts = std::collections::HashMap::new();

    for item in items {
        *counts.entry(item.clone()).or_insert(0) += 1;
    }

    counts
}

fn array_shape(expr: &Expr) -> Option<Vec<usize>> {
    let items = list_items(expr)?;
    list_shape(items)
}

fn list_shape(items: &[Expr]) -> Option<Vec<usize>> {
    if items.is_empty() {
        return Some(vec![0]);
    }

    let child_shape = element_shape(&items[0])?;
    for item in &items[1..] {
        if element_shape(item)? != child_shape {
            return None;
        }
    }

    let mut shape = vec![items.len()];
    shape.extend(child_shape);
    Some(shape)
}

fn element_shape(expr: &Expr) -> Option<Vec<usize>> {
    match list_items(expr) {
        Some(items) => list_shape(items),
        None => Some(Vec::new()),
    }
}

fn compare_exact_numeric(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn compare_exact_order_subset(lhs: &Expr, rhs: &Expr) -> std::cmp::Ordering {
    match (exact_numeric_value(lhs), exact_numeric_value(rhs)) {
        (Some(lhs), Some(rhs)) => compare_exact_numeric(&lhs, &rhs),
        _ => compare_expr_canonical(lhs, rhs),
    }
}

fn compare_lexicographic_order_subset(lhs: &Expr, rhs: &Expr) -> std::cmp::Ordering {
    match (list_items(lhs), list_items(rhs)) {
        (Some(lhs), Some(rhs)) => compare_lexicographic_slices(lhs, rhs),
        _ => match (string_atom_value(lhs), string_atom_value(rhs)) {
            (Some(lhs), Some(rhs)) => lhs.cmp(rhs),
            _ => compare_exact_order_subset(lhs, rhs),
        },
    }
}

fn compare_lexicographic_slices(lhs: &[Expr], rhs: &[Expr]) -> std::cmp::Ordering {
    for (lhs_item, rhs_item) in lhs.iter().zip(rhs) {
        let ordering = compare_lexicographic_order_subset(lhs_item, rhs_item);
        if ordering != std::cmp::Ordering::Equal {
            return ordering;
        }
    }

    lhs.len().cmp(&rhs.len())
}

fn order_result_expr(ordering: std::cmp::Ordering) -> Expr {
    let value = match ordering {
        std::cmp::Ordering::Less => 1_i64,
        std::cmp::Ordering::Equal => 0_i64,
        std::cmp::Ordering::Greater => -1_i64,
    };

    Expr::integer(BigInt::from(value))
}

fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}
