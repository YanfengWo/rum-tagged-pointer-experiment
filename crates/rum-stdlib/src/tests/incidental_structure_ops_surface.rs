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
fn arithmetic_builtins_thread_over_lists() {
    let mut session = core_session();

    let cases = [
        ("Abs[{-1,-2,3}]", "List[1, 2, 3]"),
        ("GCD[{12,18},6]", "List[6, 6]"),
        ("LCM[{4,6},3]", "List[12, 6]"),
        ("Mod[{5,7},3]", "List[2, 1]"),
        ("Quotient[{17,9},5]", "List[3, 1]"),
        ("PowerMod[{2,3},10,7]", "List[2, 4]"),
        ("Numerator[{3/4,5/6}]", "List[3, 5]"),
        ("Denominator[{3/4,5/6}]", "List[4, 6]"),
        ("Factorial[{3,4}]", "List[6, 24]"),
        ("Factorial2[{4,5}]", "List[8, 15]"),
        ("CoprimeQ[{4,9},3]", "List[True, False]"),
        ("Round[{3/2,5/2,7/3}]", "List[2, 2, 2]"),
        ("Abs[-5]", "5"),
        ("GCD[12,18]", "6"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn hold_form_holds_argument_and_fullform_shows_wrapper() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "Attributes[HoldForm]"),
        "List[HoldAll, Protected]"
    );
    assert_eq!(
        eval_to_string(&mut session, "FullForm[HoldForm[1 + 1]]"),
        "\"HoldForm[Plus[1, 1]]\""
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "FullForm[HoldForm[1 + 1]] === \"HoldForm[2]\""
        ),
        "False"
    );
}

#[test]
fn fullform_evaluates_its_argument_but_preserves_inner_hold() {
    let mut session = core_session();

    assert_eq!(eval_to_string(&mut session, "FullForm[1+1]"), "\"2\"");
    assert_eq!(
        eval_to_string(&mut session, "FullForm[Hold[1+1]]"),
        "\"Hold[Plus[1, 1]]\""
    );
    assert_eq!(
        eval_to_string(&mut session, "Hold[Evaluate[1+1]] === Hold[2]"),
        "True"
    );
}
