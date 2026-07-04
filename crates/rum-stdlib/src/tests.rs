use super::*;

#[path = "tests/associations/mod.rs"]
mod associations;
#[path = "tests/associations_missing_surface.rs"]
mod associations_missing_surface;
#[path = "tests/calculus.rs"]
mod calculus;
#[path = "tests/combinatorics.rs"]
mod combinatorics;
#[path = "tests/debug_tools_evaluation.rs"]
mod debug_tools_evaluation;
#[path = "tests/debug_tools_reports.rs"]
mod debug_tools_reports;
#[path = "tests/debug_tools_resources.rs"]
mod debug_tools_resources;
#[path = "tests/functional_ops/mod.rs"]
mod functional_ops;
#[path = "tests/holding_and_control.rs"]
mod holding_and_control;
#[path = "tests/holding_and_control_abort.rs"]
mod holding_and_control_abort;
#[path = "tests/holding_and_control_cleanup.rs"]
mod holding_and_control_cleanup;
#[path = "tests/holding_and_control_failure_edges.rs"]
mod holding_and_control_failure_edges;
#[path = "tests/holding_and_control_failure_properties.rs"]
mod holding_and_control_failure_properties;
#[path = "tests/holding_and_control_part3.rs"]
mod holding_and_control_part3;
#[path = "tests/holding_and_control_part4.rs"]
mod holding_and_control_part4;
#[path = "tests/holding_and_control_part5.rs"]
mod holding_and_control_part5;
#[path = "tests/holding_and_control_part6.rs"]
mod holding_and_control_part6;
#[path = "tests/holding_and_control_part8.rs"]
mod holding_and_control_part8;
#[path = "tests/holding_and_control_success.rs"]
mod holding_and_control_success;
#[path = "tests/incidental_arithmetic.rs"]
mod incidental_arithmetic;
#[path = "tests/incidental_associations_part2.rs"]
mod incidental_associations_part2;
#[path = "tests/incidental_context_and_loader.rs"]
mod incidental_context_and_loader;
#[path = "tests/incidental_debug_tools_dialogs.rs"]
mod incidental_debug_tools_dialogs;
#[path = "tests/incidental_debug_tools_resources.rs"]
mod incidental_debug_tools_resources;
#[path = "tests/incidental_definitions_and_rules_cases.rs"]
mod incidental_definitions_and_rules_cases;
#[path = "tests/incidental_definitions_and_rules_lifecycle.rs"]
mod incidental_definitions_and_rules_lifecycle;
#[path = "tests/incidental_definitions_and_rules_part2.rs"]
mod incidental_definitions_and_rules_part2;
#[path = "tests/incidental_definitions_and_rules_patterns.rs"]
mod incidental_definitions_and_rules_patterns;
#[path = "tests/incidental_definitions_updates.rs"]
mod incidental_definitions_updates;
#[path = "tests/incidental_holding_and_control_part2.rs"]
mod incidental_holding_and_control_part2;
#[path = "tests/incidental_holding_and_control_part3.rs"]
mod incidental_holding_and_control_part3;
#[path = "tests/incidental_holding_and_control_part5.rs"]
mod incidental_holding_and_control_part5;
#[path = "tests/incidental_holding_and_control_part6.rs"]
mod incidental_holding_and_control_part6;
#[path = "tests/incidental_holding_and_control_part8.rs"]
mod incidental_holding_and_control_part8;
#[path = "tests/incidental_lowering_and_context.rs"]
mod incidental_lowering_and_context;
#[path = "tests/incidental_nothing.rs"]
mod incidental_nothing;
#[path = "tests/incidental_options_and_protection.rs"]
mod incidental_options_and_protection;
#[path = "tests/incidental_package_tools.rs"]
mod incidental_package_tools;
#[path = "tests/incidental_predicates.rs"]
mod incidental_predicates;
#[path = "tests/incidental_print_sink.rs"]
mod incidental_print_sink;
#[path = "tests/incidental_scoping_unique.rs"]
mod incidental_scoping_unique;
#[path = "tests/incidental_session_requests_debug_tooling_resume.rs"]
mod incidental_session_requests_debug_tooling_resume;
#[path = "tests/incidental_session_requests_functional_sequence_resume.rs"]
mod incidental_session_requests_functional_sequence_resume;
#[path = "tests/incidental_session_requests_input_resume.rs"]
mod incidental_session_requests_input_resume;
#[path = "tests/incidental_session_requests_matcher_resume.rs"]
mod incidental_session_requests_matcher_resume;
#[path = "tests/incidental_session_requests_transforms_resume.rs"]
mod incidental_session_requests_transforms_resume;
#[path = "tests/incidental_session_requests_transforms_resume_associations.rs"]
mod incidental_session_requests_transforms_resume_associations;
#[path = "tests/incidental_structure_ops_ordering_sets.rs"]
mod incidental_structure_ops_ordering_sets;
#[path = "tests/incidental_structure_ops_replace.rs"]
mod incidental_structure_ops_replace;
#[path = "tests/incidental_structure_ops_selectors_part2.rs"]
mod incidental_structure_ops_selectors_part2;
#[path = "tests/incidental_structure_ops_surface.rs"]
mod incidental_structure_ops_surface;
#[path = "tests/incidental_thread_and_dispatch.rs"]
mod incidental_thread_and_dispatch;
#[path = "tests/integer_digits.rs"]
mod integer_digits;
#[path = "tests/logic/mod.rs"]
mod logic;
#[path = "tests/mma_hol_cold_lane.rs"]
mod mma_hol_cold_lane;
#[path = "tests/number_theory.rs"]
mod number_theory;
#[path = "tests/polynomials.rs"]
mod polynomials;
#[path = "tests/predicates.rs"]
mod predicates;
#[path = "tests/predicates_part2.rs"]
mod predicates_part2;
#[path = "tests/session_requests/functional_iteration_resume.rs"]
mod session_requests_functional_iteration_resume;
#[path = "tests/session_requests/functional_sequence_resume.rs"]
mod session_requests_functional_sequence_resume;
#[path = "tests/session_requests/functional_targeted_resume.rs"]
mod session_requests_functional_targeted_resume;
#[path = "tests/session_requests/functional_tensor_string_resume.rs"]
mod session_requests_functional_tensor_string_resume;
#[path = "tests/session_requests/transforms_resume_associations.rs"]
mod session_requests_transforms_resume_associations;
#[path = "tests/strings/mod.rs"]
mod strings;
#[path = "tests/structure_ops/mod.rs"]
mod structure_ops;
#[path = "tests/traversal.rs"]
mod traversal;

