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

const DERIVED_FRAME_ROOT_TRACE_TYPES: &[&str] = &[
    "crates/rum-runtime/src/runtime/debug_tools/reap.rs:ReapBucketRenderState",
    "crates/rum-runtime/src/runtime/debug_tools/reap.rs:ReapScopeRenderState",
    "crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs:EvalCapturedOutcome",
    "crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs:TestCaseSpec",
    "crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs:TestReportOptions",
    "crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:TestReportCaseEvalState",
    "crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:TestReportCaseOutputState",
    "crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:TestReportEvalState",
    "crates/rum-runtime/src/runtime/debug_tools/timing.rs:RepeatedTimingEvalState",
    "crates/rum-runtime/src/runtime/debug_tools/trace.rs:TraceEvalState",
    "crates/rum-runtime/src/runtime/debug_tools/trace.rs:TraceOutput",
    "crates/rum-runtime/src/runtime/definitions/application/replace_all/plan.rs:RewriteRuleExactIndex",
    "crates/rum-runtime/src/runtime/definitions/application/replace_all/plan.rs:RewriteRulePlan",
    "crates/rum-runtime/src/runtime/definitions/application/replace_all/plan.rs:RewriteRulePlanData",
    "crates/rum-runtime/src/runtime/definitions/definition_updates.rs:HeldUpdateReturn",
    "crates/rum-runtime/src/runtime/definitions/definition_updates.rs:HeldUpdateTarget",
    "crates/rum-runtime/src/runtime/definitions/part_assignment.rs:PositionalPartAssignmentTarget",
    "crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:OwnedGuards",
    "crates/rum-runtime/src/runtime/eval_call_prepare.rs:UnevaluatedRestorePlan",
    "crates/rum-runtime/src/runtime/file_io.rs:FileIoError",
    "crates/rum-runtime/src/runtime/kernel_data/association_data.rs:AssociationEntryForm",
    "crates/rum-runtime/src/runtime/kernel_data/association_data.rs:AssociationSetLValueTarget",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:DoSpec",
    "crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:ExceptionHandler",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs:GuardedReplacePartTarget",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs:GuardedReplacePartTargetState",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs:ReplacePartHeadsMode",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs:ReplacePartPositionSpec",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs:ReplacePartRule",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs:ReplacePartTarget",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:AtomicPartSpec",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:PartSelector",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:PartSpanSpec",
    "crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:SpanEndpoint",
    "crates/rum-runtime/src/runtime/metadata.rs:AttributeSet",
    "crates/rum-runtime/src/runtime/model/definitions.rs:DownValue",
    "crates/rum-runtime/src/runtime/model/definitions.rs:DownValueShallowArg",
    "crates/rum-runtime/src/runtime/model/definitions.rs:DownValueShallowSignature",
    "crates/rum-runtime/src/runtime/model/definitions.rs:RewriteRule",
    "crates/rum-runtime/src/runtime/model/definitions.rs:SubValue",
    "crates/rum-runtime/src/runtime/model/definitions.rs:SymbolDefinition",
    "crates/rum-runtime/src/runtime/model/definitions.rs:UpValue",
    "crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs:DownValueDispatchIndex",
    "crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs:DownValueDispatchIndexCache",
    "crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs:DownValueDispatchMetadata",
    "crates/rum-runtime/src/runtime/model/definitions/tagged_value_store.rs:TaggedValueStoreIndex",
    "crates/rum-runtime/src/runtime/model/definitions/tagged_value_store.rs:TaggedValueStoreMetadata",
    "crates/rum-runtime/src/runtime/model/messages.rs:MessageDefinition",
    "crates/rum-runtime/src/runtime/model/messages.rs:MessageNameKey",
    "crates/rum-runtime/src/runtime/model/messages.rs:RuntimeInputKind",
    "crates/rum-runtime/src/runtime/model/messages.rs:RuntimeInputRequest",
    "crates/rum-runtime/src/runtime/model/messages.rs:RuntimeMessage",
    "crates/rum-runtime/src/runtime/model/traversal.rs:CasesLevelSpec",
    "crates/rum-runtime/src/runtime/model/traversal.rs:CasesSpec",
    "crates/rum-runtime/src/runtime/options.rs:OptionValueScope",
    "crates/rum-runtime/src/runtime/options.rs:SymbolOptionEntry",
    "crates/rum-runtime/src/runtime/options.rs:SymbolOptions",
    "crates/rum-runtime/src/runtime/options/builtins.rs:OptionValueQuery",
    "crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/candidates.rs:OrderlessCandidateChoice",
    "crates/rum-runtime/src/runtime/scoping/packages.rs:PackageInstallPlan",
    "crates/rum-runtime/src/runtime/scoping/packages.rs:SourceUnit",
    "crates/rum-runtime/src/runtime/session/session_state.rs:BlockSymbolState",
    "crates/rum-runtime/src/runtime/session/session_state.rs:CallDispatchState",
    "crates/rum-runtime/src/runtime/session/session_state.rs:CompiledOptionalDefault",
    "crates/rum-runtime/src/runtime/session/session_state.rs:CompiledOptionalPattern",
    "crates/rum-runtime/src/runtime/session/session_state.rs:CompiledSequencePattern",
    "crates/rum-runtime/src/runtime/session/session_state.rs:DefinitionConditionMatcher",
    "crates/rum-runtime/src/runtime/session/session_state.rs:EvalError",
    "crates/rum-runtime/src/runtime/session/session_state.rs:MatcherArg",
    "crates/rum-runtime/src/runtime/session/session_state.rs:MatcherNode",
    "crates/rum-runtime/src/runtime/session/session_state.rs:ReapBucket",
    "crates/rum-runtime/src/runtime/session/session_state.rs:ReapBucketGroup",
    "crates/rum-runtime/src/runtime/session/session_state.rs:SequenceGreediness",
    "crates/rum-runtime/src/runtime/session/session_state.rs:SequencePatternCheck",
    "crates/rum-stdlib/src/associations/aggregate.rs:CatenateStep",
    "crates/rum-stdlib/src/associations/basic.rs:LookupOutcome",
    "crates/rum-stdlib/src/associations/basic.rs:LookupPlan",
    "crates/rum-stdlib/src/associations/basic.rs:PreparedLookup",
    "crates/rum-stdlib/src/associations/grouping.rs:GroupBySpec",
    "crates/rum-stdlib/src/associations/grouping.rs:IndexedGroups",
    "crates/rum-stdlib/src/associations/resumable/key_sort.rs:KeySortByEntry",
    "crates/rum-stdlib/src/associations/resumable/key_sort.rs:KeySortByOrderingState",
    "crates/rum-stdlib/src/associations/transforms/key_collections.rs:KeyUnionPlan",
    "crates/rum-stdlib/src/functional_ops/arrays/generators.rs:ArrayEvalState",
    "crates/rum-stdlib/src/functional_ops/arrays/generators.rs:OuterEvalState",
    "crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:InnerEvalState",
    "crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:TensorCallbackEvalState",
    "crates/rum-stdlib/src/functional_ops/iteration.rs:FixedPointEvalState",
    "crates/rum-stdlib/src/functional_ops/iteration.rs:NestEvalState",
    "crates/rum-stdlib/src/functional_ops/iteration.rs:NestWhileConfig",
    "crates/rum-stdlib/src/functional_ops/iteration.rs:NestWhileEvalState",
    "crates/rum-stdlib/src/functional_ops/mod.rs:NestWhileTestArity",
    "crates/rum-stdlib/src/functional_ops/sequence/fold/mod.rs:FoldWhileConfig",
    "crates/rum-stdlib/src/functional_ops/sequence/fold/pair.rs:FoldPairEvalState",
    "crates/rum-stdlib/src/functional_ops/sequence/fold/plain.rs:FoldEvalState",
    "crates/rum-stdlib/src/functional_ops/sequence/fold/sequence_fold.rs:SequenceFoldEvalState",
    "crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs:FoldWhileEvalState",
    "crates/rum-stdlib/src/functional_ops/sequence/select.rs:SelectSource",
    "crates/rum-stdlib/src/functional_ops/sequence/window.rs:BlockMapEvalState",
    "crates/rum-stdlib/src/functional_ops/sequence/window.rs:SequenceLikeExpr",
    "crates/rum-stdlib/src/functional_ops/targeted_map.rs:MapAtEvalState",
    "crates/rum-stdlib/src/functional_ops/targeted_map.rs:SubsetMapEvalState",
    "crates/rum-stdlib/src/predicates/arrays.rs:ArrayLeafTestState",
    "crates/rum-stdlib/src/predicates/collections.rs:DuplicateFreeSameTestState",
    "crates/rum-stdlib/src/predicates/collections.rs:SameTestMatrixState",
    "crates/rum-stdlib/src/predicates/collections.rs:SameTestRelationKind",
    "crates/rum-stdlib/src/predicates/ordering.rs:LexicographicListComparatorState",
    "crates/rum-stdlib/src/predicates/ordering.rs:LexicographicPairComparatorState",
    "crates/rum-stdlib/src/predicates/ordering.rs:OrderedQComparatorState",
    "crates/rum-stdlib/src/strings.rs:StringApplyEvalState",
    "crates/rum-stdlib/src/strings/patterns.rs:ExactStringCaseOutput",
    "crates/rum-stdlib/src/strings/patterns.rs:ExactStringCaseRule",
    "crates/rum-stdlib/src/strings/patterns.rs:ExactStringCaseSpec",
    "crates/rum-stdlib/src/strings/patterns.rs:ExactStringReplacementRule",
    "crates/rum-stdlib/src/strings/patterns.rs:ExactStringReplacementValue",
    "crates/rum-stdlib/src/strings/patterns.rs:GuardedExactStringCaseRule",
    "crates/rum-stdlib/src/strings/patterns.rs:GuardedExactStringReplacementRule",
    "crates/rum-stdlib/src/strings/patterns.rs:StringCasesConfig",
    "crates/rum-stdlib/src/strings/patterns.rs:StringPattern",
    "crates/rum-stdlib/src/strings/patterns.rs:StringPatternMatchConfig",
    "crates/rum-stdlib/src/strings/patterns.rs:StringRuleGuard",
    "crates/rum-stdlib/src/strings/patterns.rs:StringSearchOverlaps",
    "crates/rum-stdlib/src/strings/patterns/regex.rs:RegexAtom",
    "crates/rum-stdlib/src/strings/patterns/regex.rs:RegexClassItem",
    "crates/rum-stdlib/src/strings/patterns/regex.rs:RegexPattern",
    "crates/rum-stdlib/src/strings/patterns/regex.rs:RegexQuantifier",
    "crates/rum-stdlib/src/strings/patterns/regex.rs:RegexToken",
    "crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:GuardedStringCasesRuleEvalState",
    "crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:GuardedStringCasesValuesEvalState",
    "crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:GuardedStringReplaceRuleEvalState",
    "crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:GuardedStringReplaceValuesEvalState",
    "crates/rum-stdlib/src/strings/patterns/search/rules.rs:StringCasesRuleEvalState",
    "crates/rum-stdlib/src/strings/patterns/search/rules.rs:StringCasesValuesEvalState",
    "crates/rum-stdlib/src/strings/patterns/search/rules.rs:StringReplaceRuleEvalState",
    "crates/rum-stdlib/src/strings/patterns/search/rules.rs:StringReplaceValuesEvalState",
    "crates/rum-stdlib/src/structure_ops/collection_stats.rs:DecorateByState",
    "crates/rum-stdlib/src/structure_ops/collection_stats.rs:DecoratedOperation",
    "crates/rum-stdlib/src/structure_ops/collection_stats.rs:GatherByFunctionListState",
    "crates/rum-stdlib/src/structure_ops/collection_stats.rs:KeyedEntry",
    "crates/rum-stdlib/src/structure_ops/collection_stats.rs:TallyTestState",
    "crates/rum-stdlib/src/structure_ops/dedup.rs:AdaptiveExprSet",
    "crates/rum-stdlib/src/structure_ops/dedup.rs:DeleteDuplicatesTestState",
    "crates/rum-stdlib/src/structure_ops/list_lookup.rs:SelectPositionsLimit",
    "crates/rum-stdlib/src/structure_ops/list_lookup.rs:SelectPositionsState",
    "crates/rum-stdlib/src/structure_ops/ordering_sets.rs:SetOperationKind",
    "crates/rum-stdlib/src/structure_ops/ordering_sets.rs:SetSameTestPhase",
    "crates/rum-stdlib/src/structure_ops/ordering_sets.rs:SetSameTestState",
    "crates/rum-stdlib/src/structure_ops/ordering_sets.rs:SortByEntry",
    "crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:ComparatorResultPolicy",
    "crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:MergeSortState",
    "crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:Run",
    "crates/rum-stdlib/src/structure_ops/selectors.rs:AtomicPartSpec",
    "crates/rum-stdlib/src/structure_ops/selectors.rs:PartSelector",
    "crates/rum-stdlib/src/structure_ops/selectors.rs:PartSpanSpec",
    "crates/rum-stdlib/src/structure_ops/selectors.rs:SpanEndpoint",
];

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
