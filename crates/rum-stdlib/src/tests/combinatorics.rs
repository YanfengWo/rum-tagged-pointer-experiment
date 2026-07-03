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
fn cheap_combinatorics_builtins_match_wolfram_kernel_integer_anchors() {
    let mut session = core_session();

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
        ("PartitionsP[5]", "7"),
        ("PartitionsP[10]", "42"),
        ("PartitionsP[0]", "1"),
        ("PartitionsP[100]", "190569292"),
        ("PartitionsQ[5]", "3"),
        ("PartitionsQ[10]", "10"),
        ("PartitionsQ[0]", "1"),
        (
            "Table[BernoulliB[i], {i, 0, 8}]",
            "List[1, -1/2, 1/6, 0, -1/30, 0, 1/42, 0, -1/30]",
        ),
        (
            "Table[Subfactorial[i], {i, 0, 6}]",
            "List[1, 0, 1, 2, 9, 44, 265]",
        ),
        ("StirlingS1[4, 2]", "11"),
        ("StirlingS1[5, 2]", "-50"),
        ("StirlingS2[4, 2]", "7"),
        ("StirlingS2[5, 3]", "25"),
        (
            "Table[CatalanNumber[i], {i, 0, 6}]",
            "List[1, 1, 2, 5, 14, 42, 132]",
        ),
        ("Multinomial[1, 2, 3]", "60"),
        ("Multinomial[2, 2]", "6"),
        ("Pochhammer[2, 3]", "24"),
        ("Pochhammer[3, 0]", "1"),
        ("Pochhammer[a, 3]", "Times[a, Plus[1, a], Plus[2, a]]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn cheap_combinatorics_builtins_thread_and_passthrough_like_wolfram_kernel() {
    let mut session = core_session();

    let cases = [
        ("Binomial[{5, 6}, 2]", "List[10, 15]"),
        ("Binomial[5, {0, 2, 6}]", "List[1, 10, 0]"),
        ("Fibonacci[{0, 1, 10, -6}]", "List[0, 1, 55, -8]"),
        ("PartitionsP[{0, 5, x}]", "List[1, 7, PartitionsP[x]]"),
        ("PartitionsQ[{0, 5, x}]", "List[1, 3, PartitionsQ[x]]"),
        (
            "BernoulliB[{0, 1, 2, x}]",
            "List[1, -1/2, 1/6, BernoulliB[x]]",
        ),
        ("Subfactorial[{0, 3, x}]", "List[1, 2, Subfactorial[x]]"),
        ("StirlingS1[{4, 5}, 2]", "List[11, -50]"),
        ("StirlingS2[5, {2, 3}]", "List[15, 25]"),
        ("CatalanNumber[{0, 4, x}]", "List[1, 14, CatalanNumber[x]]"),
        ("Multinomial[3, 2, 1]", "60"),
        (
            "Pochhammer[{2, a}, 3]",
            "List[24, Times[a, Plus[1, a], Plus[2, a]]]",
        ),
        ("Binomial[n, 2]", "Binomial[n, 2]"),
        ("Fibonacci[n]", "Fibonacci[n]"),
        ("PartitionsP[n]", "PartitionsP[n]"),
        ("PartitionsP[3/2]", "PartitionsP[3/2]"),
        ("PartitionsP[-1]", "PartitionsP[-1]"),
        ("PartitionsQ[n]", "PartitionsQ[n]"),
        ("PartitionsQ[3/2]", "PartitionsQ[3/2]"),
        ("BernoulliB[n]", "BernoulliB[n]"),
        ("BernoulliB[3/2]", "BernoulliB[3/2]"),
        ("BernoulliB[-1]", "BernoulliB[-1]"),
        ("Subfactorial[n]", "Subfactorial[n]"),
        ("Subfactorial[3/2]", "Subfactorial[3/2]"),
        ("StirlingS1[n, 2]", "StirlingS1[n, 2]"),
        ("StirlingS1[3, n]", "StirlingS1[3, n]"),
        ("StirlingS1[-1, 1]", "StirlingS1[-1, 1]"),
        ("StirlingS2[n, 2]", "StirlingS2[n, 2]"),
        ("StirlingS2[3, -1]", "StirlingS2[3, -1]"),
        ("CatalanNumber[n]", "CatalanNumber[n]"),
        ("CatalanNumber[3/2]", "CatalanNumber[3/2]"),
        ("CatalanNumber[-1]", "CatalanNumber[-1]"),
        ("Multinomial[1, n]", "Multinomial[1, n]"),
        ("Multinomial[1, 3/2]", "Multinomial[1, 3/2]"),
        ("Multinomial[1, -1]", "Multinomial[-1, 1]"),
        ("Pochhammer[a, n]", "Pochhammer[a, n]"),
        ("Pochhammer[a, 3/2]", "Pochhammer[a, 3/2]"),
        ("Pochhammer[a, -1]", "Pochhammer[a, -1]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn cheap_combinatorics_builtins_report_wolfram_kernel_attributes() {
    let mut session = core_session();

    let cases = [
        (
            "Attributes[Binomial]",
            "List[Listable, NumericFunction, Protected]",
        ),
        (
            "Attributes[Fibonacci]",
            "List[Listable, NumericFunction, Protected, ReadProtected]",
        ),
        ("Attributes[PartitionsP]", "List[Listable, Protected]"),
        ("Attributes[PartitionsQ]", "List[Listable, Protected]"),
        ("Attributes[BernoulliB]", "List[Listable, Protected]"),
        (
            "Attributes[Subfactorial]",
            "List[Listable, NumericFunction, Protected, ReadProtected]",
        ),
        ("Attributes[StirlingS1]", "List[Listable, Protected]"),
        ("Attributes[StirlingS2]", "List[Listable, Protected]"),
        (
            "Attributes[CatalanNumber]",
            "List[Listable, NumericFunction, Protected, ReadProtected]",
        ),
        (
            "Attributes[Multinomial]",
            "List[Listable, NumericFunction, Orderless, Protected]",
        ),
        (
            "Attributes[Pochhammer]",
            "List[Listable, NumericFunction, Protected]",
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn binomial_respects_integer_work_limit() {
    let mut session = core_session();
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

#[test]
fn recurrence_combinatorics_respect_integer_work_limit() {
    let mut session = core_session();
    session.set_integer_work_limit(3);

    let cases = [
        "PartitionsP[10]",
        "PartitionsQ[4]",
        "BernoulliB[4]",
        "Subfactorial[5]",
        "StirlingS1[5, 2]",
        "StirlingS2[5, 2]",
        "Pochhammer[a, 4]",
    ];

    for source in cases {
        let limited = session.eval_text(source).unwrap();
        assert_eq!(
            limited.value.to_string(),
            "TerminatedEvaluation[\"IntegerWorkLimit\"]",
            "{source}"
        );
        assert!(
            limited
                .messages
                .iter()
                .any(|message| message.symbol == "General" && message.tag == "reslim"),
            "{source}"
        );
    }

    assert_eq!(eval_to_string(&mut session, "StirlingS1[3, 4]"), "0");
    assert_eq!(eval_to_string(&mut session, "StirlingS2[3, 4]"), "0");
}
