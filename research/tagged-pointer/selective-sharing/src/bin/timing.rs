use super::policy::{Policy, build};
use rum_core::Expr;
use rum_recursive_expr::{Compact, Safe, Tree};
use std::{hint::black_box, time::Instant};

fn timed<N: Tree>(expr: &Expr, policy: Policy, count: usize) -> f64 {
    let start = Instant::now();
    for _ in 0..count {
        let (node, report) = build::<N>(black_box(expr), policy);
        black_box(report);
        black_box(node); // Ordinary Drop is included; preflight is repeated each time.
    }
    start.elapsed().as_secs_f64() * 1000.0
}
pub fn run(label: &str, expr: &Expr, reversed: bool) {
    let count = if label == "rulesSimple" { 200 } else { 2 };
    for round in 0..10 {
        for offset in 0..6 {
            let case = (offset + round) % 6;
            let case = if reversed { 5 - case } else { case };
            let policy = [Policy::Plain, Policy::Shared, Policy::Selective][case % 3];
            let (owner, ms) = if case < 3 {
                ("safe", timed::<Safe>(expr, policy, count))
            } else {
                ("compact", timed::<Compact>(expr, policy, count))
            };
            println!(
                "{label}\t{round}\t{owner}-{}\t{count}\t{ms:.9}",
                policy.name()
            );
        }
    }
}
