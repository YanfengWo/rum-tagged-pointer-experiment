use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rum_core::{Expr, SharedArgs, SharedExpr, Symbol};

use crate::runtime::suspension::trace::{
    FrameRootTrace, FrameRootVisitor, MANUAL_FRAME_ROOT_TRACE_IMPLS,
};
use crate::runtime::{
    AssociationEntryForm, BlockSymbolState, FileIoError, MatcherArg, MatcherNode, SymbolExprMap,
};

#[derive(Default)]
struct RecordedRoots {
    exprs: Vec<*const Expr>,
    symbols: Vec<*const Symbol>,
}

impl FrameRootVisitor for RecordedRoots {
    fn visit_expr(&mut self, expr: &Expr) {
        self.exprs.push(expr as *const Expr);
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        self.symbols.push(symbol as *const Symbol);
    }
}

#[derive(rum_derive::FrameRootTrace)]
struct NestedRoots {
    expression: Expr,
    symbol: Symbol,
}

#[derive(rum_derive::FrameRootTrace)]
struct NestedFrame {
    direct: Expr,
    nested: Option<Box<NestedRoots>>,
}

#[derive(rum_derive::FrameRootTrace)]
struct ContainerRoots {
    vector: Vec<Expr>,
    queue: VecDeque<Symbol>,
    optional: Option<Box<Expr>>,
    arc_slice: Arc<[Expr]>,
    hash_map: HashMap<Symbol, Expr>,
    tree_map: BTreeMap<Symbol, Expr>,
    tuple: (Expr, Symbol),
}

#[derive(rum_derive::FrameRootTrace)]
enum VariantRoots {
    Named { expression: Expr, symbol: Symbol },
    Tuple(Expr, Symbol),
    Unit,
}

#[derive(rum_derive::FrameRootTrace)]
struct GenericRoots<T, const N: usize> {
    values: Vec<T>,
    fixed: [Expr; N],
}

mod derived_inventory;
use derived_inventory::DERIVED_FRAME_ROOT_TRACE_TYPES;

#[test]
fn expr_is_one_whole_root_and_bare_symbol_is_an_independent_root() {
    let expression = Expr::call(Expr::symbol("head"), vec![Expr::symbol("argument")]);
    let symbol = Symbol::from_name("bare");
    let mut roots = RecordedRoots::default();

    expression.trace_roots(&mut roots);
    symbol.trace_roots(&mut roots);

    assert_eq!(roots.exprs, vec![&expression as *const Expr]);
    assert_eq!(roots.symbols, vec![&symbol as *const Symbol]);
}

#[test]
fn shared_expression_types_borrow_their_expression_roots() {
    let shared_expr = SharedExpr::new(Expr::symbol("shared"));
    let shared_args = SharedArgs::new(vec![Expr::symbol("first"), Expr::symbol("second")]);
    let mut roots = RecordedRoots::default();

    shared_expr.trace_roots(&mut roots);
    shared_args.trace_roots(&mut roots);

    assert_eq!(
        roots.exprs,
        vec![
            shared_expr.as_ref() as *const Expr,
            &shared_args[0] as *const Expr,
            &shared_args[1] as *const Expr,
        ]
    );
    assert!(roots.symbols.is_empty());
}

#[test]
fn nested_derived_fields_are_all_traversed() {
    let frame = NestedFrame {
        direct: Expr::symbol("direct"),
        nested: Some(Box::new(NestedRoots {
            expression: Expr::symbol("nested"),
            symbol: Symbol::from_name("bare"),
        })),
    };
    let nested = frame.nested.as_ref().unwrap();
    let mut roots = RecordedRoots::default();

    frame.trace_roots(&mut roots);

    assert_eq!(
        roots.exprs,
        vec![
            &frame.direct as *const Expr,
            &nested.expression as *const Expr,
        ]
    );
    assert_eq!(
        roots.symbols.len(),
        1,
        "the nested Symbol field must be traversed"
    );
    assert_eq!(roots.symbols, vec![&nested.symbol as *const Symbol]);
}

#[test]
fn container_implementations_visit_keys_values_and_elements() {
    let map_symbol = Symbol::from_name("hash-key");
    let tree_symbol = Symbol::from_name("tree-key");
    let mut hash_map = HashMap::new();
    hash_map.insert(map_symbol, Expr::symbol("hash-value"));
    let mut tree_map = BTreeMap::new();
    tree_map.insert(tree_symbol, Expr::symbol("tree-value"));
    let frame = ContainerRoots {
        vector: vec![Expr::symbol("vector")],
        queue: VecDeque::from([Symbol::from_name("queue")]),
        optional: Some(Box::new(Expr::symbol("optional"))),
        arc_slice: Arc::from([Expr::symbol("arc")]),
        hash_map,
        tree_map,
        tuple: (Expr::symbol("tuple"), Symbol::from_name("tuple-symbol")),
    };
    let mut roots = RecordedRoots::default();

    frame.trace_roots(&mut roots);

    assert_eq!(roots.exprs.len(), 6);
    assert_eq!(roots.symbols.len(), 4);
}

