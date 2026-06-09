use super::super::{run_difftest, stderr_text, stdout_text};

#[test]
fn inventory_reports_default_fixture_coverage() {
    let output = run_difftest(&["inventory"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("fixture inventory"));
    assert!(stdout.contains("lane-a fixtures: 38"));
    assert!(stdout.contains("lane-a source seed-golden: fixtures=9"));
    assert!(stdout.contains("lane-a source curated-core: fixtures=29"));
    assert!(stdout.contains("lane-a category associations-core: fixtures=1"));
    assert!(stdout.contains("lane-a category calculus-symbolic: fixtures=1"));
    assert!(stdout.contains("lane-a category collection-shape-predicates: fixtures=1"));
    assert!(stdout.contains("lane-a category context-namespace: fixtures=1"));
    assert!(stdout.contains("lane-a category exact-arithmetic: fixtures=1"));
    assert!(stdout.contains("lane-a category lists-surface: fixtures=1"));
    assert!(stdout.contains("lane-a category messages-attributes: fixtures=1"));
    assert!(stdout.contains("lane-a category options-surface: fixtures=1"));
    assert!(stdout.contains("lane-a category ordering-surface: fixtures=1"));
    assert!(stdout.contains("lane-a category polynomial-rational: fixtures=1"));
    assert!(stdout.contains("lane-a category predicate-surface: fixtures=1"));
    assert!(stdout.contains("lane-a category scoping-closures: fixtures=1"));
    assert!(stdout.contains("lane-a category functional-combinators: fixtures=1"));
    assert!(stdout.contains("lane-a category structure-selectors: fixtures=1"));
    assert!(stdout.contains("lane-a category threading: fixtures=1"));
    assert!(stdout.contains("lane-a category truth-comparison: fixtures=1"));
    assert!(stdout.contains("lane-b source stateful: fixtures=7"));
    assert!(stdout.contains("lane-b category definitions-messages: fixtures=1"));
    assert!(stdout.contains("lane-b category message-switches: fixtures=1"));
    assert!(stdout.contains("lane-b category mutation-updates: fixtures=1"));
    assert!(stdout.contains("lane-b category options-state: fixtures=1"));
    assert!(stdout.contains("lane-b category scoping-control: fixtures=1"));
    assert!(stdout.contains("lane-b category zz-fixture-isolation: fixtures=1"));
}

#[test]
fn inventory_help_lists_baseline_gate() {
    let output = run_difftest(&["inventory", "--help"]);

    assert!(output.status.success(), "stderr: {}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("--baseline"));
    assert!(stdout.contains("--check-baseline"));
    assert!(stdout.contains("--fail-on-regression"));
    assert!(stdout.contains("--write-baseline"));
    assert!(stdout.contains("--lane"));
    assert!(stdout.contains("--source"));
    assert!(stdout.contains("--category"));
}
