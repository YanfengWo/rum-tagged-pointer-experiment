use rum_core::Expr;
use rum_recursive_expr::{Compact, Safe, Tree, to_expr};
use std::{env, path::Path};
#[cfg(feature = "allocation-count")]
#[path = "../../../recursive-expr/src/bin/study/allocation.rs"]
mod allocation;
#[allow(dead_code)]
#[path = "../../../expression-census/src/corpus.rs"]
mod corpus;
mod policy;
#[path = "../../../symbol-sharing/src/bin/scan.rs"]
mod scan;
mod timing;
#[cfg(feature = "allocation-count")]
#[global_allocator]
static ALLOCATOR: allocation::Counting = allocation::Counting;

const DATASETS: [&str; 8] = [
    "gMicroWideTraversal",
    "gMicroSortSymbolWidth16000",
    "gMicroAssociationPartWidth32000",
    "gMicroReplacePartRules",
    "rulesSimple",
    "synthetic-repeated-symbol-16000",
    "synthetic-late-repeats-16000",
    "synthetic-early-repeats-16000",
];
fn validate<N: Tree>(expr: &Expr) {
    let expected = scan::original(expr);
    if matches!(expr, Expr::Call { .. }) {
        assert_eq!(
            scan::original(&scan::cow_original(expr)),
            scan::appended_checksum(expected)
        );
        assert_eq!(scan::original(expr), expected);
    }
    for p in policy::POLICIES {
        let (node, record) = policy::build::<N>(expr, p);
        if let Some(probe) = record.probe {
            assert!(probe.nodes <= rum_selective_sharing::NODE_BUDGET);
            assert_eq!(
                record.sharing.is_some(),
                probe.decision == rum_selective_sharing::Decision::Shared
            );
        }
        if let Some(stats) = record.sharing {
            assert!(stats.entries <= 128);
            assert_eq!(
                stats.symbol_visits,
                stats.entries + stats.hits + stats.uncached
            );
        }
        assert_eq!(to_expr(&node), *expr);
        assert_eq!(scan::tree(&node), expected);
        if matches!(expr, Expr::Call { .. }) {
            assert_eq!(
                scan::tree(&scan::cow_tree(&node)),
                scan::appended_checksum(expected)
            );
            assert_eq!(scan::tree(&node), expected);
        }
    }
}
fn controls() -> Vec<Expr> {
    let repeated = vec![Expr::symbol("repeated"); 16_000];
    let mut late: Vec<_> = (0..300)
        .map(|n| Expr::symbol(&format!("late{n}")))
        .collect();
    late.extend(vec![Expr::symbol("f"); 15_700]);
    let mut early = vec![Expr::symbol("f"); 8];
    early.extend((0..15_992).map(|n| Expr::symbol(&format!("early{n}"))));
    [repeated, late, early]
        .into_iter()
        .map(|args| Expr::call(Expr::symbol("f"), args))
        .collect()
}
fn main() {
    let args: Vec<_> = env::args().collect();
    assert!(
        args.len() == 3 || args.len() == 4,
        "selective-study validate|timing|timing-reverse|measure CORPUS [OUTPUT]"
    );
    let mut snapshots = corpus::load(Path::new(&args[2]));
    for (label, expr) in DATASETS[5..].iter().zip(controls()) {
        snapshots.push((
            corpus::Entry {
                label: (*label).into(),
                kind: "synthetic-control".into(),
                file: "generated-in-memory".into(),
                source_start: 0,
                source_end: 0,
            },
            expr,
        ));
    }
    if args[1] == "validate" {
        for (_, expr) in &snapshots {
            validate::<Compact>(expr);
            validate::<Safe>(expr);
        }
        println!(
            "PASS: 130 snapshots; 780 full equalities and scan checks; COW append/source checks for Call roots"
        );
        return;
    }
    if args[1].starts_with("timing") {
        assert!(!cfg!(feature = "allocation-count"));
        println!("dataset\tround\trepresentation\titerations\tms");
        let mut selected: Vec<_> = snapshots
            .iter()
            .filter(|(e, _)| DATASETS.contains(&e.label.as_str()))
            .collect();
        let reversed = args[1] == "timing-reverse";
        if reversed {
            selected.reverse();
        }
        for (entry, expr) in selected {
            timing::run(&entry.label, expr, reversed);
        }
        return;
    }
    assert_eq!(args[1], "measure");
    assert_eq!(args.len(), 4);
    #[cfg(feature = "allocation-count")]
    {
        let rows: Vec<_> = snapshots
            .iter()
            .map(|(entry, expr)| {
                let mut measurements = vec![measure("original-normalized", || {
                    (
                        rum_symbol_sharing::plain::<Safe>(expr).to_expr(),
                        policy::Record::default(),
                    )
                })];
                for p in policy::POLICIES {
                    measurements.push(measure(&format!("safe-{}", p.name()), || {
                        policy::build::<Safe>(expr, p)
                    }));
                    measurements.push(measure(&format!("compact-{}", p.name()), || {
                        policy::build::<Compact>(expr, p)
                    }));
                }
                serde_json::json!({"entry":entry,"measurements":measurements})
            })
            .collect();
        std::fs::write(
            &args[3],
            serde_json::to_string_pretty(&rows).unwrap() + "\n",
        )
        .unwrap();
        println!("PASS: 130 snapshots; 910 fresh graph windows returned to baseline");
    }
    #[cfg(not(feature = "allocation-count"))]
    panic!("measure requires allocation-count");
}
#[cfg(feature = "allocation-count")]
fn measure<T>(name: &str, build: impl FnOnce() -> (T, policy::Record)) -> serde_json::Value {
    let start = allocation::start();
    let (node, record) = build();
    let kept = allocation::snapshot();
    drop(node);
    let end = allocation::snapshot();
    assert_eq!(end.live, start.live);
    serde_json::json!({"representation":name,"root_stack_bytes":std::mem::size_of::<T>(),
        "retained_requested_bytes":kept.live-start.live,"peak_requested_bytes":kept.peak-start.live,
        "allocation_calls":kept.calls-start.calls,"remaining_bytes":end.live-start.live,"record":record.json()})
}
