use std::collections::HashMap;

use rum_core::Expr;

const GROUP_INDEX_THRESHOLD: usize = 16;

pub(super) struct StableGroups<T> {
    entries: Vec<(Expr, Vec<T>)>,
}

impl<T> StableGroups<T> {
    pub(super) fn collect(pairs: impl IntoIterator<Item = (Expr, T)>) -> Self {
        let mut remaining = pairs.into_iter();
        let mut entries: Vec<(Expr, Vec<T>)> = Vec::new();
        for (key, value) in remaining.by_ref() {
            if let Some((_, values)) = entries.iter_mut().find(|(existing, _)| existing == &key) {
                values.push(value);
            } else {
                entries.push((key, vec![value]));
                if entries.len() > GROUP_INDEX_THRESHOLD {
                    return Self::collect_indexed(entries, remaining);
                }
            }
        }

        Self { entries }
    }

    #[inline(never)]
    fn collect_indexed(
        mut entries: Vec<(Expr, Vec<T>)>,
        remaining: impl Iterator<Item = (Expr, T)>,
    ) -> Self {
        let mut positions: HashMap<Expr, usize> = entries
            .iter()
            .enumerate()
            .map(|(index, (key, _))| (key.clone(), index))
            .collect();
        for (key, value) in remaining {
            if let Some(index) = positions.get(&key).copied() {
                entries[index].1.push(value);
            } else {
                let index = entries.len();
                positions.insert(key.clone(), index);
                entries.push((key, vec![value]));
            }
        }

        Self { entries }
    }

    pub(super) fn into_entries(self) -> Vec<(Expr, Vec<T>)> {
        self.entries
    }
}

pub(super) struct StableCounts {
    entries: Vec<(Expr, usize)>,
}

impl StableCounts {
    pub(super) fn collect(items: impl IntoIterator<Item = Expr>) -> Self {
        let mut remaining = items.into_iter();
        let mut entries: Vec<(Expr, usize)> = Vec::new();
        for item in remaining.by_ref() {
            if let Some((_, count)) = entries.iter_mut().find(|(existing, _)| existing == &item) {
                *count += 1;
            } else {
                entries.push((item, 1));
                if entries.len() > GROUP_INDEX_THRESHOLD {
                    return Self::collect_indexed(entries, remaining);
                }
            }
        }

        Self { entries }
    }

    #[inline(never)]
    fn collect_indexed(
        mut entries: Vec<(Expr, usize)>,
        remaining: impl Iterator<Item = Expr>,
    ) -> Self {
        let mut positions: HashMap<Expr, usize> = entries
            .iter()
            .enumerate()
            .map(|(index, (item, _))| (item.clone(), index))
            .collect();
        for item in remaining {
            if let Some(index) = positions.get(&item).copied() {
                entries[index].1 += 1;
            } else {
                let index = entries.len();
                positions.insert(item.clone(), index);
                entries.push((item, 1));
            }
        }

        Self { entries }
    }

    pub(super) fn into_entries(self) -> Vec<(Expr, usize)> {
        self.entries
    }
}
