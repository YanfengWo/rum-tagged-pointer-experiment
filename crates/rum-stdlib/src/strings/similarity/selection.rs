use super::*;

pub(super) fn select_nearest_indices_by_usize(
    distances: &[(usize, usize)],
    spec: NearestResultSpec,
) -> Vec<usize> {
    match spec {
        NearestResultSpec::Count(0) => Vec::new(),
        NearestResultSpec::Count(limit) => select_count_indices_by_usize(distances, limit),
        NearestResultSpec::All => select_all_indices_by_usize(distances),
        NearestResultSpec::BestTies => select_best_ties_indices_by_usize(distances),
    }
}

pub(super) fn select_nearest_indices_by_rational(
    distances: &[(usize, Rational)],
    spec: NearestResultSpec,
) -> Vec<usize> {
    match spec {
        NearestResultSpec::Count(0) => Vec::new(),
        NearestResultSpec::Count(limit) => select_count_indices_by_rational(distances, limit),
        NearestResultSpec::All => select_all_indices_by_rational(distances),
        NearestResultSpec::BestTies => select_best_ties_indices_by_rational(distances),
    }
}

#[inline(always)]
fn select_all_indices_by_usize(distances: &[(usize, usize)]) -> Vec<usize> {
    let mut indices = (0..distances.len()).collect::<Vec<_>>();
    indices.sort_unstable_by_key(|index| (distances[*index].1, distances[*index].0));
    indices
}

#[inline(always)]
fn select_all_indices_by_rational(distances: &[(usize, Rational)]) -> Vec<usize> {
    let mut indices = (0..distances.len()).collect::<Vec<_>>();
    indices.sort_unstable_by(|lhs, rhs| {
        compare_rationals_local(&distances[*lhs].1, &distances[*rhs].1)
            .then(distances[*lhs].0.cmp(&distances[*rhs].0))
    });
    indices
}

#[inline(never)]
fn select_best_ties_indices_by_usize(distances: &[(usize, usize)]) -> Vec<usize> {
    select_best_ties_indices(distances.len(), |lhs, rhs| {
        distances[lhs].1.cmp(&distances[rhs].1)
    })
}

#[inline(never)]
fn select_best_ties_indices_by_rational(distances: &[(usize, Rational)]) -> Vec<usize> {
    select_best_ties_indices(distances.len(), |lhs, rhs| {
        compare_rationals_local(&distances[lhs].1, &distances[rhs].1)
    })
}

fn select_best_ties_indices(
    len: usize,
    compare: impl Fn(usize, usize) -> std::cmp::Ordering,
) -> Vec<usize> {
    if len == 0 {
        return Vec::new();
    }

    let mut best_index = 0;
    let mut indices = vec![best_index];
    for index in 1..len {
        match compare(index, best_index) {
            std::cmp::Ordering::Less => {
                best_index = index;
                indices.clear();
                indices.push(index);
            }
            std::cmp::Ordering::Equal => indices.push(index),
            std::cmp::Ordering::Greater => {}
        }
    }
    indices
}

// Keep each selection strategy isolated so changing one does not perturb the
// inlining and code generation of its same-tier neighbors.
#[inline(never)]
fn select_count_indices_by_usize(distances: &[(usize, usize)], limit: usize) -> Vec<usize> {
    if limit == 1 {
        return distances
            .iter()
            .enumerate()
            .min_by_key(|(_, (source_index, distance))| (*distance, *source_index))
            .map(|(index, _)| vec![index])
            .unwrap_or_default();
    }

    if limit >= distances.len() {
        return select_all_indices_by_usize(distances);
    }

    let mut compare = |lhs: usize, rhs: usize| {
        (distances[lhs].1, distances[lhs].0).cmp(&(distances[rhs].1, distances[rhs].0))
    };
    select_count_indices(distances.len(), limit, &mut compare)
}

