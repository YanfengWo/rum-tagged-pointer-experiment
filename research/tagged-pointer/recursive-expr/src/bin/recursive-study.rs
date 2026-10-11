#[cfg(not(feature = "allocation-count"))]
use rum_core::Expr;
use rum_recursive_expr::{Compact, Safe};
#[cfg(not(feature = "allocation-count"))]
use rum_recursive_expr::{Tree, View, from_expr, to_expr};
#[cfg(not(feature = "allocation-count"))]
use std::{hint::black_box, time::Instant};
#[cfg(feature = "allocation-count")]
#[path = "study/allocation.rs"]
mod allocation;
#[path = "study/fixtures.rs"]
#[cfg_attr(feature = "allocation-count", allow(dead_code))]
mod fixtures;
#[cfg(feature = "allocation-count")]
#[global_allocator]
static ALLOCATOR: allocation::Counting = allocation::Counting;

#[cfg(not(feature = "allocation-count"))]
fn clock(mut work: impl FnMut() -> u64, passes: usize) -> (u128, u64) {
    let start = Instant::now();
    let mut sum = 0_u64;
    for _ in 0..passes {
        sum = sum.wrapping_add(black_box(work()));
    }
    (start.elapsed().as_nanos(), sum)
}
#[cfg(not(feature = "allocation-count"))]
fn row(dataset: &str, round: usize, repr: &str, operation: &str, (time, sum): (u128, u64)) {
    println!("{dataset}\t{round}\t{repr}\t{operation}\t{time}\t{sum}");
}

#[cfg(not(feature = "allocation-count"))]
fn tree_rows<N: Tree>(dataset: &str, round: usize, repr: &str, input: &Expr, root: &N) {
    row(
        dataset,
        round,
        repr,
        "scan",
        clock(|| fixtures::checksum(black_box(root)), 10),
    );
    row(
        dataset,
        round,
        repr,
        "clone_drop",
        clock(
            || {
                drop(black_box(root.clone()));
                1
            },
            5_000,
        ),
    );
    row(
        dataset,
        round,
        repr,
        "cow",
        clock(
            || {
                let View::Call { head, args } = root.view() else {
                    unreachable!()
                };
                let mut args = args.clone();
                args.make_mut()
                    .push(N::atom(rum_core::Atom::Integer(rum_core::IntValue::Small(
                        1,
                    ))));
                drop(black_box(N::call(head.clone(), args)));
                1
            },
            10,
        ),
    );
    row(
        dataset,
        round,
        repr,
        "bridge_roundtrip",
        clock(
            || {
                let node: N = from_expr(black_box(input));
                drop(black_box(to_expr(&node)));
                drop(node);
                1
            },
            3,
        ),
    );
    // Last-owner setup is outside the timed interval, including bridge caches.
    for operation in ["last_drop", "iterative_release"] {
        let mut time = 0;
        let mut sum = 0;
        for _ in 0..5 {
            let node: N = from_expr(input);
            let start = Instant::now();
            if operation == "last_drop" {
                drop(black_box(node));
            } else {
                black_box(node).release_iterative();
            }
            time += start.elapsed().as_nanos();
            sum += 1;
        }
        row(dataset, round, repr, operation, (time, sum));
    }
}
#[cfg(not(feature = "allocation-count"))]
fn original_rows(dataset: &str, round: usize, input: &Expr) {
    row(
        dataset,
        round,
        "original",
        "scan",
        clock(|| fixtures::original_checksum(black_box(input)), 10),
    );
    row(
        dataset,
        round,
        "original",
        "clone_drop",
        clock(
            || {
                drop(black_box(input.clone()));
                1
            },
            5_000,
        ),
    );
    row(
        dataset,
        round,
        "original",
        "cow",
        clock(
            || {
                let Expr::Call { head, args } = input else {
                    unreachable!()
                };
                let mut args = args.clone();
                args.make_mut().push(Expr::small_integer(1));
                drop(black_box(Expr::call_shared(head.clone(), args)));
                1
            },
            10,
        ),
    );
    for operation in ["last_drop", "iterative_release"] {
        let mut time = 0;
        let mut sum = 0;
        for _ in 0..5 {
            let fresh = fixtures::fixture(dataset);
            let start = Instant::now();
            if operation == "last_drop" {
                drop(black_box(fresh));
            } else {
                fixtures::release_original(black_box(fresh));
            }
            time += start.elapsed().as_nanos();
            sum += 1;
        }
        row(dataset, round, "original", operation, (time, sum));
    }
}

