use rum_core::{Atom, BigInt, Expr, IntValue, Rational, SharedArgs, SharedExpr, Symbol};
use std::{env, path::Path};
#[cfg(feature = "allocation-count")]
#[path = "../../../recursive-expr/src/bin/study/allocation.rs"]
mod allocation;
#[allow(dead_code)]
#[path = "../../../expression-census/src/corpus.rs"]
mod corpus;
#[cfg(feature = "allocation-count")]
#[global_allocator]
static ALLOCATOR: allocation::Counting = allocation::Counting;
#[cfg(feature = "allocation-count")]
mod census;
mod graph;
mod scan;
mod timing;
use graph::{Case, Graph};
const DATASETS: [&str; 13] = [
    "gMicroWideTraversal",
    "gMicroSortSymbolWidth16000",
    "gMicroAssociationPartWidth32000",
    "gMicroReplacePartRules",
    "rulesSimple",
    "synthetic-repeated-symbol-16000",
    "synthetic-generated-symbol-16000",
    "synthetic-strings-16000",
    "synthetic-deep-256",
    "synthetic-shared-args-200",
    "synthetic-manual-big-small-16000",
    "synthetic-big-large-16000",
    "synthetic-rational-16000",
];
fn controls() -> Vec<Expr> {
    let repeated = Expr::call(Expr::symbol("f"), vec![Expr::symbol("repeated"); 16_000]);
    let generated = Expr::call(
        Expr::symbol("f"),
        (1..=16_000)
            .map(|i| Expr::Atom(Atom::Symbol(Symbol::with_identity("Global`", "g", i))))
            .collect(),
    );
    let strings = Expr::call(
        Expr::symbol("f"),
        (0..16_000).map(|_| Expr::string("payload")).collect(),
    );
    let mut deep = Expr::small_integer(7);
    for _ in 0..256 {
        deep = Expr::call(Expr::symbol("f"), vec![deep])
    }
    let h = SharedExpr::new(Expr::symbol("f"));
    let a = SharedArgs::new((0..1_000).map(Expr::small_integer).collect());
    let dag = Expr::call(
        Expr::symbol("List"),
        (0..200)
            .map(|_| Expr::call_shared(h.clone(), a.clone()))
            .collect(),
    );
    let big_small = Expr::call(
        Expr::symbol("f"),
        (0..16000)
            .map(|_| Expr::Atom(Atom::Integer(IntValue::Big(Box::new(BigInt::from(7))))))
            .collect(),
    );
    let big_large = Expr::call(
        Expr::symbol("f"),
        (0..16000)
            .map(|i| {
                Expr::Atom(Atom::Integer(IntValue::Big(Box::new(
                    (BigInt::from(1) << 100) + i,
                ))))
            })
            .collect(),
    );
    let rational = Expr::call(
        Expr::symbol("f"),
        (0..16000)
            .map(|i| {
                Expr::Atom(Atom::Rational(Box::new(
                    Rational::new((3 * i + 1).into(), 3.into()).unwrap(),
                )))
            })
            .collect(),
    );
    vec![
        repeated, generated, strings, deep, dag, big_small, big_large, rational,
    ]
}
fn main() {
    assert!(
        env::var_os("RUM_PROFILE_REPR").is_none(),
        "run with RUM_PROFILE_REPR unset"
    );
    let args: Vec<_> = env::args().collect();
    assert!(
        args.len() == 3 || args.len() == 4,
        "typed-study validate|measure|timing CORPUS [OUTPUT|RUN]"
    );
    let mut snapshots = corpus::load(Path::new(&args[2]));
    for (label, e) in DATASETS[5..].iter().zip(controls()) {
        snapshots.push((
            corpus::Entry {
                label: (*label).into(),
                kind: "synthetic-control".into(),
                file: "generated-in-memory".into(),
                source_start: 0,
                source_end: 0,
            },
            e,
        ));
    }
    if args[1] == "validate" {
        let mut cow_checks = 0;
        for (_, e) in &snapshots {
            let expected = scan::original(e);
            for case in Case::ALL {
                let n = Graph::build(case, e);
                assert_eq!(n.project(), *e);
                assert_eq!(n.scan(), expected);
                assert_eq!(n.scan_key(), expected);
                if matches!(e, Expr::Call { .. }) {
                    let edited = n.cow();
                    assert_eq!(edited.scan(), scan::appended_checksum(expected));
                    assert_eq!(edited.project(), scan::cow_original(e));
                    assert_eq!(n.project(), *e);
                    assert_eq!(n.scan(), expected);
                    cow_checks += 1;
                }
            }
            assert_eq!(scan::original(e), expected);
        }
        println!(
            "PASS: 135 snapshots; 675 full equalities; 1350 full/key scans; COW append/source checks for Call roots"
        );
        println!("PASS: {cow_checks} full COW/source equalities");
        println!("PASS: RUM_PROFILE_REPR unset; original disabled profiling branches remain");
        return;
    }
    if args[1] == "timing" {
        assert!(!cfg!(feature = "allocation-count"));
        let run: usize = args[3].parse().unwrap();
        assert!((1..=3).contains(&run));
        println!("dataset\tround\toperation\trepresentation\titerations\tms\tchecksum");
        let mut selected: Vec<_> = snapshots
            .iter()
            .filter(|(e, _)| DATASETS.contains(&e.label.as_str()))
            .collect();
        if run == 2 {
            selected.reverse()
        }
        for (entry, e) in selected {
            timing::run(&entry.label, e, scan::original(e), run)
        }
        return;
    }
    assert_eq!(args[1], "measure");
    assert_eq!(args.len(), 4);
    #[cfg(feature = "allocation-count")]
    {
        let rows: Vec<_> = snapshots
            .iter()
            .map(|(entry, e)| {
                serde_json::json!({"entry":entry,"physical":census::count(e),
            "measurements":Case::ALL.iter().map(|c|measure(*c,e)).collect::<Vec<_>>()})
            })
            .collect();
        std::fs::write(
            &args[3],
            serde_json::to_string_pretty(&rows).unwrap() + "\n",
        )
        .unwrap();
        println!("PASS: 135 snapshots; 675 fresh graph windows returned to baseline");
    }
    #[cfg(not(feature = "allocation-count"))]
    panic!("measure needs allocation-count");
}
#[cfg(feature = "allocation-count")]
fn measure(case: Case, source: &Expr) -> serde_json::Value {
    let start = allocation::start();
    let n = Graph::build(case, source);
    let kept = allocation::snapshot();
    drop(n);
    let end = allocation::snapshot();
    assert_eq!(end.live, start.live);
    serde_json::json!({"representation":case.name(),"root_stack_bytes":case.root_bytes(),
        "retained_requested_bytes":kept.live-start.live,"peak_requested_bytes":kept.peak-start.live,
        "allocation_calls":kept.calls-start.calls,"remaining_bytes":end.live-start.live})
}
