use rum_recursive_expr::{Compact, Safe};
use std::{env, fs, path::Path};
#[cfg(feature = "allocation-count")]
#[path = "../../recursive-expr/src/bin/study/allocation.rs"]
mod allocation;
mod corpus;
mod graph;
mod shape;
#[cfg(feature = "allocation-count")]
#[global_allocator]
static ALLOCATOR: allocation::Counting = allocation::Counting;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(
        args.len(),
        4,
        "usage: rum-expression-census extract|census|measure SOURCE_OR_CORPUS OUTPUT"
    );
    let input = Path::new(&args[2]);
    let out = Path::new(&args[3]);
    if args[1] == "extract" {
        corpus::extract(input, out);
        return;
    }
    let snapshots = corpus::load(input);
    let mut rows = Vec::new();
    for (entry, expr) in snapshots {
        let compact = Compact::from_expr(&expr);
        let safe = Safe::from_expr(&expr);
        assert_eq!(compact.to_expr(), expr, "{}", entry.label);
        assert_eq!(safe.to_expr(), expr);
        drop(compact);
        drop(safe);
        if args[1] == "census" {
            rows.push(serde_json::json!({"entry": entry, "graph": graph::census(&expr)}));
        } else {
            assert_eq!(args[1], "measure");
            #[cfg(feature = "allocation-count")]
            rows.push(serde_json::json!({"entry": entry, "measurements": [
                measure("original-normalized", || Safe::from_expr(&expr).to_expr()),
                measure("safe", || Safe::from_expr(&expr)), measure("compact", || Compact::from_expr(&expr))]}));
            #[cfg(not(feature = "allocation-count"))]
            panic!("measure requires the allocation-count feature");
        }
    }
    fs::write(out, serde_json::to_string_pretty(&rows).unwrap() + "\n").unwrap();
    println!(
        "PASS: {} snapshots; Compact and Safe roundtrip equality checked on every snapshot",
        rows.len()
    );
}

#[cfg(feature = "allocation-count")]
fn measure<T>(representation: &str, build: impl FnOnce() -> T) -> serde_json::Value {
    let start = allocation::start();
    let root = build();
    let kept = allocation::snapshot();
    drop(root);
    let released = allocation::snapshot();
    assert_eq!(
        released.live, start.live,
        "allocation window did not return to baseline"
    );
    serde_json::json!({"representation": representation, "root_stack_bytes": std::mem::size_of::<T>(),
        "retained_requested_bytes": kept.live - start.live, "peak_requested_bytes": kept.peak - start.live,
        "allocation_calls": kept.calls - start.calls, "remaining_bytes": released.live - start.live})
}
