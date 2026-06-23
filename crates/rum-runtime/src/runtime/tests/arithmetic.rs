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
    assert_eq!(session.eval_text("Abs[-7]").unwrap().value.to_string(), "7");
    assert_eq!(
        session.eval_text("Abs[-3/2]").unwrap().value.to_string(),
        "3/2"
    );
    assert_eq!(
        session.eval_text("Factorial[0]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Factorial[8]").unwrap().value.to_string(),
        "40320"
    );
    assert_eq!(
        session
            .eval_text("Factorial[-1]")
            .unwrap()
            .value
            .to_string(),
        "Factorial[-1]"
    );
    assert_eq!(
        session
            .eval_text("Factorial2[0]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Factorial2[6]")
            .unwrap()
            .value
            .to_string(),
        "48"
    );
    assert_eq!(
        session
            .eval_text("Factorial2[7]")
            .unwrap()
            .value
            .to_string(),
        "105"
    );
    assert_eq!(
        session
            .eval_text("Factorial2[-1]")
            .unwrap()
            .value
            .to_string(),
        "Factorial2[-1]"
    );
    assert_eq!(
        session
            .eval_text("Factorial2[3/2]")
            .unwrap()
            .value
            .to_string(),
        "Factorial2[3/2]"
    );
}

#[test]
fn round_handles_exact_numbers_decimal_literals_and_half_even_ties() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Round[3/2]").unwrap().value.to_string(),
        "2"
    );
    assert_eq!(
        session.eval_text("Round[5/2]").unwrap().value.to_string(),
        "2"
    );
    assert_eq!(
        session.eval_text("Round[7/2]").unwrap().value.to_string(),
        "4"
    );
    assert_eq!(
        session.eval_text("Round[-5/2]").unwrap().value.to_string(),
        "-2"
    );
    assert_eq!(
        session
            .eval_text("Round[7/3, 1/2]")
            .unwrap()
            .value
            .to_string(),
        "5/2"
    );
    assert_eq!(
        session
            .eval_text("Round[0.15, 0.1]")
            .unwrap()
            .value
            .to_string(),
        "1/5"
    );
}

#[test]
fn factorial_respects_integer_work_limit() {
    let mut session = Session::new();
    session.set_integer_work_limit(3);

    let limited = session.eval_text("Factorial[5]").unwrap();
    assert_eq!(
        limited.value.to_string(),
        "TerminatedEvaluation[\"ResourceLimit\"]"
    );
    assert!(
        limited
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );

    let limited = session.eval_text("Factorial2[5]").unwrap();
    assert_eq!(
        limited.value.to_string(),
        "TerminatedEvaluation[\"ResourceLimit\"]"
    );
    assert!(
        limited
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
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
fn numerator_mirrors_denominator_for_exact_numbers_and_registration() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Numerator /@ {1/2, 3/4, -3/4, 5, -5, 0}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 3, -3, 5, -5, 0]"
    );
    assert_eq!(
        session
            .eval_text("Context[Numerator]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Names[\"System`Numerator\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"System`Numerator\"]"
    );
    assert_eq!(
        session
            .eval_text("FullForm[Numerator[x]]")
            .unwrap()
            .value
            .to_string(),
        "\"Numerator[x]\""
    );
    assert_eq!(
        session
            .eval_text("Denominator /@ {3/4, 5}")
            .unwrap()
            .value
            .to_string(),
        "List[4, 1]"
    );
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
fn max_min_handle_exact_extrema_lists_and_registration() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Max[3, 5]").unwrap().value.to_string(),
        "5"
    );
    assert_eq!(
        session.eval_text("Min[3, 5]").unwrap().value.to_string(),
        "3"
    );
    assert_eq!(session.eval_text("Max[3]").unwrap().value.to_string(), "3");
    assert_eq!(
        session.eval_text("Max[1, 5, 2]").unwrap().value.to_string(),
        "5"
    );
    assert_eq!(
        session.eval_text("Max[3/2, 1]").unwrap().value.to_string(),
        "3/2"
    );
    assert_eq!(
        session.eval_text("Min[3/2, 1]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Max[-3, -5]").unwrap().value.to_string(),
        "-3"
    );
    assert_eq!(
        session
            .eval_text("Max[{3, {5}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
    assert_eq!(
        session
            .eval_text("Min[{4, 1}, 2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Context[Max]").unwrap().value.to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Names[\"System`Min\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"System`Min\"]"
    );
    assert_eq!(
        session
            .eval_text("FullForm[Max[x, 3]]")
            .unwrap()
            .value
            .to_string(),
        "\"Max[x, 3]\""
    );
    assert_eq!(
        session.eval_text("Min[]").unwrap().value.to_string(),
        "Infinity"
    );
    assert_eq!(
        session.eval_text("Max[]").unwrap().value.to_string(),
        "Times[-1, Infinity]"
    );
    assert_eq!(
        session.eval_text("Min[{}]").unwrap().value.to_string(),
        "Infinity"
    );
    assert_eq!(
        session.eval_text("Max[{}]").unwrap().value.to_string(),
        "Times[-1, Infinity]"
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
