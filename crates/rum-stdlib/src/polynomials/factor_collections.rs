use std::collections::HashMap;

use super::core::Polynomial;

const FACTOR_GROUP_INDEX_THRESHOLD: usize = 32;
const FACTOR_CANCEL_SIDE_THRESHOLD: usize = 8;
const FACTOR_CANCEL_WORK_THRESHOLD: usize = 256;

pub(super) fn stable_factor_counts(factors: &[Polynomial]) -> Vec<(&Polynomial, usize)> {
    if factors.len() < FACTOR_GROUP_INDEX_THRESHOLD {
        return linear_factor_counts(factors);
    }

    let mut grouped: Vec<(&Polynomial, usize)> = Vec::new();
    let mut indices: HashMap<&Polynomial, usize> = HashMap::with_capacity(factors.len());
    for factor in factors {
        if let Some(index) = indices.get(factor).copied() {
            grouped[index].1 += 1;
        } else {
            let index = grouped.len();
            indices.insert(factor, index);
            grouped.push((factor, 1));
        }
    }
    grouped
}

fn linear_factor_counts(factors: &[Polynomial]) -> Vec<(&Polynomial, usize)> {
    let mut grouped: Vec<(&Polynomial, usize)> = Vec::new();
    for factor in factors {
        if let Some((_, count)) = grouped.iter_mut().find(|(existing, _)| *existing == factor) {
            *count += 1;
        } else {
            grouped.push((factor, 1));
        }
    }
    grouped
}

pub(super) fn cancel_common_factor_multisets(
    numerator: Vec<Polynomial>,
    denominator: Vec<Polynomial>,
) -> (Vec<Polynomial>, Vec<Polynomial>) {
    if numerator.len() < FACTOR_CANCEL_SIDE_THRESHOLD
        || denominator.len() < FACTOR_CANCEL_SIDE_THRESHOLD
        || numerator.len().saturating_mul(denominator.len()) < FACTOR_CANCEL_WORK_THRESHOLD
    {
        return cancel_common_factors_linear(numerator, denominator);
    }

    cancel_common_factors_indexed(numerator, denominator)
}

fn cancel_common_factors_linear(
    mut numerator: Vec<Polynomial>,
    mut denominator: Vec<Polynomial>,
) -> (Vec<Polynomial>, Vec<Polynomial>) {
    let mut index = 0;
    while index < numerator.len() {
        if let Some(position) = denominator
            .iter()
            .position(|factor| factor == &numerator[index])
        {
            numerator.remove(index);
            denominator.remove(position);
        } else {
            index += 1;
        }
    }
    (numerator, denominator)
}

fn cancel_common_factors_indexed(
    numerator: Vec<Polynomial>,
    denominator: Vec<Polynomial>,
) -> (Vec<Polynomial>, Vec<Polynomial>) {
    let mut denominator_counts = HashMap::with_capacity(denominator.len());
    for factor in &denominator {
        *denominator_counts.entry(factor).or_insert(0_usize) += 1;
    }

    let mut remove_numerator = vec![false; numerator.len()];
    for (index, factor) in numerator.iter().enumerate() {
        let Some(count) = denominator_counts.get_mut(factor) else {
            continue;
        };
        if *count > 0 {
            *count -= 1;
            remove_numerator[index] = true;
        }
    }

    let mut keep_denominator = vec![false; denominator.len()];
    for (index, factor) in denominator.iter().enumerate() {
        let count = denominator_counts
            .get_mut(factor)
            .expect("every denominator factor should have a count");
        if *count > 0 {
            *count -= 1;
            keep_denominator[index] = true;
        }
    }
    drop(denominator_counts);

    let numerator = numerator
        .into_iter()
        .zip(remove_numerator)
        .filter_map(|(factor, remove)| (!remove).then_some(factor))
        .collect();
    let denominator = denominator
        .into_iter()
        .zip(keep_denominator)
        .filter_map(|(factor, keep)| keep.then_some(factor))
        .collect();
    (numerator, denominator)
}

#[cfg(test)]
mod tests {
    use rum_runtime::kernel_api::system_expr;

    use super::*;

    #[test]
    fn indexed_factor_collections_preserve_order_and_multiplicity() {
        let catalog = (0..40).map(test_factor).collect::<Vec<_>>();
        let mut grouped_input = catalog.clone();
        grouped_input.push(catalog[5].clone());
        grouped_input.push(catalog[5].clone());
        grouped_input.push(catalog[17].clone());

        let grouped = stable_factor_counts(&grouped_input);
        assert_eq!(grouped.len(), 40);
        assert_eq!(grouped[5], (&catalog[5], 3));
        assert_eq!(grouped[17], (&catalog[17], 2));

        let mut numerator = catalog[..20].to_vec();
        numerator.push(catalog[5].clone());
        numerator.push(catalog[5].clone());
        let mut denominator = catalog[10..30].to_vec();
        denominator.push(catalog[5].clone());

        let (numerator, denominator) = cancel_common_factor_multisets(numerator, denominator);
        let mut expected_numerator = catalog[..5].to_vec();
        expected_numerator.extend_from_slice(&catalog[6..10]);
        expected_numerator.push(catalog[5].clone());
        expected_numerator.push(catalog[5].clone());

        assert_eq!(numerator, expected_numerator);
        assert_eq!(denominator, catalog[20..30]);
    }

    fn test_factor(index: usize) -> Polynomial {
        Polynomial::indeterminate(system_expr(&format!("factor{index}")))
    }
}
