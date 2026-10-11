//! Representation-only timings; no evaluator performance claim follows.
use rum_compact_expr::{CompactExpr, ExprView, SafeExpr};
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

fn view_read(view: ExprView<'_>) -> i64 {
    match view {
        ExprView::Integer(value) => value,
        ExprView::Heap(expr) => original_read(expr),
    }
}

fn sample<T: Clone>(label: &str, values: &[T], read: impl Fn(&T) -> i64) {
    let start = Instant::now();
    let mut sum = 0_i64;
    for _ in 0..100 {
        for value in black_box(values) {
            sum = sum.wrapping_add(read(value));
        }
    }
    let elapsed = start.elapsed().as_nanos();
    println!("scan\t{label}\t{elapsed}\t{}", black_box(sum));
    let start = Instant::now();
    for _ in 0..20 {
        black_box(values.to_vec());
    }
    println!("clone_drop\t{label}\t{}\t0", start.elapsed().as_nanos());
}

fn construct_convert<T>(
    label: &str,
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
    println!(
        "construct_convert\t{label}\t{}\t{}",
        start.elapsed().as_nanos(),
        black_box(sum)
    );
}

fn main() {
    println!("workload\trepresentation\tnanoseconds\tchecksum");
    for mixed in [false, true] {
        let originals: Vec<_> = (0..100_000)
            .map(|index| {
                if mixed && index % 4 == 0 {
                    Expr::string("payload")
                } else {
                    Expr::small_integer(index)
                }
            })
            .collect();
        let safe: Vec<_> = originals.iter().cloned().map(SafeExpr::new).collect();
        let compact: Vec<_> = originals.iter().cloned().map(CompactExpr::new).collect();
        let prefix = if mixed { "mixed" } else { "integers" };
        // Rotate order across repetitions to reduce systematic order bias.
        for round in 0..9 {
            for slot in 0..3 {
                match (round + slot) % 3 {
                    0 => {
                        sample(&format!("{prefix}/original"), &originals, original_read);
                        construct_convert(&format!("{prefix}/original"), &originals, |v| v, |v| v);
                    }
                    1 => {
                        sample(&format!("{prefix}/safe"), &safe, |v| view_read(v.view()));
                        construct_convert(
                            &format!("{prefix}/safe"),
                            &originals,
                            SafeExpr::new,
                            SafeExpr::into_expr,
                        );
                    }
                    _ => {
                        sample(&format!("{prefix}/compact"), &compact, |v| {
                            view_read(v.view())
                        });
                        construct_convert(
                            &format!("{prefix}/compact"),
                            &originals,
                            CompactExpr::new,
                            CompactExpr::into_expr,
                        );
                    }
                }
            }
        }
    }
}
