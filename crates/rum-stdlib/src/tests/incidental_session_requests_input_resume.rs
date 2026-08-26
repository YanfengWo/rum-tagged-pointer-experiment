use crate::install_core;
use rum_core::Expr;
use rum_runtime::{Session, SessionRequestError, kernel_api::system_expr};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_can_finish_select_predicate() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "Select[{1, 2}, Function[x, If[SameQ[x, 2], MatchQ[InputString[\"keep: \"], \"yes\"], False]]]",
        )
        .expect_err("Select predicate should suspend when it requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "keep: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("resume should finish Select after the predicate resolves");

    assert_eq!(response.value.to_string(), "List[2]");
}

#[test]
fn resume_eval_request_with_input_preserves_logical_sequence_state() {
    for symbol in ["And", "Or", "Nand", "Nor"] {
        let mut session = core_session();
        let prompt = format!("{symbol}: ");
        let failure = session
            .eval_request(&format!("{symbol}[x, InputString[\"{prompt}\"], y]"))
            .expect_err("logical sequence should suspend when it requests input");

        match &failure.error {
            SessionRequestError::InputRequired { request } => {
                assert_eq!(request.prompt, prompt);
            }
            other => panic!("expected input-required failure, got {other:?}"),
        }

        let response = session
            .resume_eval_request_with_input("ok")
            .expect("logical sequence should resume from its preserved position");
        assert_eq!(
            response.value.to_string(),
            format!("{symbol}[x, \"ok\", y]")
        );
    }
}

#[test]
fn resume_eval_request_with_input_preserves_ordering_comparator_state() {
    let mut ordered_session = core_session();
    let ordered_failure = ordered_session
        .eval_request(
            "OrderedQ[{1, 2, 3}, Function[{a, b}, If[SameQ[a, 1], SameQ[InputString[\"ordered: \"], \"yes\"], Less[a, b]]]]",
        )
        .expect_err("OrderedQ comparator should suspend when it requests input");
    match &ordered_failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "ordered: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(
        ordered_session
            .resume_eval_request_with_input("yes")
            .expect("OrderedQ should resume from its preserved comparator position")
            .value
            .to_string(),
        "True"
    );

    let mut lexicographic_session = core_session();
    let lexicographic_failure = lexicographic_session
        .eval_request(
            "seen = False; LexicographicOrder[{1, 2}, {1, 3}, Function[{a, b}, If[Not[seen], seen = True; SameQ[InputString[\"lexicographic: \"], \"yes\"], Less[a, b]]]]",
        )
        .expect_err("LexicographicOrder comparator should suspend when it requests input");
    match &lexicographic_failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "lexicographic: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(
        lexicographic_session
            .resume_eval_request_with_input("no")
            .expect("LexicographicOrder should resume from its preserved comparator position")
            .value
            .to_string(),
        "1"
    );
}

