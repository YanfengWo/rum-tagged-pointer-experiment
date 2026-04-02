use super::*;
pub(super) fn parse_string_rule_dataset(expr: &Expr) -> Option<(Vec<String>, Vec<Expr>)> {
    let (keys, values) = parse_nearest_rule_pairs(expr)?;
    Some((
        keys.iter()
            .map(expr_as_string)
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .map(str::to_string)
            .collect(),
        values,
    ))
}

pub(super) fn parse_numeric_dataset(expr: &Expr) -> Option<(Vec<Rational>, Vec<Expr>)> {
    if let Some(items) = call_items_with_head(expr, &system_expr("List")) {
        let keys = items
            .iter()
            .map(expr_as_exact_rational)
            .collect::<Option<Vec<_>>>()?;
        return Some((keys, items.to_vec()));
    }

    let (keys, values) = parse_nearest_rule_pairs(expr)?;
    Some((
        keys.iter()
            .map(expr_as_exact_rational)
            .collect::<Option<Vec<_>>>()?,
        values,
    ))
}

pub(super) fn parse_nearest_rule_pairs(expr: &Expr) -> Option<(Vec<Expr>, Vec<Expr>)> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Rule") && args.len() == 2 => {
            let keys = call_items_with_head(&args[0], &system_expr("List"))?.to_vec();
            let values = call_items_with_head(&args[1], &system_expr("List"))?.to_vec();
            (keys.len() == values.len()).then_some((keys, values))
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut keys = Vec::with_capacity(args.len());
            let mut values = Vec::with_capacity(args.len());
            for item in args {
                let Expr::Call {
                    head: rule_head,
                    args: rule_args,
                } = item
                else {
                    return None;
                };
                if !is_system_symbol_name(rule_head, "Rule") || rule_args.len() != 2 {
                    return None;
                }
                keys.push(rule_args[0].clone());
                values.push(rule_args[1].clone());
            }
            Some((keys, values))
        }
        _ => None,
    }
}

pub(super) fn parse_nearest_result_spec(expr: &Expr) -> Option<NearestResultSpec> {
    if is_system_symbol_name(expr, heads::structure::ALL) {
        return Some(NearestResultSpec::All);
    }

    expr_as_non_negative_usize(expr).map(NearestResultSpec::Count)
}

pub(super) fn nearest_strings(
    values: &[String],
    target: &str,
    spec: NearestResultSpec,
) -> Vec<String> {
    let target_chars = target.chars().collect::<Vec<_>>();
    let distances = values
        .iter()
        .map(|value| {
            let value_chars = value.chars().collect::<Vec<_>>();
            levenshtein_distance(&value_chars, &target_chars)
        })
        .collect::<Vec<_>>();
    select_nearest_indices_by_usize(&distances, spec)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

pub(super) fn nearest_values_for_strings(
    keys: &[String],
    values: &[Expr],
    target: &str,
    spec: NearestResultSpec,
) -> Vec<Expr> {
    let target_chars = target.chars().collect::<Vec<_>>();
    let distances = keys
        .iter()
        .map(|value| {
            let value_chars = value.chars().collect::<Vec<_>>();
            levenshtein_distance(&value_chars, &target_chars)
        })
        .collect::<Vec<_>>();
    select_nearest_indices_by_usize(&distances, spec)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

pub(super) fn nearest_values_for_numbers(
    keys: &[Rational],
    values: &[Expr],
    target: &Rational,
    spec: NearestResultSpec,
) -> Vec<Expr> {
    let distances = keys
        .iter()
        .map(|value| rational_abs_difference(value, target))
        .collect::<Vec<_>>();
    select_nearest_indices_by_rational(&distances, spec)
        .into_iter()
        .map(|index| values[index].clone())
        .collect()
}

pub(super) fn select_nearest_indices_by_usize(
    distances: &[usize],
    spec: NearestResultSpec,
) -> Vec<usize> {
    match spec {
        NearestResultSpec::Count(0) => Vec::new(),
        NearestResultSpec::Count(limit) => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by_key(|index| (distances[*index], *index));
            indices.truncate(limit.min(indices.len()));
            indices
        }
        NearestResultSpec::All => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by_key(|index| (distances[*index], *index));
            indices
        }
        NearestResultSpec::BestTies => {
            let Some(best) = distances.iter().copied().min() else {
                return Vec::new();
            };
            distances
                .iter()
                .enumerate()
                .filter_map(|(index, distance)| (*distance == best).then_some(index))
                .collect()
        }
    }
}

pub(super) fn select_nearest_indices_by_rational(
    distances: &[Rational],
    spec: NearestResultSpec,
) -> Vec<usize> {
    match spec {
        NearestResultSpec::Count(0) => Vec::new(),
        NearestResultSpec::Count(limit) => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by(|lhs, rhs| {
                compare_rationals_local(&distances[*lhs], &distances[*rhs]).then(lhs.cmp(rhs))
            });
            indices.truncate(limit.min(indices.len()));
            indices
        }
        NearestResultSpec::All => {
            let mut indices = (0..distances.len()).collect::<Vec<_>>();
            indices.sort_by(|lhs, rhs| {
                compare_rationals_local(&distances[*lhs], &distances[*rhs]).then(lhs.cmp(rhs))
            });
            indices
        }
        NearestResultSpec::BestTies => {
            let Some((best_index, _)) = distances
                .iter()
                .enumerate()
                .min_by(|(_, lhs), (_, rhs)| compare_rationals_local(lhs, rhs))
            else {
                return Vec::new();
            };
            distances
                .iter()
                .enumerate()
                .filter_map(|(index, distance)| {
                    (compare_rationals_local(distance, &distances[best_index])
                        == std::cmp::Ordering::Equal)
                        .then_some(index)
                })
                .collect()
        }
    }
}

pub(super) fn rational_abs_difference(lhs: &Rational, rhs: &Rational) -> Rational {
    let difference = lhs.sub(rhs);
    if compare_rationals_local(&difference, &Rational::zero()) == std::cmp::Ordering::Less {
        difference.negated()
    } else {
        difference
    }
}

pub(super) fn compare_rationals_local(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}
