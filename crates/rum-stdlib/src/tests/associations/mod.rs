use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

mod association_part_cursor;
mod associations;
mod associations_aggregate_diagnostics;
mod associations_basic_diagnostics;
mod associations_key_arity_diagnostics;
mod associations_key_diagnostics;
mod associations_key_reshape;
mod associations_key_sets;
mod associations_key_sort;
mod associations_key_value_pattern;
mod associations_missing_diagnostics;
mod associations_mutation;
mod associations_mutation_definition_fallback;
mod associations_mutation_diagnostics;
mod associations_mutation_operator_forms;
mod associations_mutation_surface;
mod associations_part2;
mod associations_set_lvalue;
mod join_across;