#[inline(never)]
fn select_count_indices_by_rational(distances: &[(usize, Rational)], limit: usize) -> Vec<usize> {
    if limit == 1 {
        return distances
            .iter()
            .enumerate()
            .min_by(|(_, lhs), (_, rhs)| {
                compare_rationals_local(&lhs.1, &rhs.1).then(lhs.0.cmp(&rhs.0))
            })
            .map(|(index, _)| vec![index])
            .unwrap_or_default();
    }

    if limit >= distances.len() {
        return select_all_indices_by_rational(distances);
    }

    let mut compare = |lhs: usize, rhs: usize| {
        compare_rationals_local(&distances[lhs].1, &distances[rhs].1)
            .then(distances[lhs].0.cmp(&distances[rhs].0))
    };
    select_count_indices(distances.len(), limit, &mut compare)
}

fn select_count_indices(
    len: usize,
    limit: usize,
    compare: &mut dyn FnMut(usize, usize) -> std::cmp::Ordering,
) -> Vec<usize> {
    let mut indices = (0..len).collect::<Vec<_>>();
    if limit >= len {
        indices.sort_by(|lhs, rhs| compare(*lhs, *rhs));
        return indices;
    }

    indices.select_nth_unstable_by(limit, |lhs, rhs| compare(*lhs, *rhs));
    indices.truncate(limit);
    indices.sort_unstable_by(|lhs, rhs| compare(*lhs, *rhs));
    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn partial_nearest_selection_matches_full_sort_with_linear_comparison_work() {
        let len = 32_000usize;
        let limit = 10usize;
        let distances = (0..len)
            .map(|index| {
                let value = (index + 1).wrapping_mul(7_919) % 32_003;
                (index, value.abs_diff(len / 2))
            })
            .collect::<Vec<_>>();

        let full_comparisons = Cell::new(0usize);
        let mut expected = (0..len).collect::<Vec<_>>();
        expected.sort_by(|lhs, rhs| {
            full_comparisons.set(full_comparisons.get() + 1);
            (distances[*lhs].1, distances[*lhs].0).cmp(&(distances[*rhs].1, distances[*rhs].0))
        });
        expected.truncate(limit);

        let partial_comparisons = Cell::new(0usize);
        let mut compare = |lhs: usize, rhs: usize| {
            partial_comparisons.set(partial_comparisons.get() + 1);
            (distances[lhs].1, distances[lhs].0).cmp(&(distances[rhs].1, distances[rhs].0))
        };
        let selected = select_count_indices(len, limit, &mut compare);

        assert_eq!(selected, expected);
        assert!(partial_comparisons.get() * 4 < full_comparisons.get());
    }

    #[test]
    fn partial_nearest_selection_preserves_distance_then_source_order() {
        let distances = vec![(7, 3), (2, 1), (9, 1), (1, 2), (5, 4)];
        assert_eq!(
            select_nearest_indices_by_usize(&distances, NearestResultSpec::Count(1)),
            vec![1]
        );
        assert_eq!(
            select_nearest_indices_by_usize(&distances, NearestResultSpec::Count(3)),
            vec![1, 2, 3]
        );
        assert_eq!(
            select_nearest_indices_by_usize(&distances, NearestResultSpec::Count(10)),
            vec![1, 2, 3, 0, 4]
        );
        assert_eq!(
            select_nearest_indices_by_usize(&distances, NearestResultSpec::All),
            vec![1, 2, 3, 0, 4]
        );
    }

    #[test]
    fn best_ties_selection_uses_one_comparison_pass() {
        let distances = [4, 1, 3, 1, 2, 1];
        let comparisons = Cell::new(0usize);
        let selected = select_best_ties_indices(distances.len(), |lhs, rhs| {
            comparisons.set(comparisons.get() + 1);
            distances[lhs].cmp(&distances[rhs])
        });

        assert_eq!(selected, vec![1, 3, 5]);
        assert_eq!(comparisons.get(), distances.len() - 1);
    }

    #[test]
    fn rational_comparison_handles_equal_and_distinct_denominators() {
        let one_third = Rational::new(1.into(), 3.into()).unwrap();
        let two_thirds = Rational::new(2.into(), 3.into()).unwrap();
        let one_half = Rational::new(1.into(), 2.into()).unwrap();

        assert_eq!(
            compare_rationals_local(&one_third, &two_thirds),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_rationals_local(&one_half, &two_thirds),
            std::cmp::Ordering::Less
        );
    }
}
