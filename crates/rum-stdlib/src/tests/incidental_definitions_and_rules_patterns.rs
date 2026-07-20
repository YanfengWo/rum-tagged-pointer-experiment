use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn compiled_downvalue_matcher_falls_back_for_condition_patterns() {
    let mut session = core_session();

    session.eval_text("p[x_] /; x > 0 := x").unwrap();

    assert_eq!(session.eval_text("p[3]").unwrap().value.to_string(), "3");
    assert_eq!(
        session.eval_text("p[-1]").unwrap().value.to_string(),
        "p[-1]"
    );
}

#[test]
fn orderless_sequence_backtracking_depth_limit_terminates_gracefully() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let mut session = core_session();

            let response = session
                .eval_text("SetAttributes[f, Orderless]; MatchQ[f @@ Range[14], f[a___, b___, c___] /; False]")
                .unwrap();

            assert_eq!(
                response.value.to_string(),
                "TerminatedEvaluation[\"PatternMatchRecursionLimit\"]"
            );
            assert!(response
                .messages
                .iter()
                .any(|message| message.symbol == "General" && message.tag == "matchreclim"));
        })
        .expect("depth-limit regression test thread should start")
        .join()
        .expect("depth-limit regression test should finish");
}

#[test]
fn install_seeds_mutable_arithmetic_head_defaults() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Default[Plus]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("Default[Times]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );

    session
        .eval_text("Default[Plus] = 9; Default[Times] = 8")
        .unwrap();
    assert_eq!(
        session
            .eval_text("{Default[Plus], Default[Times]}")
            .unwrap()
            .value
            .to_string(),
        "List[9, 8]"
    );

    session
        .eval_text("Unset[Default[Plus]]; Unset[Default[Times]]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("{Default[Plus], Default[Times]}")
            .unwrap()
            .value
            .to_string(),
        "List[Default[Plus], Default[Times]]"
    );
}

#[test]
fn arithmetic_one_identity_optional_idioms_use_installed_defaults() {
    let mut session = core_session();

    for (source, expected) in [
        ("MatchQ[5, x_ + y_.]", "True"),
        ("5 /. x_ + y_. :> {x, y}", "List[5, 0]"),
        ("7 /. x_*y_. :> {x, y}", "List[7, 1]"),
        ("MatchQ[f[a], x_ + y_.]", "True"),
        ("f[a] /. x_ + y_. :> {x, y}", "List[f[a], 0]"),
        ("MatchQ[5, x_ + y_]", "False"),
        ("MatchQ[5, x_. + y_.]", "True"),
        ("a + b /. x_ + y_. :> {x, y}", "List[b, a]"),
    ] {
        assert_eq!(
            session.eval_text(source).unwrap().value.to_string(),
            expected,
            "source: {source}"
        );
    }
}

#[test]
fn nested_compound_optional_idiom_remains_unmatched_at_the_power_boundary() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("7 /. c_. * b_^e_. :> {c, b, e}")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
}

#[test]
fn arithmetic_defaults_and_promoted_matching_survive_snapshot_roundtrip() {
    let source = core_session();
    let snapshot = source
        .snapshot_definitions_source()
        .expect("fresh core snapshot should render");

    let mut restored = core_session();
    let outcome = restored
        .eval_script(&snapshot)
        .expect("fresh core snapshot should restore");
    assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);

    assert_eq!(
        restored
            .eval_text("Default[Plus]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        restored
            .eval_text("5 /. x_ + y_. :> {x, y}")
            .unwrap()
            .value
            .to_string(),
        "List[5, 0]"
    );
}
