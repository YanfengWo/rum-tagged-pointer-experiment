use super::paths::text_files_in;
use super::*;

#[test]
fn parses_imported_golden_seed_pack() {
    let mut total = 0usize;
    let mut normal = 0usize;
    let mut known_diffs = 0usize;

    let files = text_files_in(golden_dir()).unwrap();

    assert_eq!(files.len(), 9);

    for file in files {
        let text = load_text(&file).unwrap();
        let entries = parse_golden_file(&text).unwrap();
        total += entries.len();
        for entry in entries {
            match entry {
                GoldenEntry::Normal { .. } => normal += 1,
                GoldenEntry::KnownDifference { .. } => known_diffs += 1,
            }
        }
    }

    assert_eq!(normal, 166);
    assert_eq!(known_diffs, 3);
    assert_eq!(total, 169);
}

#[test]
fn parses_imported_timing_and_memory_seed_suites() {
    let timing =
        parse_expression_suite(&load_text(seed_root().join("bench/timing_suite.txt")).unwrap());
    let memory =
        parse_expression_suite(&load_text(seed_root().join("mem/memory_suite.txt")).unwrap());

    assert_eq!(timing.len(), 20);
    assert_eq!(memory.len(), 13);
    assert_eq!(timing[0].category.as_deref(), Some("trivial"));
    assert_eq!(memory[0].category.as_deref(), Some("minimal"));
}

#[test]
fn parses_curated_lane_a_smoke_fixtures() {
    let files = text_files_in(curated_wolfram_core_dir()).unwrap();

    assert!(files.len() >= 4);

    for file in files {
        let text = load_text(&file).unwrap();
        let entries = parse_golden_file(&text).unwrap();
        assert!(
            !entries.is_empty(),
            "curated fixture should not be empty: {}",
            file.display()
        );
    }
}

#[test]
fn lane_a_default_fixture_paths_include_seed_and_curated_cases() {
    let files = lane_a_default_fixture_paths().unwrap();

    assert_eq!(files.len(), 38);
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("golden/arithmetic.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/smoke_arithmetic.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/options_surface.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/ordering_surface.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/collection_shape_predicates.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/calculus_symbolic.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/context_namespace.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/lists_surface.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/messages_attributes.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/scoping_closures.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/exact_arithmetic.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/polynomial_rational.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/predicate_surface.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/structure_selectors.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/threading.txt"))
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/wolfram/core/truth_comparison.txt"))
    );
}

#[test]
fn parses_curated_stateful_fixtures() {
    let files = lane_b_default_fixture_paths().unwrap();

    assert!(
        !files.is_empty(),
        "stateful fixture directory should contain at least one fixture"
    );
    assert!(
        files
            .iter()
            .any(|path| path.ends_with("cases/stateful/scoping_control.txt")),
        "Lane B default fixtures should include stateful scoping/control coverage"
    );

    for file in files {
        let text = load_text(&file).unwrap();
        let entries = parse_golden_file(&text).unwrap();
        assert!(
            !entries.is_empty(),
            "stateful fixture should not be empty: {}",
            file.display()
        );
    }
}