#[test]
fn enum_shapes_and_generic_arrays_trace_without_special_cases() {
    let variants = [
        VariantRoots::Named {
            expression: Expr::symbol("named"),
            symbol: Symbol::from_name("named-symbol"),
        },
        VariantRoots::Tuple(Expr::symbol("tuple"), Symbol::from_name("tuple-symbol")),
        VariantRoots::Unit,
    ];
    let generic = GenericRoots {
        values: vec![Expr::symbol("generic")],
        fixed: [Expr::symbol("array")],
    };
    let mut roots = RecordedRoots::default();

    for variant in variants {
        variant.trace_roots(&mut roots);
    }
    generic.trace_roots(&mut roots);

    assert_eq!(roots.exprs.len(), 4);
    assert_eq!(roots.symbols.len(), 2);
}

#[test]
fn matcher_binding_table_visits_the_exact_symbol_and_expression_roots() {
    let mut bindings = SymbolExprMap::default();
    bindings.insert(Symbol::from_name("binding"), Expr::symbol("binding-value"));
    let (symbol, expression) = bindings.iter().next().expect("binding should exist");
    let mut roots = RecordedRoots::default();

    bindings.trace_roots(&mut roots);

    assert_eq!(roots.exprs, vec![expression as *const Expr]);
    assert_eq!(roots.symbols, vec![symbol as *const Symbol]);
}

#[test]
fn recursive_matcher_arg_visits_every_nested_root_exactly_once() {
    let matcher = MatcherArg::PatternSequenceGroup(vec![
        MatcherArg::Single(MatcherNode::Named {
            name: Symbol::from_name("named-pattern"),
            inner: Box::new(MatcherNode::Exact(Expr::symbol("exact-pattern"))),
        }),
        MatcherArg::PatternSequenceGroup(vec![MatcherArg::Single(MatcherNode::Blank {
            head_constraint: Some(Expr::symbol("head-constraint")),
        })]),
    ]);
    let MatcherArg::PatternSequenceGroup(arguments) = &matcher else {
        unreachable!("test matcher should be a pattern-sequence group");
    };
    let MatcherArg::Single(MatcherNode::Named { name, inner }) = &arguments[0] else {
        unreachable!("first test matcher should be named");
    };
    let MatcherNode::Exact(exact) = inner.as_ref() else {
        unreachable!("named matcher should wrap an exact matcher");
    };
    let MatcherArg::PatternSequenceGroup(nested) = &arguments[1] else {
        unreachable!("second test matcher should be recursive");
    };
    let MatcherArg::Single(MatcherNode::Blank {
        head_constraint: Some(head_constraint),
    }) = &nested[0]
    else {
        unreachable!("recursive matcher should hold a head constraint");
    };
    let mut roots = RecordedRoots::default();

    matcher.trace_roots(&mut roots);

    assert_eq!(
        roots.exprs,
        vec![exact as *const Expr, head_constraint as *const Expr]
    );
    assert_eq!(roots.symbols, vec![name as *const Symbol]);
}

#[test]
fn association_entry_form_visits_key_and_value_roots_in_field_order() {
    let entry = AssociationEntryForm {
        key: Expr::symbol("association-key"),
        value: Expr::symbol("association-value"),
        delayed: true,
    };
    let mut roots = RecordedRoots::default();

    entry.trace_roots(&mut roots);

    assert_eq!(
        roots.exprs,
        vec![&entry.key as *const Expr, &entry.value as *const Expr]
    );
    assert!(roots.symbols.is_empty());
}

#[test]
fn block_symbol_state_map_visits_the_bare_symbol_key() {
    let mut states = BTreeMap::new();
    states.insert(Symbol::from_name("blocked"), BlockSymbolState::default());
    let symbol = states
        .first_key_value()
        .expect("block state should contain one symbol")
        .0;
    let mut roots = RecordedRoots::default();

    states.trace_roots(&mut roots);

    assert!(roots.exprs.is_empty());
    assert_eq!(roots.symbols, vec![symbol as *const Symbol]);
}

#[test]
fn file_io_error_is_the_zero_root_negative_calibration_target() {
    let error = FileIoError::new("calibration");
    let mut roots = RecordedRoots::default();

    error.trace_roots(&mut roots);

    println!("file_io_error_expr_roots={}", roots.exprs.len());
    assert_eq!(roots.exprs.len(), 0);
    assert!(roots.symbols.is_empty());
}

