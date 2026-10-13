use super::*;
use crate::key::Probe;
use std::hash::{BuildHasherDefault, Hash, Hasher};
#[derive(Default)]
struct Zero;
impl Hasher for Zero {
    fn write(&mut self, _: &[u8]) {}
    fn finish(&self) -> u64 {
        0
    }
}
#[derive(Default)]
struct Bytes(Vec<u8>);
impl Hasher for Bytes {
    fn write(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b)
    }
    fn finish(&self) -> u64 {
        0
    }
}
fn symbols() -> [Symbol; 4] {
    [
        Symbol::new("Global`", "x"),
        Symbol::new("Other`", "x"),
        Symbol::with_identity("Global`", "x", 1),
        Symbol::with_identity("Global`", "x", u64::MAX),
    ]
}
#[test]
fn complete_identity_collision_overwrite_and_cached_none() {
    let mut map = Map::<Option<u32>, BuildHasherDefault<Zero>>::default();
    let values = symbols();
    for (i, s) in values.iter().enumerate() {
        assert_eq!(map.insert(*s, Some(i as u32)), None);
    }
    for (i, s) in values.iter().enumerate() {
        assert_eq!(map.get(s), Some(&Some(i as u32)));
    }
    assert_eq!(map.insert(values[0], None), Some(Some(0)));
    assert_eq!(map.get(&values[0]), Some(&None));
    assert_eq!(map.get(&Symbol::new("Missing`", "x")), None);
    assert_eq!(map.len(), 4);
    map.clear();
    assert!(map.is_empty());
    for s in values {
        assert_eq!(map.get(&s), None);
    }
}
#[test]
fn exact_core_hash_stream_and_size() {
    for s in symbols() {
        let key = Key::new(s);
        let mut a = Bytes::default();
        let mut b = Bytes::default();
        s.hash(&mut a);
        key.hash(&mut b);
        assert_eq!(a.0, b.0);
        assert_eq!(key.probe(), Probe::from_symbol(s));
    }
    let expected = if cfg!(any(feature = "packed16", feature = "tagged16")) {
        16
    } else {
        24
    };
    assert_eq!(
        Map::<u32, std::collections::hash_map::RandomState>::key_size(),
        expected
    );
}
#[test]
fn native_random_map_large_and_repeated_values() {
    let mut a = Map::<u32, std::collections::hash_map::RandomState>::default();
    let mut b = std::collections::HashMap::new();
    for i in 0..64 {
        let s = Symbol::with_identity("Global`", "g", i + 1);
        a.insert(s, i as u32);
        b.insert(s, i as u32);
    }
    for (k, v) in b {
        assert_eq!(a.get(&k), Some(&v));
    }
}
