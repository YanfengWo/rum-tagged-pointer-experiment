use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn cheap_combinatorics_builtins_match_wolfram_kernel_integer_anchors() {
    let mut session = Session::new();

    let cases = [
        ("Binomial[5, 2]", "10"),
        ("Binomial[4, 6]", "0"),
        ("Binomial[5, 0]", "1"),
        ("Binomial[-1, 2]", "1"),
        ("Binomial[-3, 2]", "6"),
        ("Binomial[3, -1]", "0"),
        ("Binomial[-3, -1]", "0"),
        ("Fibonacci[10]", "55"),
        ("Fibonacci[0]", "0"),
        ("Fibonacci[1]", "1"),
        ("Fibonacci[-6]", "-8"),
        ("Fibonacci[100]", "354224848179261915075"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn cheap_combinatorics_builtins_thread_and_passthrough_like_wolfram_kernel() {
    let mut session = Session::new();

    let cases = [
        ("Binomial[{5, 6}, 2]", "List[10, 15]"),
        ("Binomial[5, {0, 2, 6}]", "List[1, 10, 0]"),
        ("Fibonacci[{0, 1, 10, -6}]", "List[0, 1, 55, -8]"),
        ("Binomial[n, 2]", "Binomial[n, 2]"),
        ("Fibonacci[n]", "Fibonacci[n]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn cheap_combinatorics_builtins_report_wolfram_kernel_attributes() {
    let mut session = Session::new();

    let cases = [
        (
            "Attributes[Binomial]",
            "List[Listable, NumericFunction, Protected]",
        ),
        (
            "Attributes[Fibonacci]",
            "List[Listable, NumericFunction, Protected, ReadProtected]",
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn binomial_respects_integer_work_limit() {
    let mut session = Session::new();
    session.set_integer_work_limit(3);

    let limited = session.eval_text("Binomial[-1, 5]").unwrap();
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

    assert_eq!(eval_to_string(&mut session, "Binomial[4, 6]"), "0");
}