#[test]
fn installs_identity_builtin() {
    let mut session = Session::new();
    install_core(&mut session);

    let info = session.inspect("Identity").unwrap();
    assert!(info.is_builtin);
}

#[test]
fn catalog_attributes_match_kernel_vocabulary() {
    use std::collections::BTreeMap;

    use rum_runtime::kernel_api::stdlib_builtin_attribute_specs;

    let mut catalog_by_name = BTreeMap::new();
    for spec in stdlib_catalog_specs() {
        let previous = catalog_by_name.insert(spec.name, spec.attributes);
        assert!(
            previous.is_none(),
            "duplicate stdlib catalog spec: {}",
            spec.name
        );
    }

    let mut kernel_by_name = BTreeMap::new();
    for spec in stdlib_builtin_attribute_specs() {
        let previous = kernel_by_name.insert(spec.name, spec.attributes);
        assert!(
            previous.is_none(),
            "duplicate kernel attribute vocabulary spec: {}",
            spec.name
        );
    }

    for (name, attributes) in catalog_by_name.iter() {
        assert_eq!(
            kernel_by_name.get(name).copied(),
            Some(*attributes),
            "kernel attribute vocabulary drift for {name}"
        );
    }

    for name in kernel_by_name.keys() {
        assert!(
            catalog_by_name.contains_key(name),
            "kernel attribute vocabulary has non-stdlib symbol: {name}"
        );
    }
}

fn stdlib_catalog_specs() -> impl Iterator<Item = &'static rum_runtime::kernel_api::BuiltinSpec> {
    [
        catalog::ARITHMETIC_BUILTINS,
        catalog::LOGIC_BUILTINS,
        catalog::PREDICATE_BUILTINS,
        catalog::ASSOCIATION_BUILTINS,
        catalog::COMBINATORICS_BUILTINS,
        catalog::CONTROL_BUILTINS,
        catalog::FUNCTIONAL_BUILTINS,
        catalog::INTEGER_DIGITS_BUILTINS,
        catalog::NUMBER_THEORY_BUILTINS,
        catalog::CALCULUS_BUILTINS,
        catalog::POLYNOMIAL_BUILTINS,
        catalog::REWRITE_BUILTINS,
        catalog::STRING_BUILTINS,
        catalog::STRUCTURE_BUILTINS,
    ]
    .into_iter()
    .flatten()
}
