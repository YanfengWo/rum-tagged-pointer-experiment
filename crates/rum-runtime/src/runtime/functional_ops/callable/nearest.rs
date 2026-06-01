use super::*;

#[derive(Clone, Debug)]
struct NearestToSpec<'a> {
    target: &'a Expr,
    selection: Option<&'a Expr>,
}

pub(super) fn is_nearest_to_wrapper(expr: &Expr) -> bool {
    parse_nearest_to_wrapper(expr).is_some()
}

pub(super) fn apply_nearest_to_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_nearest_to_wrapper(head)?;
    let [data] = args else {
        return None;
    };

    let mut nearest_args = vec![data.clone(), spec.target.clone()];
    if let Some(selection) = spec.selection {
        nearest_args.push(selection.clone());
    }
    Some(Expr::call(system_expr("Nearest"), nearest_args))
}

fn parse_nearest_to_wrapper(expr: &Expr) -> Option<NearestToSpec<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "NearestTo") {
        return None;
    }

    match args.as_slice() {
        [target] => Some(NearestToSpec {
            target,
            selection: None,
        }),
        [target, selection] if is_nearest_to_selection_spec(selection) => Some(NearestToSpec {
            target,
            selection: Some(selection),
        }),
        _ => None,
    }
}

fn is_nearest_to_selection_spec(expr: &Expr) -> bool {
    if expr_as_positive_usize(expr).is_some() {
        return true;
    }

    let Expr::Call { head, args } = expr else {
        return false;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return false;
    }
    let [count, radius] = args.as_slice() else {
        return false;
    };

    (is_system_symbol_name(count, heads::structure::ALL) || expr_as_positive_usize(count).is_some())
        && is_non_negative_nearest_to_radius(radius)
}

fn is_non_negative_nearest_to_radius(expr: &Expr) -> bool {
    if is_system_symbol_name(expr, heads::structure::INFINITY) {
        return true;
    }
    expr_as_exact_rational(expr).is_some_and(|value| {
        compare_rationals_for_nearest_to(&value, &Rational::zero()) != std::cmp::Ordering::Less
    })
}

fn compare_rationals_for_nearest_to(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}
