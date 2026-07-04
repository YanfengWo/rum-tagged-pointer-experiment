use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn power_times_combine_like_bases_and_terms() {
    let mut session = core_session();

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
    let mut session = core_session();

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
fn numerator_mirrors_denominator_for_exact_numbers_and_registration() {
    let mut session = core_session();

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
fn max_min_handle_exact_extrema_lists_and_registration() {
    let mut session = core_session();

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
