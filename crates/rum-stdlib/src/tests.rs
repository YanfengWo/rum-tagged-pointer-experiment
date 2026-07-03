use super::*;

#[path = "tests/associations_missing_surface.rs"]
mod associations_missing_surface;
#[path = "tests/calculus.rs"]
mod calculus;
#[path = "tests/combinatorics.rs"]
mod combinatorics;
#[path = "tests/debug_tools_reports.rs"]
mod debug_tools_reports;
#[path = "tests/functional_ops/mod.rs"]
mod functional_ops;
#[path = "tests/holding_and_control_abort.rs"]
mod holding_and_control_abort;
#[path = "tests/holding_and_control_part4.rs"]
mod holding_and_control_part4;
#[path = "tests/incidental_debug_tools_resources.rs"]
mod incidental_debug_tools_resources;
#[path = "tests/incidental_definitions_and_rules_patterns.rs"]
mod incidental_definitions_and_rules_patterns;
#[path = "tests/incidental_holding_and_control_part2.rs"]
mod incidental_holding_and_control_part2;
#[path = "tests/incidental_holding_and_control_part5.rs"]
mod incidental_holding_and_control_part5;
#[path = "tests/incidental_predicates.rs"]
mod incidental_predicates;
#[path = "tests/incidental_scoping_gensym_gc.rs"]
mod incidental_scoping_gensym_gc;
#[path = "tests/incidental_structure_ops_ordering_sets.rs"]
mod incidental_structure_ops_ordering_sets;
#[path = "tests/integer_digits.rs"]
mod integer_digits;
#[path = "tests/logic/mod.rs"]
mod logic;
#[path = "tests/number_theory.rs"]
mod number_theory;
#[path = "tests/polynomials.rs"]
mod polynomials;
#[path = "tests/strings/mod.rs"]
mod strings;

#[test]
fn installs_identity_builtin() {
    let mut session = Session::new();
    install_core(&mut session);

    let info = session.inspect("Identity").unwrap();
    assert!(info.is_builtin);
}