#[test]
fn resume_eval_request_with_input_preserves_predicate_traversal_state() {
    let mut truth_session = core_session();
    let truth_failure = truth_session
        .eval_request(
            "AllTrue[{1, 2, 3}, Function[x, If[SameQ[x, 2], SameQ[InputString[\"all: \"], \"yes\"], Positive[x]]]]",
        )
        .expect_err("AllTrue criterion should suspend when it requests input");
    match &truth_failure.error {
        SessionRequestError::InputRequired { request } => assert_eq!(request.prompt, "all: "),
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(
        truth_session
            .resume_eval_request_with_input("yes")
            .expect("AllTrue should resume from its preserved traversal position")
            .value
            .to_string(),
        "True"
    );

    let mut array_session = core_session();
    let array_failure = array_session
        .eval_request(
            "VectorQ[{1, 2, 3}, Function[x, If[SameQ[x, 2], SameQ[InputString[\"vector: \"], \"yes\"], IntegerQ[x]]]]",
        )
        .expect_err("VectorQ leaf test should suspend when it requests input");
    match &array_failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "vector: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
    assert_eq!(
        array_session
            .resume_eval_request_with_input("yes")
            .expect("VectorQ should resume from its preserved leaf position")
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn resume_eval_request_with_input_preserves_do_iterator_scope_until_completion() {
    let mut session = core_session();
    session.eval_text("i = global").unwrap();
    session.eval_text("seen = none").unwrap();

    let failure = session
        .eval_request(
            "Do[If[SameQ[i, 2], seen = List[i, InputString[\"iter: \"], i], Null], {i, 3}]; List[seen, i]",
        )
        .expect_err("Do iterator body should suspend when it reaches InputString");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "iter: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resume should continue inside the active Do iterator scope");

    assert_eq!(
        response.value.to_string(),
        "List[List[2, \"ok\", 2], global]"
    );
    assert_eq!(session.eval_text("i").unwrap().value.to_string(), "global");
}

#[test]
fn resumed_set_rhs_normalizes_iterator_lhs_once_and_restores_clean_scope() {
    let mut session = core_session();
    let setup = session
        .eval_text("ClearAll[i, f, calls]; calls = 0")
        .unwrap();
    assert_eq!(setup.value.to_string(), "0");
    assert!(setup.messages.is_empty());

    let failure = session
        .eval_request(
            "Do[f[i] = List[i, calls = calls + 1, If[SameQ[i, 1], InputString[\"iter-set: \"], done], i], {i, 1, 2}]; List[DownValues[f], calls, i]",
        )
        .expect_err("the first Set rhs should suspend inside the clean iterator scope");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "iter-set: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("ok")
        .expect("resume should finish Set without replaying its rhs");

    assert_eq!(
        response.value.to_string(),
        "List[List[RuleDelayed[HoldPattern[f[1]], List[1, 1, \"ok\", 1]], RuleDelayed[HoldPattern[f[2]], List[2, 2, done, 2]]], 2, i]"
    );
    assert!(response.messages.is_empty());

    let restored = session.eval_text("List[i, OwnValues[i]]").unwrap();
    assert_eq!(restored.value.to_string(), "List[i, List[]]");
    assert!(restored.messages.is_empty());
}

#[test]
fn resume_eval_request_with_input_preserves_prepared_lookup_across_default_and_wrapper() {
    let mut session = core_session();

    let default_request = session
        .eval_request(
            "Lookup[Association[a -> 1], {a, b}, InputString[\"default: \"], If[SameQ[InputString[\"wrapper: \"], \"yes\"], Hold, Identity]]",
        )
        .expect_err("Lookup should suspend while evaluating a needed default");
    match &default_request.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "default: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let wrapper_request = session
        .resume_eval_request_with_input("fallback")
        .expect_err("Lookup should then suspend while evaluating a needed wrapper");
    match &wrapper_request.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "wrapper: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("yes")
        .expect("Lookup should finish from its prepared result after both inputs");
    assert_eq!(response.value.to_string(), "List[Hold[1], \"fallback\"]");
}

#[test]
fn resume_eval_request_with_input_finishes_evaluation_data_capture() {
    let mut session = core_session();

    let failure = session
        .eval_request("EvaluationData[Echo[1 + 1]; InputString[\"name: \"]]")
        .expect_err("EvaluationData should suspend when the wrapped evaluation requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "name: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("Alice")
        .expect("resume should finish the suspended EvaluationData");

    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![response.value.clone(), Expr::string("Result")],
            ))
            .unwrap()
            .to_string(),
        "\"Alice\""
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![response.value.clone(), Expr::string("OutputLog")],
            ))
            .unwrap()
            .to_string(),
        "List[\"2\"]"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![response.value.clone(), Expr::string("Input")],
            ))
            .unwrap()
            .to_string(),
        "Hold[CompoundExpression[Echo[Plus[1, 1]], InputString[\"name: \"]]]"
    );
    assert_eq!(
        session
            .evaluate(Expr::call(
                system_expr("Lookup"),
                vec![response.value, Expr::string("Success")],
            ))
            .unwrap()
            .to_string(),
        "True"
    );
}
