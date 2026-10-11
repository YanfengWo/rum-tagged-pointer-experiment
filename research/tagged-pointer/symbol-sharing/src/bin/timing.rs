use super::scan;
use rum_core::Expr;
use rum_recursive_expr::{Compact, Safe, Tree};
use std::{hint::black_box, time::Instant};

fn timed(mut operation: impl FnMut() -> u64, iterations: usize) -> (f64, u64) {
    let start = Instant::now();
    let mut sum = 0_u64;
    for _ in 0..iterations {
        sum = sum.wrapping_add(black_box(operation()));
    }
    (start.elapsed().as_secs_f64() * 1000.0, sum)
}
fn builder<N: Tree>(expr: &Expr, cache: bool) -> N {
    if cache {
        let (n, stats) = rum_symbol_sharing::shared(expr, 128);
        black_box(stats);
        n
    } else {
        rum_symbol_sharing::plain(expr)
    }
}
fn row<N: Tree>(op: &str, root: &N, source: &Expr, cache: bool, n: usize) -> (f64, u64) {
    match op {
        "scan" => timed(|| scan::tree(root), n),
        "build_drop" => timed(
            || {
                let node = builder::<N>(source, cache);
                black_box(node);
                1
            },
            n,
        ),
        "cow" => timed(
            || {
                black_box(scan::cow_tree(root));
                1
            },
            n,
        ),
        _ => unreachable!(),
    }
}
pub fn run(dataset: &str, original: &Expr, expected: u64) {
    let safe_plain: Safe = builder(original, false);
    let safe_shared: Safe = builder(original, true);
    let compact_plain: Compact = builder(original, false);
    let compact_shared: Compact = builder(original, true);
    let small = dataset == "rulesSimple";
    for round in 0..10 {
        for (op_index, op) in ["scan", "build_drop", "cow"].iter().enumerate() {
            let count = if small {
                200
            } else if *op == "scan" {
                3
            } else {
                2
            };
            let cases = if *op == "build_drop" { 4 } else { 5 };
            for offset in 0..cases {
                let case = (offset + round + op_index) % cases;
                let (name, result) = match case {
                    0 => ("safe-plain", row(op, &safe_plain, original, false, count)),
                    1 => ("safe-shared", row(op, &safe_shared, original, true, count)),
                    2 => (
                        "compact-plain",
                        row(op, &compact_plain, original, false, count),
                    ),
                    3 => (
                        "compact-shared",
                        row(op, &compact_shared, original, true, count),
                    ),
                    4 => (
                        "original",
                        if *op == "scan" {
                            timed(|| scan::original(original), count)
                        } else {
                            timed(
                                || {
                                    black_box(scan::cow_original(original));
                                    1
                                },
                                count,
                            )
                        },
                    ),
                    _ => unreachable!(),
                };
                let wanted = if *op == "scan" {
                    expected.wrapping_mul(count as u64)
                } else {
                    count as u64
                };
                assert_eq!(result.1, wanted);
                println!(
                    "{dataset}\t{round}\t{op}\t{name}\t{count}\t{:.9}\t{}",
                    result.0, result.1
                );
            }
        }
    }
}
