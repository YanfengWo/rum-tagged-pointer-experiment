use crate::graph::{Case, Graph};
use rum_core::{Expr, Symbol};
use std::{collections::hash_map::RandomState, hint::black_box, time::Instant};
pub fn checksum(symbols: &[Symbol]) -> u64 {
    symbols.iter().fold(0_u64, |s, k| {
        s.wrapping_mul(31)
            .wrapping_add(k.interned_id() as u64)
            .wrapping_add(k.identity().unwrap_or(0))
    })
}
pub fn run(
    label: &str,
    e: &Expr,
    hint: usize,
    expected: &[Symbol],
    run: usize,
    seed: &RandomState,
) {
    let ops = ["dedup-1", "dedup-8", "convert-dedup-1", "convert-dedup-8"];
    let checksum_expected = checksum(expected);
    for round in 0..10 {
        let mut graphs: [Option<Graph>; 7] = std::array::from_fn(|_| None);
        for offset in 0..7 {
            let i = (offset + run + round) % 7;
            graphs[i] = Some(Graph::build(Case::ALL[i], e));
        }
        for (oi, op) in ops.iter().enumerate() {
            let repeats = if op.ends_with('8') { 8 } else { 1 };
            let iterations = if label.starts_with("unique-") || label == "rulesSimple" {
                100
            } else {
                3
            };
            for offset in 0..7 {
                let rotated = (offset + round + oi + run) % 7;
                let i = if run == 2 { 6 - rotated } else { rotated };
                let case = Case::ALL[i];
                let root = graphs[i].as_ref().unwrap();
                let mut total = 0_u64;
                let start = Instant::now();
                for _ in 0..iterations {
                    let fresh = op
                        .starts_with("convert")
                        .then(|| Graph::build(case, black_box(e)));
                    let source = fresh.as_ref().unwrap_or(root);
                    for _ in 0..repeats {
                        let result = black_box(source).collect(case, hint, black_box(seed));
                        total = total.wrapping_add(checksum(black_box(&result)));
                        drop(black_box(result));
                    }
                    drop(fresh);
                }
                let ms = start.elapsed().as_secs_f64() * 1000.;
                assert_eq!(
                    total,
                    checksum_expected.wrapping_mul((iterations * repeats) as u64)
                );
                println!(
                    "{label}\t{round}\t{op}\t{}\t{iterations}\t{ms:.9}\t{total}",
                    case.name()
                );
            }
        }
    }
}
