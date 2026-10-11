//! Extract exact source spans from the pinned rum benchmark's generated source.
use rum_core::Expr;
use rum_runtime::Session;
use rum_syntax::{BinaryOp, ScriptForm, SyntaxExpr, SyntaxExprKind as Kind, parse_script_forms};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const RESULTS: [&str; 15] = [
    "list10",
    "rulesSimple",
    "gMicroWideTraversal",
    "gMicroWideAssociation",
    "gMicroAssociationPartWidth4000",
    "gMicroAssociationPartWidth32000",
    "gMicroReplacePartRules",
    "gMicroStringLiteralHaystack",
    "gMicroStringTrimLiteralList4000",
    "gMicroStringTrimLiteralList32000",
    "gMicroSortDefaultWide",
    "gMicroSortSymbolWidth4000",
    "gMicroSortSymbolWidth16000",
    "gMicroSortCallbackSmall",
    "gMicroSortCallbackLarge",
];

#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub label: String,
    pub kind: String,
    pub file: String,
    pub source_start: usize,
    pub source_end: usize,
}

fn children(node: &SyntaxExpr) -> Vec<&SyntaxExpr> {
    match &node.kind {
        Kind::Call { head, args } => std::iter::once(head.as_ref()).chain(args.iter()).collect(),
        Kind::List(items) | Kind::Association(items) => items.iter().collect(),
        Kind::Binary { lhs, rhs, .. } => vec![lhs, rhs],
        Kind::Pattern { name, pattern } => vec![name, pattern],
        Kind::Prefix { expr, .. } | Kind::Postfix { expr, .. } => vec![expr],
        Kind::Blank { head, .. } => head.iter().map(|h| h.as_ref()).collect(),
        Kind::ComparisonChain { operands, .. } => operands.iter().collect(),
        _ => vec![],
    }
}

pub fn extract(source_file: &Path, directory: &Path) {
    let source = fs::read_to_string(source_file).unwrap();
    fs::create_dir_all(directory.join("inputs")).unwrap();
    fs::create_dir_all(directory.join("results")).unwrap();
    let mut entries = Vec::new();
    let mut helpers = Vec::new();
    for form in parse_script_forms(&source).unwrap() {
        let ScriptForm::Parsed { syntax, .. } = form else {
            panic!("unparsed benchmark form")
        };
        let mut pending = vec![&syntax];
        while let Some(node) = pending.pop() {
            if let Kind::Call { head, args } = &node.kind {
                if matches!(&head.kind, Kind::Symbol(name) if name == "rumMicroBench" || name == "rumMicroOnce")
                {
                    if let Some(SyntaxExpr {
                        kind: Kind::String(label),
                        ..
                    }) = args.first()
                    {
                        assert!(args.len() == 2 || args.len() == 3);
                        let body = args.last().unwrap();
                        save(
                            directory,
                            &source,
                            label,
                            "lowered-input",
                            "inputs",
                            body,
                            &mut entries,
                        );
                        continue;
                    }
                }
            }
            if let Kind::Binary {
                op: BinaryOp::Set | BinaryOp::SetDelayed,
                lhs,
                rhs,
            } = &node.kind
            {
                if let Kind::Symbol(name) = &lhs.kind {
                    // This final label emits its own timing row instead of using
                    // rumMicroBench. Extract the exact body of AbsoluteTiming.
                    if name == "gMicroRetainedModuleGcSeconds" {
                        if let Kind::Call { head, args } = &rhs.kind {
                            assert!(matches!(&head.kind, Kind::Symbol(name) if name == "First"));
                            assert_eq!(args.len(), 1);
                            let Kind::Call { head, args } = &args[0].kind else {
                                panic!("missing custom AbsoluteTiming")
                            };
                            assert!(
                                matches!(&head.kind, Kind::Symbol(name) if name == "AbsoluteTiming")
                            );
                            assert_eq!(args.len(), 1);
                            save(
                                directory,
                                &source,
                                "module-retained-gensym-gc-batch",
                                "lowered-input",
                                "inputs",
                                &args[0],
                                &mut entries,
                            );
                        }
                    }
                    if RESULTS.contains(&name.as_str()) {
                        save(
                            directory,
                            &source,
                            name,
                            "evaluated-result",
                            "results",
                            rhs,
                            &mut entries,
                        );
                    }
                    if name == "rumMicroScale" {
                        helpers.push(&source[node.span.start..node.span.end]);
                    }
                }
                if matches!(&lhs.kind, Kind::Call { head, .. } if matches!(&head.kind, Kind::Symbol(name) if name == "rumMicroN"))
                {
                    helpers.push(&source[node.span.start..node.span.end]);
                }
            }
            pending.extend(children(node));
        }
    }
    assert_eq!(
        entries.iter().filter(|e| e.kind == "lowered-input").count(),
        112
    );
    assert_eq!(
        entries
            .iter()
            .filter(|e| e.kind == "evaluated-result")
            .count(),
        RESULTS.len()
    );
    assert_eq!(helpers.len(), 2);
    fs::write(directory.join("setup.rum"), helpers.join(";\n") + ";\n").unwrap();
    entries.sort_by(|a, b| (&a.kind, &a.label).cmp(&(&b.kind, &b.label)));
    fs::write(
        directory.join("index.json"),
        serde_json::to_string_pretty(&entries).unwrap() + "\n",
    )
    .unwrap();
}
fn save(
    directory: &Path,
    source: &str,
    label: &str,
    kind: &str,
    folder: &str,
    node: &SyntaxExpr,
    entries: &mut Vec<Entry>,
) {
    assert!(
        label
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    );
    assert!(!entries.iter().any(|e| e.label == label && e.kind == kind));
    let file = format!("{folder}/{label}.rum");
    fs::write(
        directory.join(&file),
        source[node.span.start..node.span.end].to_owned() + "\n",
    )
    .unwrap();
    entries.push(Entry {
        label: label.into(),
        kind: kind.into(),
        file,
        source_start: node.span.start,
        source_end: node.span.end,
    });
}

pub fn load(directory: &Path) -> Vec<(Entry, Expr)> {
    let entries: Vec<Entry> =
        serde_json::from_str(&fs::read_to_string(directory.join("index.json")).unwrap()).unwrap();
    let mut session = Session::new();
    rum_stdlib::install_core(&mut session);
    let setup = session
        .eval_text(&fs::read_to_string(directory.join("setup.rum")).unwrap())
        .unwrap();
    assert!(setup.termination.is_none() && setup.messages.is_empty() && setup.prints.is_empty());
    entries
        .into_iter()
        .map(|entry| {
            let source = fs::read_to_string(directory.join(&entry.file)).unwrap();
            let expr = if entry.kind == "lowered-input" {
                session.lower_request(&source).unwrap().expr
            } else {
                let outcome = session.eval_text(&source).unwrap();
                assert!(
                    outcome.termination.is_none()
                        && outcome.messages.is_empty()
                        && outcome.prints.is_empty(),
                    "{}",
                    entry.label
                );
                outcome.value
            };
            (entry, expr)
        })
        .collect()
}
