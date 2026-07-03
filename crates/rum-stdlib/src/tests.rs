use super::*;

#[path = "tests/calculus.rs"]
mod calculus;
#[path = "tests/combinatorics.rs"]
mod combinatorics;
#[path = "tests/integer_digits.rs"]
mod integer_digits;
#[path = "tests/number_theory.rs"]
mod number_theory;
#[path = "tests/polynomials.rs"]
mod polynomials;

#[test]
fn installs_identity_builtin() {
    let mut session = Session::new();
    install_core(&mut session);

    let info = session.inspect("Identity").unwrap();
    assert!(info.is_builtin);
}
