use rum_core::Expr;
use rum_recursive_expr::{Compact, Safe, Tree, to_expr};
#[cfg(feature = "allocation-count")]
use std::fs;
use std::{env, path::Path};
#[cfg(feature = "allocation-count")]
#[path = "../../../recursive-expr/src/bin/study/allocation.rs"]
mod allocation;
#[allow(dead_code)]
#[path = "../../../expression-census/src/corpus.rs"]
mod corpus;
mod scan;
mod timing;
#[cfg(feature = "allocation-count")]
#[global_allocator]
static ALLOCATOR: allocation::Counting = allocation::Counting;

fn validate<N: Tree>(expr: &Expr, expected: u64) {
    let plain: N = rum_symbol_sharing::plain(expr);
    let (shared, stats) = rum_symbol_sharing::shared::<N>(expr, 128);
    assert_eq!(to_expr(&plain), *expr);
    assert_eq!(to_expr(&shared), *expr);
    assert_eq!(scan::tree(&plain), expected);
    assert_eq!(scan::tree(&shared), expected);
    assert_eq!(
        stats.symbol_visits,
        stats.entries + stats.hits + stats.uncached
    );
    if matches!(expr, Expr::Call { .. }) {
        for node in [&plain, &shared] {
            assert_eq!(
                scan::tree(&scan::cow_tree(node)),
                scan::appended_checksum(expected)
            );
            assert_eq!(scan::tree(node), expected);
        }
        assert_eq!(
            scan::original(&scan::cow_original(expr)),
            scan::appended_checksum(expected)
        );
        assert_eq!(scan::original(expr), expected);
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(
        args.len() == 3 || args.len() == 4,
        "symbol-study validate|timing|measure CORPUS [OUTPUT]"
    );
    let mut snapshots = corpus::load(Path::new(&args[2]));
    snapshots.push((
        corpus::Entry {
            label: "synthetic-repeated-symbol-16000".into(),
            kind: "synthetic-control".into(),
            file: "generated-in-memory".into(),
            source_start: 0,
            source_end: 0,
        },
        Expr::call(Expr::symbol("f"), vec![Expr::symbol("repeated"); 16_000]),
    ));
    if args[1] == "validate" {
        for (_, expr) in &snapshots {
            let expected = scan::original(expr);
            validate::<Compact>(expr, expected);
            validate::<Safe>(expr, expected);
        }
        println!(
            "PASS: 128 snapshots; 512 full equalities and scan checks; COW append/source checks for all Call roots"
        );
        return;
    }
    if args[1] == "timing" {
        assert!(
            !cfg!(feature = "allocation-count"),
            "instrumented builds must not time operations"
        );
        println!("dataset\tround\toperation\trepresentation\titerations\tms\tchecksum");
        for (entry, expr) in &snapshots {
            if [
                "gMicroWideTraversal",
                "gMicroSortSymbolWidth16000",
                "gMicroAssociationPartWidth32000",
                "gMicroReplacePartRules",
                "rulesSimple",
                "synthetic-repeated-symbol-16000",
            ]
            .contains(&entry.label.as_str())
            {
                timing::run(&entry.label, expr, scan::original(expr));
            }
        }
        return;
    }
    assert_eq!(args[1], "measure");
    assert_eq!(args.len(), 4);
    #[cfg(feature = "allocation-count")]
    {
        let rows: Vec<_> = snapshots.iter().map(|(entry, expr)| serde_json::json!({"entry": entry,
            "measurements": [measure("original-normalized", || (rum_symbol_sharing::plain::<Safe>(expr).to_expr(), None)),
                measure("safe-plain", || (rum_symbol_sharing::plain::<Safe>(expr), None)),
                measure("safe-shared", || { let (node, stats) = rum_symbol_sharing::shared::<Safe>(expr, 128); (node, Some(stats)) }),
                measure("compact-plain", || (rum_symbol_sharing::plain::<Compact>(expr), None)),
                measure("compact-shared", || { let (node, stats) = rum_symbol_sharing::shared::<Compact>(expr, 128); (node, Some(stats)) })]})).collect();
        fs::write(
            &args[3],
            serde_json::to_string_pretty(&rows).unwrap() + "\n",
        )
        .unwrap();
        println!("PASS: 128 snapshots, 640 fresh graph windows returned to baseline");
    }
    #[cfg(not(feature = "allocation-count"))]
    panic!("measure requires allocation-count");
}

#[cfg(feature = "allocation-count")]
fn measure<T>(
    representation: &str,
    build: impl FnOnce() -> (T, Option<rum_symbol_sharing::Stats>),
) -> serde_json::Value {
    let start = allocation::start();
    let (root, stats) = build();
    let kept = allocation::snapshot();
    drop(root);
    let end = allocation::snapshot();
    assert_eq!(end.live, start.live);
    serde_json::json!({"representation": representation, "root_stack_bytes": std::mem::size_of::<T>(),
        "retained_requested_bytes": kept.live-start.live, "peak_requested_bytes": kept.peak-start.live,
        "allocation_calls": kept.calls-start.calls, "remaining_bytes": end.live-start.live,
        "stats": stats.map(|s| serde_json::json!({"symbol_visits":s.symbol_visits,"hits":s.hits,"entries":s.entries,"uncached":s.uncached}))})
}
