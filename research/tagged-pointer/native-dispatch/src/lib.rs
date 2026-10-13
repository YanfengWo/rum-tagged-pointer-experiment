//! Native cache keys behind a safe map API. No Expr conversion in lookup.
#![forbid(unsafe_code)]
mod key;
use key::Key;
#[cfg(any(feature = "tagged16", feature = "safe24", feature = "packed16"))]
use key::Probe;
use rum_core::Symbol;
use std::hash::BuildHasher;
pub struct Map<V, S> {
    inner: hashbrown::HashMap<Key, V, S>,
}
impl<V, S: BuildHasher + Default> Default for Map<V, S> {
    fn default() -> Self {
        Self {
            inner: hashbrown::HashMap::with_hasher(S::default()),
        }
    }
}
impl<V, S: BuildHasher> Map<V, S> {
    #[inline]
    pub fn get(&self, s: &Symbol) -> Option<&V> {
        #[cfg(not(any(feature = "tagged16", feature = "safe24", feature = "packed16")))]
        {
            self.inner.get(s)
        }
        #[cfg(any(feature = "tagged16", feature = "safe24", feature = "packed16"))]
        {
            let query = Probe::from_symbol(*s);
            self.inner.get(&query)
        }
    }
    #[inline]
    pub fn insert(&mut self, s: Symbol, value: V) -> Option<V> {
        self.inner.insert(Key::new(s), value)
    }
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    pub fn key_size() -> usize {
        std::mem::size_of::<Key>()
    }
}
impl<V, S> std::fmt::Debug for Map<V, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeDispatchMap")
            .field("entries", &self.inner.len())
            .finish()
    }
}
#[cfg(test)]
mod tests;
