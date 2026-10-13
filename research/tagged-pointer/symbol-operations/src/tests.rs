use super::*;
use crate::collector::Observation;
use std::hash::{Hash, Hasher};
#[derive(Default)]
struct Stream(Vec<u8>);
impl Hasher for Stream {
    fn write(&mut self, b: &[u8]) {
        self.0.extend_from_slice(b);
    }
    fn finish(&self) -> u64 {
        0
    }
}
#[test]
fn complete_order_identity_thresholds_and_empty() {
    let mut inputs = (0..=20)
        .map(|count| {
            Expr::call(
                Expr::symbol("f"),
                (0..count)
                    .map(|i| Expr::symbol(&format!("b{}", i % 19)))
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    let shared = Expr::call(
        Expr::symbol("f"),
        vec![Expr::string("界\0🙂"), Expr::symbol("b1")],
    );
    inputs.push(Expr::call(Expr::symbol("h"), vec![shared.clone(), shared]));
    inputs.push(Expr::small_integer(0));
    inputs.push(Expr::call(
        Expr::symbol("f"),
        vec![
            Expr::Atom(Atom::Symbol(Symbol::with_identity(
                "Global`",
                "g",
                u64::MAX,
            ))),
            Expr::symbol("g"),
            Expr::Atom(Atom::Symbol(Symbol::new("Other`", "g"))),
        ],
    ));
    let seed = RandomState::new();
    for e in inputs {
        let expected = oracle(&e);
        for case in Case::ALL {
            assert_eq!(
                Graph::build(case, &e).collect(case, hint(&e), &seed),
                expected
            );
        }
    }
}
#[test]
fn compatible_key_hash_has_exact_core_input_stream() {
    for s in [
        Symbol::from_name("x"),
        Symbol::new("Other`", "x"),
        Symbol::with_identity("Global`", "x", 1),
        Symbol::with_identity("Global`", "x", u64::MAX),
    ] {
        let n = rum_typed_layout::Typed16::atom(Atom::Symbol(s));
        let k = walk::Cached::<_, true>::new(&n).next().unwrap();
        let mut a = Stream::default();
        let mut b = Stream::default();
        s.hash(&mut a);
        k.hash(&mut b);
        assert_eq!(a.0, b.0);
        assert_eq!(k.resolve(), s);
    }
}
#[test]
fn small_collector_output_retains_first_seen_order() {
    let input = ["a", "b", "a", "c"].map(Symbol::from_name);
    assert_eq!(
        collector::collect(input.into_iter(), 4, &RandomState::new()),
        vec![input[0], input[1], input[3]]
    );
}