#[test]
fn derived_frame_root_trace_inventory_matches_the_closed_capture_graph() {
    assert!(
        DERIVED_FRAME_ROOT_TRACE_TYPES
            .windows(2)
            .all(|entries| entries[0] < entries[1]),
        "the derived FrameRootTrace inventory must stay sorted"
    );

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should live under the workspace crates directory");
    let mut files = Vec::new();
    collect_rust_sources(&workspace.join("crates/rum-runtime/src"), &mut files);
    collect_rust_sources(&workspace.join("crates/rum-stdlib/src"), &mut files);

    let mut derived = files
        .iter()
        .flat_map(|path| derived_frame_root_trace_types(workspace, path))
        .collect::<Vec<_>>();
    derived.sort();
    let expected = DERIVED_FRAME_ROOT_TRACE_TYPES
        .iter()
        .map(|entry| (*entry).to_string())
        .collect::<Vec<_>>();

    assert_eq!(derived, expected);
}

#[test]
fn manual_frame_root_trace_implementations_match_sorted_allowlist() {
    assert!(
        MANUAL_FRAME_ROOT_TRACE_IMPLS
            .windows(2)
            .all(|entries| entries[0] < entries[1]),
        "the manual FrameRootTrace implementation allowlist must stay sorted"
    );

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should live under the workspace crates directory");
    let mut files = Vec::new();
    collect_rust_sources(&workspace.join("crates/rum-runtime/src"), &mut files);
    collect_rust_sources(&workspace.join("crates/rum-stdlib/src"), &mut files);

    let mut implementations = files
        .iter()
        .flat_map(|path| manual_frame_root_trace_impls(path))
        .collect::<Vec<_>>();
    implementations.sort();

    assert_eq!(implementations, MANUAL_FRAME_ROOT_TRACE_IMPLS);
}

fn collect_rust_sources(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("source gate should read source directories") {
            let entry = entry.expect("source gate should read source entries");
            collect_rust_sources(&entry.path(), files);
        }
        return;
    }

    let normalized = path.display().to_string().replace('\\', "/");
    if path.extension().and_then(|extension| extension.to_str()) == Some("rs")
        && !normalized.contains("/tests/")
    {
        files.push(path.to_path_buf());
    }
}

fn manual_frame_root_trace_impls(path: &Path) -> Vec<String> {
    let source = fs::read_to_string(path).expect("source gate should read Rust source");
    let mut implementations = Vec::new();
    let mut header = None::<String>;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if header.is_none() && trimmed.starts_with("impl") {
            header = Some(trimmed.to_string());
        } else if let Some(header) = &mut header {
            header.push(' ');
            header.push_str(trimmed);
        }

        let Some(current) = &header else {
            continue;
        };
        if !current.contains('{') {
            continue;
        }

        let normalized = current
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        if let Some((_, implementation)) = normalized.split_once("FrameRootTracefor") {
            let end = implementation
                .find("where")
                .or_else(|| implementation.find('{'))
                .expect("an impl header should end in a body");
            implementations.push(implementation[..end].to_string());
        }
        header = None;
    }

    implementations
}

fn derived_frame_root_trace_types(workspace: &Path, path: &Path) -> Vec<String> {
    let source = fs::read_to_string(path).expect("source gate should read Rust source");
    let relative_path = path
        .strip_prefix(workspace)
        .expect("source path should be inside the workspace")
        .display()
        .to_string()
        .replace('\\', "/");
    let mut types = Vec::new();
    let mut derive_attribute = String::new();
    let mut reading_derive = false;
    let mut derives_frame_root_trace = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if !reading_derive && trimmed.starts_with("#[derive(") {
            derive_attribute.clear();
            derive_attribute.push_str(trimmed);
            reading_derive = !trimmed.ends_with(")]");
            if !reading_derive {
                derives_frame_root_trace = derive_attribute.contains("FrameRootTrace");
            }
            continue;
        }

        if reading_derive {
            derive_attribute.push_str(trimmed);
            if trimmed.ends_with(")]") {
                reading_derive = false;
                derives_frame_root_trace = derive_attribute.contains("FrameRootTrace");
            }
            continue;
        }

        if !derives_frame_root_trace {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("#[") {
            continue;
        }

        if let Some(type_name) = declared_type_name(trimmed) {
            types.push(format!("{relative_path}:{type_name}"));
        }
        derives_frame_root_trace = false;
    }

    types
}

fn declared_type_name(declaration: &str) -> Option<&str> {
    let (_, name) = ["struct ", "enum "]
        .into_iter()
        .filter_map(|keyword| {
            declaration
                .find(keyword)
                .map(|index| (index, &declaration[index + keyword.len()..]))
        })
        .min_by_key(|(index, _)| *index)?;

    let name_len = name
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .map(char::len_utf8)
        .sum::<usize>();
    (name_len > 0).then_some(&name[..name_len])
}
