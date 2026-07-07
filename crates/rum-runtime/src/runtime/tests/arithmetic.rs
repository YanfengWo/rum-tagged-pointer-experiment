use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn exact_integer_arithmetic_evaluates_core_heads() {
    let mut session = session_with_test_rewrite_builtins();

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
    let mut session = session_with_test_rewrite_builtins();

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
fn scalar_math_builtins_exact_cases() {
    let mut session = session_with_test_rewrite_builtins();

    let cases = [
        ("Sign[-3/2]", "-1"),
        ("Sign[0]", "0"),
        ("Sign[3]", "1"),
        ("Sign[x]", "Sign[x]"),
        ("Floor[7/2]", "3"),
        ("Floor[-7/2]", "-4"),
        ("Floor[3]", "3"),
        ("Floor[x]", "Floor[x]"),
        ("Ceiling[7/2]", "4"),
        ("Ceiling[-7/2]", "-3"),
        ("Ceiling[3]", "3"),
        ("Ceiling[x]", "Ceiling[x]"),
        ("IntegerPart[7/2]", "3"),
        ("IntegerPart[-7/2]", "-3"),
        ("IntegerPart[0]", "0"),
        ("IntegerPart[x]", "IntegerPart[x]"),
        ("FractionalPart[7/2]", "1/2"),
        ("FractionalPart[-7/2]", "-1/2"),
        ("FractionalPart[0]", "0"),
        ("FractionalPart[x]", "FractionalPart[x]"),
        ("Minus[3]", "-3"),
        ("Minus[-3/2]", "3/2"),
        ("Minus[0]", "0"),
        ("Minus[x]", "Times[-1, x]"),
        ("MinMax[{3, 1, 2}]", "List[1, 3]"),
        ("MinMax[{3/2, 1, -2}]", "List[-2, 3/2]"),
        ("MinMax[{}]", "List[Infinity, Times[-1, Infinity]]"),
        ("MinMax[{3}]", "List[3, 3]"),
        ("MinMax[x]", "List[x, x]"),
        ("MinMax[{x, 3}]", "List[Min[3, x], Max[3, x]]"),
        ("MinMax[{3, {x}}]", "List[Min[3, x], Max[3, x]]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn scalar_math_builtins_thread_over_lists_and_leave_skipped_forms_symbolic() {
    let mut session = session_with_test_rewrite_builtins();

    let cases = [
        ("Sign[{7/2, -1/2, x}]", "List[1, -1, Sign[x]]"),
        ("Floor[{7/2, -1/2, x}]", "List[3, -1, Floor[x]]"),
        ("Ceiling[{7/2, -1/2, x}]", "List[4, 0, Ceiling[x]]"),
        ("IntegerPart[{7/2, -1/2, x}]", "List[3, 0, IntegerPart[x]]"),
        (
            "FractionalPart[{7/2, -1/2, x}]",
            "List[1/2, -1/2, FractionalPart[x]]",
        ),
        ("Minus[{3, -1/2, x}]", "List[-3, 1/2, Times[-1, x]]"),
        ("Floor[7/2, 2]", "Floor[7/2, 2]"),
        ("MinMax[{3, 1, 2}, 2]", "MinMax[List[3, 1, 2], 2]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn scalar_math_builtins_report_declared_attributes() {
    let mut session = session_with_test_rewrite_builtins();

    let scalar_attributes = "List[Listable, NumericFunction, Protected]";
    for name in [
        "Sign",
        "Floor",
        "Ceiling",
        "IntegerPart",
        "FractionalPart",
        "Minus",
    ] {
        let source = format!("Attributes[{name}]");
        assert_eq!(
            eval_to_string(&mut session, &source),
            scalar_attributes,
            "{source}"
        );
    }

    assert_eq!(
        eval_to_string(&mut session, "Attributes[MinMax]"),
        "List[Protected]"
    );
}

#[test]
fn factorial_respects_integer_work_limit() {
    let mut session = session_with_test_rewrite_builtins();
    session.set_integer_work_limit(3);

    let limited = session.eval_text("Factorial[5]").unwrap();
    assert_eq!(
        limited.value.to_string(),
        "TerminatedEvaluation[\"IntegerWorkLimit\"]"
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
        "TerminatedEvaluation[\"IntegerWorkLimit\"]"
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
    let mut session = session_with_test_rewrite_builtins();

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
    let mut session = session_with_test_rewrite_builtins();

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
fn total_evaluates_documented_level_forms_and_leaves_invalid_specs_inert() {
    let mut session = session_with_test_rewrite_builtins();

    let cases = [
        ("Total[{{1, 2}, {3, 4}}]", "List[4, 6]"),
        ("Total[{1, 2, 3}]", "6"),
        ("Total[{{1, 2}, {3, 4}}, 1]", "List[4, 6]"),
        ("Total[{{1, 2}, {3, 4}}, 2]", "10"),
        ("Total[{1, 2, 3}, 2]", "6"),
        ("Total[{{1, 2}, {3, 4}}, {1}]", "List[4, 6]"),
        ("Total[{{1, 2}, {3, 4}}, {2}]", "List[3, 7]"),
        ("Total[{{1, 2}, {3, 4}}, Infinity]", "10"),
        (
            "Total[{{1, 2}, {3}}, {2}]",
            "Total[List[List[1, 2], List[3]], List[2]]",
        ),
        (
            "Total[{{1, 2}, {3, 4}}, 0]",
            "Total[List[List[1, 2], List[3, 4]], 0]",
        ),
        (
            "Total[{{1, 2}, {3, 4}}, x]",
            "Total[List[List[1, 2], List[3, 4]], x]",
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn exact_integer_mod_quotient_gcd_lcm_and_powermod_follow_floor_style_semantics() {
    let mut session = session_with_test_rewrite_builtins();

    assert_eq!(
        session.eval_text("Mod[5, 0]").unwrap().value.to_string(),
        "5"
    );
    assert_eq!(
        session.eval_text("Mod[-3, 0]").unwrap().value.to_string(),
        "-3"
    );
    assert_eq!(
        session.eval_text("Mod[7, 3]").unwrap().value.to_string(),
        "1"
    );
    assert_eq!(
        session.eval_text("Mod[x, 0]").unwrap().value.to_string(),
        "Mod[x, 0]"
    );
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
fn powermod_negative_exponents_use_modular_inverse() {
    let mut session = session_with_test_rewrite_builtins();

    for (source, expected) in [
        ("PowerMod[2, -1, 5]", "3"),
        ("PowerMod[3, -1, 7]", "5"),
        ("PowerMod[2, -3, 5]", "2"),
        ("PowerMod[10, -1, 1]", "0"),
        ("PowerMod[2, 3, 5]", "3"),
        ("PowerMod[2, 3, 0]", "PowerMod[2, 3, 0]"),
        ("PowerMod[2, -1, 0]", "PowerMod[2, -1, 0]"),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert_eq!(outcome.value.to_string(), expected, "{source}");
        assert!(outcome.messages.is_empty(), "{source}");
    }

    let noninvertible = session.eval_text("PowerMod[2, -1, 4]").unwrap();
    assert_eq!(noninvertible.value.to_string(), "PowerMod[2, -1, 4]");
    assert!(
        noninvertible
            .messages
            .iter()
            .any(|message| message.symbol == "PowerMod" && message.tag == "ninv")
    );
}

#[test]
fn rational_atoms_expose_a_rational_head_for_patterns() {
    let mut session = session_with_test_rewrite_builtins();

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
