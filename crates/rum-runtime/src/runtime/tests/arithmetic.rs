use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

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
fn scalar_math_builtins_match_exact_wolfram_kernel_cases() {
    let mut session = Session::new();

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
    let mut session = Session::new();

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
fn scalar_math_builtins_report_wolfram_kernel_attributes() {
    let mut session = Session::new();

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
    let mut session = Session::new();
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
fn power_times_combine_like_bases_and_terms() {
    let mut session = Session::new();

    let sameq_cases = [
        "x x === x^2",
        "x^2 x^3 === x^5",
        "x^2 x === x^3",
        "a b / b === a",
        "x x x / x === x^2",
        "2 a 3 a === 6 a^2",
        "x/x === 1",
        "x^2 y x === x^3 y",
        "(x^2)^3 === x^6",
        "(x^2)^-1 === x^-2",
        "x^3/x^2 === x",
        "x^2/x^3 === x^-1",
        "x + x === 2 x",
        "2 x + 3 x === 5 x",
        "x - x === 0",
        "3 x - x === 2 x",
        "x + 2 x - 3 x === 0",
        "a + b + a === 2 a + b",
        "x + x^2 + x === 2 x + x^2",
    ];

    for source in sameq_cases {
        assert_eq!(eval_to_string(&mut session, source), "True", "{source}");
    }

    let scalar_cases = [("x - x", "0"), ("x/x", "1"), ("x + 2 x - 3 x", "0")];

    for (source, expected) in scalar_cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }

    assert_eq!(
        eval_to_string(&mut session, "1 (x^2)^(1/2)"),
        "Power[Power[x, 2], 1/2]"
    );
}

#[test]
fn power_over_times_distributes_numeric_exponents() {
    let mut session = Session::new();

    let sameq_cases = [
        "(2 x)^2 === 4 x^2",
        "(2 x)^3 === 8 x^3",
        "(-2 x)^3 === -8 x^3",
        "(3/2 x)^2 === 9/4 x^2",
        "(6 x)^2 === 36 x^2",
        "(x/2)^2 === x^2/4",
        "(2 x)^-2 === 1/(4 x^2)",
        "(2 x y)^2 === 4 x^2 y^2",
        "(a b)^2 === a^2 b^2",
        "(a/b)^2 === a^2/b^2",
        "(2 x)^(1/2) === 2^(1/2) x^(1/2)",
        "(2 x y)^(1/2) === 2^(1/2) (x y)^(1/2)",
        "(a b)^(1/2) === (a b)^(1/2)",
    ];

    for source in sameq_cases {
        assert_eq!(eval_to_string(&mut session, source), "True", "{source}");
    }

    let guard_cases = [
        ("(a b)^(1/2)", "Power[Times[a, b], 1/2]"),
        ("(2 x)^n", "Power[Times[2, x], n]"),
        (
            "(4 x^2)^(1/2)",
            "Times[Power[4, 1/2], Power[Power[x, 2], 1/2]]",
        ),
    ];

    for (source, expected) in guard_cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
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
