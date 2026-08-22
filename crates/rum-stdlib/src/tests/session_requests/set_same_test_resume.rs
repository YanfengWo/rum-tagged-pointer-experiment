use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn custom_same_test_set_operations_preserve_progress_across_input() {
    assert_eq!(
        eval_with_false_inputs(
            "Union[{c, b, a}, SameTest -> Function[SameQ[InputString[\"union: \"], \"yes\"]]]",
            "union: ",
            3,
        ),
        "List[a, b, c]"
    );
    assert_eq!(
        eval_with_false_inputs(
            "Intersection[{a, b}, {x, y}, SameTest -> Function[SameQ[InputString[\"intersection: \"], \"yes\"]]]",
            "intersection: ",
            4,
        ),
        "List[]"
    );
    assert_eq!(
        eval_with_false_inputs(
            "Complement[{a, b}, {x, y}, SameTest -> Function[SameQ[InputString[\"complement: \"], \"yes\"]]]",
            "complement: ",
            5,
        ),
        "List[a, b]"
    );
}

fn eval_with_false_inputs(source: &str, prompt: &str, input_count: usize) -> String {
    let mut session = core_session();
    let mut result = session.eval_request(source);

    for _ in 0..input_count {
        let failure = result.expect_err("SameTest should request the next input");
        assert_input_prompt(&failure.error, prompt);
        result = session.resume_eval_request_with_input("no");
    }

    result
        .expect("SameTest operation should finish after all comparisons resume")
        .value
        .to_string()
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
