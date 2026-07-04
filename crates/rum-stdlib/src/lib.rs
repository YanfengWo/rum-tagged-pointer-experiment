use rum_runtime::{Session, core_builtin_specs};

mod associations;
mod calculus;
mod catalog;
mod combinatorics;
mod functional_ops;
mod integer_digits;
mod logic;
mod number_theory;
mod polynomials;
mod predicates;
mod strings;
mod structure_ops;

pub fn install_core(session: &mut Session) {
    for spec in core_builtin_specs() {
        session.register_builtin(*spec);
    }
    for spec in catalog::LOGIC_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::PREDICATE_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::ASSOCIATION_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::COMBINATORICS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::FUNCTIONAL_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::INTEGER_DIGITS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::NUMBER_THEORY_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::CALCULUS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::POLYNOMIAL_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::STRING_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::STRUCTURE_BUILTINS {
        session.register_builtin(*spec);
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
