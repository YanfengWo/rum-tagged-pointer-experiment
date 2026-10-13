mod collector;
#[path = "../../expression-census/src/corpus.rs"]
#[allow(dead_code)]
mod corpus;
mod graph;
mod timing;
mod walk;
use graph::{Case, Graph};
use rum_core::{Atom, Expr, Symbol};
use std::{collections::hash_map::RandomState, env, path::Path};
const LABELS: [&str; 12] = [
    "gMicroWideTraversal",
    "gMicroSortSymbolWidth16000",
    "gMicroAssociationPartWidth32000",
    "gMicroReplacePartRules",
    "rulesSimple",
    "repeat-16000",
    "generated-16000",
    "strings-16000",
    "contexts-16000",
    "shared-200",
    "unique-16",
    "unique-17",
];
fn controls() -> Vec<Expr> {
    let call = |v| Expr::call(Expr::symbol("f"), v);
    vec![
        call(vec![Expr::symbol("repeated"); 16000]),
        call(
            (1..=16000)
                .map(|i| Expr::Atom(Atom::Symbol(Symbol::with_identity("Global`", "g", i))))
                .collect(),
        ),
        call(vec![Expr::string("payload"); 16000]),
        call(
            (0..16000)
                .map(|i| {
                    Expr::Atom(Atom::Symbol(Symbol::new(
                        if i % 2 == 0 { "Global`" } else { "Other`" },
                        &format!("s{}", (i / 2) % 128),
                    )))
                })
                .collect(),
        ),
        {
            let e = call(
                (0..1000)
                    .map(|i| Expr::symbol(&format!("s{}", i % 8)))
                    .collect(),
            );
            call(vec![e; 200])
        },
        call((0..15).map(|i| Expr::symbol(&format!("b{i}"))).collect()),
        call((0..16).map(|i| Expr::symbol(&format!("b{i}"))).collect()),
    ]
}
fn hint(e: &Expr) -> usize {
    match e {
        Expr::Call { args, .. } => args.len(),
        _ => 0,
    }
}
fn oracle(e: &Expr) -> Vec<Symbol> {
    let input = walk::Original::new(e).collect::<Vec<_>>();
    rum_runtime::research_unique_symbols(&input, hint(e))
}
fn main() {
    assert!(env::var_os("RUM_PROFILE_REPR").is_none());
    let args = env::args().collect::<Vec<_>>();
    let mut snapshots = corpus::load(Path::new(&args[2]));
    for (label, e) in LABELS[5..].iter().zip(controls()) {
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
    let seed = RandomState::new();
    if args[1] == "validate" {
        let mut checks = 0;
        for (_, e) in &snapshots {
            let expected = oracle(e);
            for case in Case::ALL {
                assert_eq!(
                    Graph::build(case, e).collect(case, hint(e), &seed),
                    expected
                );
                checks += 1;
            }
        }
        println!(
            "PASS: {} snapshots; {checks} complete ordered Symbol results match unmodified runtime collector",
            snapshots.len()
        );
        return;
    }
    assert_eq!(args[1], "timing");
    let run: usize = args[3].parse().unwrap();
    assert!((1..=3).contains(&run));
    println!("dataset\tround\toperation\trepresentation\titerations\tms\tchecksum");
    let mut selected = snapshots
        .iter()
        .filter(|(entry, _)| LABELS.contains(&entry.label.as_str()))
        .collect::<Vec<_>>();
    if run == 2 {
        selected.reverse();
    }
    for (entry, e) in selected {
        timing::run(&entry.label, e, hint(e), &oracle(e), run, &seed);
    }
}
#[cfg(test)]
mod tests;
