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
