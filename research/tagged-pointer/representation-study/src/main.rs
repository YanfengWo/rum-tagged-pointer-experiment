//! Same-process interleaved comparisons against pinned v1 and a safe control.
use compact_v1::CompactExpr as V1;
use compact_v1_hinted::CompactExpr as V1Hinted;
use rum_compact_expr::{CompactExpr as V2, SafeExpr};
use rum_core::{Atom, Expr, IntValue};
use std::hint::black_box;
use std::time::Instant;

fn original_read(expr: &Expr) -> i64 {
    match expr {
        Expr::Atom(Atom::Integer(IntValue::Small(value))) => *value,
        Expr::Atom(Atom::String(value)) => value.len() as i64,
        _ => unreachable!(),
    }
}

fn record(
    dataset: &str,
    round: usize,
    operation: &str,
    representation: &str,
    nanos: u128,
    checksum: i64,
) {
    println!("{dataset}\t{round}\t{operation}\t{representation}\t{nanos}\t{checksum}");
}

fn sample<T: Clone>(
    dataset: &str,
    round: usize,
    representation: &str,
    values: &[T],
    read: impl Fn(&T) -> i64,
) {
    let start = Instant::now();
    let mut sum = 0_i64;
    for _ in 0..50 {
        for value in black_box(values) {
            sum = sum.wrapping_add(read(value));
        }
    }
    record(
        dataset,
        round,
        "scan",
        representation,
        start.elapsed().as_nanos(),
        black_box(sum),
    );
    let start = Instant::now();
    for _ in 0..10 {
        black_box(values.to_vec());
    }
    record(
        dataset,
        round,
        "clone_drop",
        representation,
        start.elapsed().as_nanos(),
        0,
    );
}

fn construct_convert<T>(
    dataset: &str,
    round: usize,
    representation: &str,
    values: &[Expr],
    new: impl Fn(Expr) -> T,
    into: impl Fn(T) -> Expr,
) {
    let start = Instant::now();
    let mut sum = 0_i64;
    for value in black_box(values) {
        let value = into(black_box(new(value.clone())));
        sum = sum.wrapping_add(original_read(&value));
    }
    record(
        dataset,
        round,
        "construct_convert",
        representation,
        start.elapsed().as_nanos(),
        black_box(sum),
    );
}

fn typed_scan(dataset: &str, round: usize, values: &[V2]) {
    use rum_compact_expr::{AtomRef, ExprRef, IntRef};
    let start = Instant::now();
    let mut sum = 0_i64;
    for _ in 0..50 {
        for value in black_box(values) {
            let scalar = match value.borrow() {
                ExprRef::Atom(AtomRef::Integer(IntRef::Small(n))) => n,
                ExprRef::Atom(AtomRef::String(text)) => text.len() as i64,
                _ => unreachable!(),
            };
            sum = sum.wrapping_add(scalar);
        }
    }
    record(
        dataset,
        round,
        "typed_scan",
        "v2",
        start.elapsed().as_nanos(),
        black_box(sum),
    );
}

fn make_values(percent: usize, shuffled: bool, strided: bool) -> Vec<Expr> {
    let mut values: Vec<_> = (0..100_000)
        .map(|index| {
            if if strided {
                index % 4 == 0
            } else {
                index % 100 < percent
            } {
                Expr::string("payload")
            } else {
                Expr::small_integer(index as i64)
            }
        })
        .collect();
    if shuffled {
        let mut seed = 0x1234_5678_9abc_def0_u64;
        for end in (1..values.len()).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            values.swap(end, seed as usize % (end + 1));
        }
    }
    values
}

fn main() {
    println!("dataset\tround\toperation\trepresentation\tnanoseconds\tchecksum");
    for (dataset, percent, shuffled, strided) in [
        ("heap00", 0, false, false),
        ("heap25-blocks", 25, false, false),
        ("heap25-strided", 25, false, true),
        ("heap25-shuffled", 25, true, false),
        ("heap50-blocks", 50, false, false),
        ("heap100", 100, false, false),
    ] {
        let originals = make_values(percent, shuffled, strided);
        let safe: Vec<_> = originals.iter().cloned().map(SafeExpr::new).collect();
        let v1: Vec<_> = originals.iter().cloned().map(V1::new).collect();
        let v1_hinted: Vec<_> = originals.iter().cloned().map(V1Hinted::new).collect();
        let v2: Vec<_> = originals.iter().cloned().map(V2::new).collect();
        for round in 0..16 {
            // Round zero warms code, data and allocator and is excluded in analysis.
            for slot in 0..5 {
                match (round + slot) % 5 {
                    0 => {
                        sample(dataset, round, "original", &originals, original_read);
                        construct_convert(dataset, round, "original", &originals, |v| v, |v| v);
                    }
                    1 => {
                        sample(dataset, round, "safe", &safe, |v| match v.view() {
                            rum_compact_expr::ExprView::Integer(n) => n,
                            rum_compact_expr::ExprView::Heap(expr) => original_read(expr),
                        });
                        construct_convert(
                            dataset,
                            round,
                            "safe",
                            &originals,
                            SafeExpr::new,
                            SafeExpr::into_expr,
                        );
                    }
                    2 => {
                        sample(dataset, round, "v1", &v1, |v| match v.view() {
                            compact_v1::ExprView::Integer(n) => n,
                            compact_v1::ExprView::Heap(expr) => original_read(expr),
                        });
                        construct_convert(dataset, round, "v1", &originals, V1::new, V1::into_expr);
                    }
                    3 => {
                        sample(dataset, round, "v1-hinted", &v1_hinted, |v| {
                            match v.view() {
                                compact_v1_hinted::ExprView::Integer(n) => n,
                                compact_v1_hinted::ExprView::Heap(expr) => original_read(expr),
                            }
                        });
                        construct_convert(
                            dataset,
                            round,
                            "v1-hinted",
                            &originals,
                            V1Hinted::new,
                            V1Hinted::into_expr,
                        );
                    }
                    _ => {
                        // Alternate typed-view timing before/after the existing
                        // view scan, so it does not always inherit a warmer cache.
                        if round % 2 == 0 {
                            typed_scan(dataset, round, &v2);
                        }
                        sample(dataset, round, "v2", &v2, |v| match v.view() {
                            rum_compact_expr::ExprView::Integer(n) => n,
                            rum_compact_expr::ExprView::Heap(expr) => original_read(expr),
                        });
                        if round % 2 != 0 {
                            typed_scan(dataset, round, &v2);
                        }
                        construct_convert(dataset, round, "v2", &originals, V2::new, V2::into_expr);
                    }
                }
            }
        }
    }
}
