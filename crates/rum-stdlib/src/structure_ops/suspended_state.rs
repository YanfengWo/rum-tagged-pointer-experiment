use std::sync::Arc;
use std::{collections::HashSet, hash::Hash};

use rum_runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenStructureSequence<T: FrameRootTrace>(Arc<[T]>);

impl<T: FrameRootTrace> FrozenStructureSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        Self(Arc::from(values))
    }

    pub(super) fn as_slice(&self) -> &[T] {
        self.0.as_ref()
    }

    pub(super) fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct FrozenStructureValue<T: FrameRootTrace>(Arc<T>);

impl<T: FrameRootTrace> FrozenStructureValue<T> {
    pub(super) fn freeze(value: T) -> Self {
        Self(Arc::new(value))
    }

    pub(super) fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentStructureSequence<T: FrameRootTrace> {
    prefix: Arc<[T]>,
    tail: Option<Arc<PersistentStructureSequenceNode<T>>>,
    len: usize,
}

#[derive(FrameRootTrace)]
struct PersistentStructureSequenceNode<T: FrameRootTrace> {
    previous: Option<Arc<PersistentStructureSequenceNode<T>>>,
    value: T,
}

impl<T: FrameRootTrace> PersistentStructureSequence<T> {
    pub(super) fn freeze(values: Vec<T>) -> Self {
        let len = values.len();
        Self {
            prefix: Arc::from(values),
            tail: None,
            len,
        }
    }

    pub(super) fn appended(&self, value: T) -> Self {
        Self {
            prefix: self.prefix.clone(),
            tail: Some(Arc::new(PersistentStructureSequenceNode {
                previous: self.tail.clone(),
                value,
            })),
            len: self.len + 1,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn get(&self, index: usize) -> Option<&T> {
        if index < self.prefix.len() {
            return self.prefix.get(index);
        }
        if index >= self.len {
            return None;
        }
        let mut remaining = self.len - index - 1;
        let mut node = self.tail.as_deref();
        while remaining > 0 {
            node = node?.previous.as_deref();
            remaining -= 1;
        }
        node.map(|node| &node.value)
    }
}

impl<T: Clone + FrameRootTrace> PersistentStructureSequence<T> {
    pub(super) fn to_vec(&self) -> Vec<T> {
        let mut suffix = Vec::with_capacity(self.len.saturating_sub(self.prefix.len()));
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            suffix.push(current.value.clone());
            node = current.previous.as_deref();
        }
        suffix.reverse();

        let mut values = Vec::with_capacity(self.len);
        values.extend(self.prefix.iter().cloned());
        values.extend(suffix);
        values
    }
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentStructureSet<T: FrameRootTrace> {
    base: Arc<HashSet<T>>,
    tail: Option<Arc<PersistentStructureSetNode<T>>>,
}

#[derive(Clone, FrameRootTrace)]
pub(super) struct PersistentStructureCounts {
    base: Arc<[(rum_core::Expr, usize)]>,
    tail: Option<Arc<PersistentStructureCountNode>>,
    len: usize,
}

#[derive(FrameRootTrace)]
struct PersistentStructureCountNode {
    previous: Option<Arc<PersistentStructureCountNode>>,
    update: PersistentStructureCountUpdate,
}

#[derive(FrameRootTrace)]
enum PersistentStructureCountUpdate {
    Increment(usize),
    Append(rum_core::Expr),
}

impl PersistentStructureCounts {
    pub(super) fn freeze(values: Vec<(rum_core::Expr, usize)>) -> Self {
        let len = values.len();
        Self {
            base: Arc::from(values),
            tail: None,
            len,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }

    pub(super) fn item(&self, index: usize) -> Option<rum_core::Expr> {
        if let Some((item, _)) = self.base.get(index) {
            return Some(item.clone());
        }
        let mut appended = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            if let PersistentStructureCountUpdate::Append(item) = &current.update {
                appended.push(item);
            }
            node = current.previous.as_deref();
        }
        let offset = index.checked_sub(self.base.len())?;
        appended
            .get(appended.len().checked_sub(offset + 1)?)
            .map(|item| (*item).clone())
    }

    pub(super) fn incremented(&self, index: usize) -> Self {
        Self {
            base: self.base.clone(),
            tail: Some(Arc::new(PersistentStructureCountNode {
                previous: self.tail.clone(),
                update: PersistentStructureCountUpdate::Increment(index),
            })),
            len: self.len,
        }
    }

    pub(super) fn appended(&self, item: rum_core::Expr) -> Self {
        Self {
            base: self.base.clone(),
            tail: Some(Arc::new(PersistentStructureCountNode {
                previous: self.tail.clone(),
                update: PersistentStructureCountUpdate::Append(item),
            })),
            len: self.len + 1,
        }
    }

    pub(super) fn to_vec(&self) -> Vec<(rum_core::Expr, usize)> {
        let mut updates = Vec::new();
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            updates.push(&current.update);
            node = current.previous.as_deref();
        }
        let mut values = self.base.iter().cloned().collect::<Vec<_>>();
        for update in updates.into_iter().rev() {
            match update {
                PersistentStructureCountUpdate::Increment(index) => values[*index].1 += 1,
                PersistentStructureCountUpdate::Append(item) => values.push((item.clone(), 1)),
            }
        }
        values
    }
}

#[derive(FrameRootTrace)]
struct PersistentStructureSetNode<T: FrameRootTrace> {
    previous: Option<Arc<PersistentStructureSetNode<T>>>,
    value: T,
}

impl<T: Clone + Eq + Hash + FrameRootTrace> PersistentStructureSet<T> {
    pub(super) fn freeze(values: HashSet<T>) -> Self {
        Self {
            base: Arc::new(values),
            tail: None,
        }
    }

    pub(super) fn inserted(&self, value: T) -> (Self, bool) {
        if self.base.contains(&value) {
            return (self.clone(), false);
        }
        let mut node = self.tail.as_deref();
        while let Some(current) = node {
            if current.value == value {
                return (self.clone(), false);
            }
            node = current.previous.as_deref();
        }
        (
            Self {
                base: self.base.clone(),
                tail: Some(Arc::new(PersistentStructureSetNode {
                    previous: self.tail.clone(),
                    value,
                })),
            },
            true,
        )
    }
}

#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<FrozenStructureSequence<rum_core::Expr>>();
    assert_traces::<FrozenStructureValue<rum_core::Expr>>();
    assert_traces::<PersistentStructureSequence<rum_core::Expr>>();
    assert_traces::<PersistentStructureSequenceNode<rum_core::Expr>>();
    assert_traces::<PersistentStructureSet<rum_core::Expr>>();
    assert_traces::<PersistentStructureSetNode<rum_core::Expr>>();
    assert_traces::<PersistentStructureCounts>();
    assert_traces::<PersistentStructureCountNode>();
    assert_traces::<PersistentStructureCountUpdate>();
};
