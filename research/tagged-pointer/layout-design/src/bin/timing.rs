//! Paired expression-operation timings. Construction is the same single-pass
//! alias-preserving conversion for Original and each new layout, not evaluation.
use super::graph::{Case, Graph};
use rum_core::Expr;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};

fn timed(mut operation: impl FnMut() -> u64, iterations: usize) -> (f64, u64) {
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    (started.elapsed().as_secs_f64() * 1000.0, checksum)
}

fn row(operation: &str, case: Case, root: &Graph, source: &Expr, iterations: usize) -> (f64, u64) {
    match operation {
        "scan" => timed(|| black_box(root).scan(), iterations),
        "build_drop" => timed(
            || {
                let fresh = Graph::build(case, black_box(source));
                drop(black_box(fresh));
                1
            },
            iterations,
        ),
        "cow" => timed(
            || {
                let edited = black_box(root).cow();
                drop(black_box(edited));
                1
            },
            iterations,
        ),
        "clone_drop" => timed(
            || {
                let copy = black_box(root).clone();
                drop(black_box(copy));
                1
            },
            iterations,
        ),
        "drop" => {
            // Every sample is a fresh graph. Building outside each timer avoids
            // measuring construction and avoids retaining a batch of graphs
            // which would distort locality or turn releases into shared drops.
            let mut elapsed = Duration::ZERO;
            let mut checksum = 0_u64;
            for _ in 0..iterations {
                let fresh = black_box(Graph::build(case, black_box(source)));
                let started = Instant::now();
                drop(fresh);
                elapsed += started.elapsed();
                checksum = checksum.wrapping_add(black_box(1));
            }
            (elapsed.as_secs_f64() * 1000.0, checksum)
        }
        _ => unreachable!("fixed operation set"),
    }
}

pub fn run(label: &str, source: &Expr, expected: u64, run: usize) {
    assert!(
        (1..=3).contains(&run),
        "the recorded plan contains three timing runs"
    );
    let operations = ["scan", "build_drop", "cow", "clone_drop", "drop"];
    let cases = Case::ALL;
    for round in 0..10 {
        // Allocate a new setup each round, rotating creation order as well as
        // measurement order. All construction and setup release are untimed.
        let mut graphs: [Option<Graph>; 5] = std::array::from_fn(|_| None);
        for offset in 0..cases.len() {
            let index = (offset + run + round) % cases.len();
            graphs[index] = Some(Graph::build(cases[index], black_box(source)));
        }
        for (operation_index, operation) in operations.iter().enumerate() {
            let iterations = if label == "rulesSimple" || *operation == "clone_drop" {
                200
            } else if *operation == "scan" {
                3
            } else {
                2
            };
            for offset in 0..cases.len() {
                let rotated = (offset + round + operation_index + run) % cases.len();
                let index = if run == 2 {
                    cases.len() - 1 - rotated
                } else {
                    rotated
                };
                let case = cases[index];
                let root = graphs[index].as_ref().expect("every setup graph exists");
                let (milliseconds, checksum) = row(operation, case, root, source, iterations);
                let wanted = if *operation == "scan" {
                    expected.wrapping_mul(iterations as u64)
                } else {
                    iterations as u64
                };
                assert_eq!(checksum, wanted, "{label}/{operation}/{}", case.name());
                println!(
                    "{label}\t{round}\t{operation}\t{}\t{iterations}\t{milliseconds:.9}\t{checksum}",
                    case.name(),
                );
            }
        }
        drop(graphs);
    }
}