#[cfg(not(feature = "allocation-count"))]
fn main() {
    println!("dataset\tround\trepresentation\toperation\tnanoseconds\tchecksum");
    for dataset in fixtures::DATASETS {
        let input = fixtures::fixture(dataset);
        let compact = Compact::from_expr(&input);
        let safe = Safe::from_expr(&input);
        let sum = fixtures::original_checksum(&input);
        assert_eq!(fixtures::checksum(&compact), sum);
        assert_eq!(fixtures::checksum(&safe), sum);
        assert_eq!(compact.to_expr(), input);
        assert_eq!(safe.to_expr(), input);
        for round in 0..10 {
            for slot in 0..3 {
                match (round + slot) % 3 {
                    0 => original_rows(dataset, round, &input),
                    1 => tree_rows(dataset, round, "safe", &input, &safe),
                    _ => tree_rows(dataset, round, "compact", &input, &compact),
                }
            }
        }
    }
}

#[cfg(test)]
mod checks {
    use super::*;
    use rum_core::Expr;
    use rum_recursive_expr::{Tree, View, from_expr};
    #[test]
    fn fixture_cow_preserves_value_and_source() {
        fn check<N: Tree>(input: &Expr, expected: u64) {
            let root: N = from_expr(input);
            let View::Call { head, args } = root.view() else {
                panic!()
            };
            let weak = args.downgrade();
            let mut changed = args.clone();
            changed
                .make_mut()
                .push(N::atom(rum_core::Atom::Integer(rum_core::IntValue::Small(
                    1,
                ))));
            assert!(!args.ptr_eq(&changed));
            let modified = N::call(head.clone(), changed);
            assert_eq!(fixtures::checksum(&modified), expected + 1);
            assert_eq!(fixtures::checksum(&root), expected);
            modified.release_iterative();
            assert!(weak.is_alive());
            root.release_iterative();
            assert!(!weak.is_alive());
        }
        for dataset in fixtures::DATASETS {
            let input = fixtures::fixture(dataset);
            let expected = fixtures::original_checksum(&input);
            check::<Compact>(&input, expected);
            check::<Safe>(&input, expected);
            fixtures::release_original(input);
        }
    }
}

#[cfg(feature = "allocation-count")]
fn measure<T>(dataset: &str, repr: &str, build: impl FnOnce() -> T) {
    let start = allocation::start();
    let root = build();
    let kept = allocation::snapshot();
    drop(root);
    let released = allocation::snapshot();
    assert_eq!(
        released.live, start.live,
        "live allocations remained after release"
    );
    println!(
        "{dataset}\t{repr}\t{}\t{}\t{}\t{}\t{}",
        std::mem::size_of::<T>(),
        kept.live - start.live,
        kept.peak - start.live,
        kept.calls - start.calls,
        released.live - start.live
    );
}
#[cfg(feature = "allocation-count")]
fn main() {
    println!(
        "dataset\trepresentation\troot_stack_bytes\tretained_requested_bytes\tpeak_requested_bytes\tallocation_calls\tremaining_bytes"
    );
    for dataset in fixtures::DATASETS {
        // Create source and intern symbols before starting each accounting window.
        let input = fixtures::fixture(dataset);
        measure(dataset, "original", || fixtures::fixture(dataset));
        measure(dataset, "safe", || Safe::from_expr(&input));
        measure(dataset, "compact", || Compact::from_expr(&input));
    }
}
