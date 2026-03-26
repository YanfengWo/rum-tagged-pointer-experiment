use super::*;

#[test]
fn exact_integer_arithmetic_evaluates_core_heads() {
    let mut session = Session::new();

    assert_eq!(session.eval_text("2 + 3").unwrap().value.to_string(), "5");
    assert_eq!(session.eval_text("2 - 5").unwrap().value.to_string(), "-3");
    assert_eq!(
        session.eval_text("2 * 3 * 4").unwrap().value.to_string(),
        "24"
    );
    assert_eq!(
        session.eval_text("Power[2, 10]").unwrap().value.to_string(),
        "1024"
    );
}

#[test]
fn exact_division_and_negative_integer_powers_produce_rationals() {
    let mut session = Session::new();

    assert_eq!(session.eval_text("1 / 2").unwrap().value.to_string(), "1/2");
    assert_eq!(session.eval_text("6 / 4").unwrap().value.to_string(), "3/2");
    assert_eq!(
        session.eval_text("Divide[9, 6]").unwrap().value.to_string(),
        "3/2"
    );
    assert_eq!(session.eval_text("2^-3").unwrap().value.to_string(), "1/8");
    assert_eq!(session.eval_text("(-2)^3").unwrap().value.to_string(), "-8");
}

#[test]
fn plus_and_times_fold_numeric_parts_without_forcing_symbolic_algebra() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("x + 2 + 1/2").unwrap().value.to_string(),
        "Plus[5/2, x]"
    );
    assert_eq!(
        session
            .eval_text("2 * x * (3/4)")
            .unwrap()
            .value
            .to_string(),
        "Times[3/2, x]"
    );
    assert_eq!(session.eval_text("Plus[]").unwrap().value.to_string(), "0");
    assert_eq!(session.eval_text("Times[]").unwrap().value.to_string(), "1");
    assert_eq!(session.eval_text("Plus[x]").unwrap().value.to_string(), "x");
    assert_eq!(
        session.eval_text("Times[x]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(session.eval_text("x + 0").unwrap().value.to_string(), "x");
    assert_eq!(session.eval_text("x * 1").unwrap().value.to_string(), "x");
    assert_eq!(session.eval_text("0 * x").unwrap().value.to_string(), "0");
}

#[test]
fn exact_integer_mod_quotient_gcd_lcm_and_powermod_follow_floor_style_semantics() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Mod[-3, 2]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Mod[3, -2]").unwrap().value.to_string(),
        "-1"
    );
    assert_eq!(
        session
            .eval_text("Quotient[-3, 2]")
            .unwrap()
            .value
            .to_string(),
        "-2"
    );
    assert_eq!(
        session
            .eval_text("Quotient[3, -2]")
            .unwrap()
            .value
            .to_string(),
        "-2"
    );
    assert_eq!(
        session
            .eval_text("GCD[18, -24, 30]")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
    assert_eq!(
        session
            .eval_text("LCM[6, -8, 15]")
            .unwrap()
            .value
            .to_string(),
        "120"
    );
    assert_eq!(
        session
            .eval_text("PowerMod[3, 5, 7]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
}

#[test]
fn rational_atoms_expose_a_rational_head_for_patterns() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("MatchQ[1/2, Rational[_Integer, _Integer]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MatchQ[1/2, _Rational]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}
