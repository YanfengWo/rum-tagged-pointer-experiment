use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn resume_eval_request_with_input_can_finish_fold_callback_loop() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "Fold[Function[List[Slot[1], InputString[\"fold: \"], Slot[2]]], seed, {a, b}]",
        )
        .expect_err("Fold callback should suspend on the first item");
    assert_input_prompt(&first.error, "fold: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("Fold should continue to the second item after resuming the first");
    assert_input_prompt(&second.error, "fold: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("Fold should finish after the second resumed callback");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed, \"one\", a], \"two\", b]"
    );
    assert!(session.pending_input_request().is_none());
}

#[test]
fn resume_eval_request_with_input_can_finish_foldlist_callback_loop() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "FoldList[Function[List[Slot[1], InputString[\"foldlist: \"], Slot[2]]], seed, {a}]",
        )
        .expect_err("FoldList callback should suspend");
    assert_input_prompt(&failure.error, "foldlist: ");

    let response = session
        .resume_eval_request_with_input("one")
        .expect("FoldList should preserve collected history after resuming");

    assert_eq!(
        response.value.to_string(),
        "List[seed, List[seed, \"one\", a]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_foldpair_operator_and_projector() {
    let mut session = core_session();

    let pair = session
        .eval_request(
            "FoldPair[Function[List[InputString[\"pair: \"], Slot[2]]], seed, {a}, Function[List[InputString[\"project: \"], Slot[1]]]]",
        )
        .expect_err("FoldPair operator should suspend before producing the pair");
    assert_input_prompt(&pair.error, "pair: ");

    let projector = session
        .resume_eval_request_with_input("x")
        .expect_err("FoldPair projector should run after the operator resumes");
    assert_input_prompt(&projector.error, "project: ");

    let response = session
        .resume_eval_request_with_input("p")
        .expect("FoldPair should finish after the projector resumes");

    assert_eq!(response.value.to_string(), "List[\"p\", List[\"x\", a]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_foldwhile_step_and_test() {
    let mut session = core_session();

    let step = session
        .eval_request(
            "FoldWhile[Function[List[Slot[1], InputString[\"step: \"], Slot[2]]], seed, {a}, Function[SameQ[InputString[\"test: \"], \"yes\"]]]",
        )
        .expect_err("FoldWhile operator should suspend");
    assert_input_prompt(&step.error, "step: ");

    let test = session
        .resume_eval_request_with_input("s")
        .expect_err("FoldWhile test should run after the step resumes");
    assert_input_prompt(&test.error, "test: ");

    let response = session
        .resume_eval_request_with_input("no")
        .expect("falsy resumed test should finish FoldWhile");

    assert_eq!(response.value.to_string(), "List[seed, \"s\", a]");
}

#[test]
fn resume_eval_request_with_input_can_finish_sequencefoldlist_callback_loop() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "SequenceFoldList[Function[List[Slot[1], InputString[\"seq: \"], Slot[2]]], {seed}, {a}]",
        )
        .expect_err("SequenceFoldList callback should suspend");
    assert_input_prompt(&failure.error, "seq: ");

    let response = session
        .resume_eval_request_with_input("one")
        .expect("SequenceFoldList should preserve seed history after resuming");

    assert_eq!(
        response.value.to_string(),
        "List[seed, List[seed, \"one\", a]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_accumulate_and_blockmap_callbacks() {
    let mut session = core_session();
    session
        .eval_text("a /: Plus[a, b] := InputString[\"acc: \"]")
        .unwrap();

    let failure = session
        .eval_request("Accumulate[f[a, b]]")
        .expect_err("Accumulate should suspend through the Plus callback");
    assert_input_prompt(&failure.error, "acc: ");

    let response = session
        .resume_eval_request_with_input("sum")
        .expect("Accumulate should rebuild the original head after resuming");
    assert_eq!(response.value.to_string(), "f[a, \"sum\"]");

    let failure = session
        .eval_request(
            "BlockMap[Function[List[InputString[\"block: \"], Slot[1]]], {a, b, c}, 2, 2]",
        )
        .expect_err("BlockMap callback should suspend");
    assert_input_prompt(&failure.error, "block: ");

    let response = session
        .resume_eval_request_with_input("blk")
        .expect("BlockMap should keep completed block results after resuming");
    assert_eq!(
        response.value.to_string(),
        "List[List[\"blk\", List[a, b]]]"
    );
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
