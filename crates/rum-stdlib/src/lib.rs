use rum_runtime::{Session, core_builtin_specs};

mod catalog;
mod combinatorics;
mod integer_digits;
mod number_theory;

pub fn install_core(session: &mut Session) {
    for spec in core_builtin_specs() {
        session.register_builtin(*spec);
    }
    for spec in catalog::COMBINATORICS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::INTEGER_DIGITS_BUILTINS {
        session.register_builtin(*spec);
    }
    for spec in catalog::NUMBER_THEORY_BUILTINS {
        session.register_builtin(*spec);
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
