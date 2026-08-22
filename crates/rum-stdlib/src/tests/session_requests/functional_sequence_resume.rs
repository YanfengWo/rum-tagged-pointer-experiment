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
fn resume_eval_request_with_input_can_finish_sortby_key_loop() {
    let mut session = core_session();

    let first = session
        .eval_request("SortBy[{first, second}, Function[x, InputString[\"sort-key: \"]]]")
        .expect_err("SortBy key evaluation should suspend on the first item");
    assert_input_prompt(&first.error, "sort-key: ");

    let second = session
        .resume_eval_request_with_input("z")
        .expect_err("SortBy should continue to the second key after resuming the first");
    assert_input_prompt(&second.error, "sort-key: ");

    let response = session
        .resume_eval_request_with_input("a")
        .expect("SortBy should preserve completed keys after resuming");

    assert_eq!(response.value.to_string(), "List[second, first]");
}

#[test]
fn resume_eval_request_with_input_can_finish_custom_sort_comparator_loops() {
    for (source, expected) in [
        (
            r#"calls = 0;
               {Sort[{2, 1}, Function[{a, b}, calls = calls + 1; If[calls == 1, SameQ[InputString["sort-compare: "], "yes"], Less[a, b]]]], calls}"#,
            "List[List[1, 2], 2]",
        ),
        (
            r#"calls = 0;
               {SortBy[{2, 1}, Identity, Function[{a, b}, calls = calls + 1; If[calls == 1, SameQ[InputString["sort-compare: "], "yes"], Less[a, b]]]], calls}"#,
            "List[List[1, 2], 2]",
        ),
        (
            r#"calls = 0;
               {ReverseSort[{1, 2}, Function[{a, b}, calls = calls + 1; If[calls == 1, SameQ[InputString["sort-compare: "], "yes"], Less[a, b]]]], calls}"#,
            "List[List[2, 1], 2]",
        ),
    ] {
        let mut session = core_session();
        let failure = session
            .eval_request(source)
            .expect_err("custom comparator should suspend on its first comparison");
        assert_input_prompt(&failure.error, "sort-compare: ");

        let response = session
            .resume_eval_request_with_input("no")
            .expect("custom comparator should finish after resuming");
        assert_eq!(response.value.to_string(), expected);
    }
}

#[test]
fn resume_eval_request_with_input_can_finish_custom_equivalence_loops() {
    for (source, expected) in [
        (
            r#"calls = 0;
               {Tally[{1, 2, 3}, Function[{a, b}, calls = calls + 1; If[calls == 1, SameQ[InputString["equivalence: "], "yes"], False]]], calls}"#,
            "List[List[List[1, 1], List[2, 1], List[3, 1]], 3]",
        ),
        (
            r#"calls = 0;
               {DeleteDuplicates[{1, 2, 3}, Function[{a, b}, calls = calls + 1; If[calls == 1, SameQ[InputString["equivalence: "], "yes"], False]]], calls}"#,
            "List[List[1, 2, 3], 3]",
        ),
    ] {
        let mut session = core_session();
        let failure = session
            .eval_request(source)
            .expect_err("custom equivalence test should suspend on its first comparison");
        assert_input_prompt(&failure.error, "equivalence: ");

        let response = session
            .resume_eval_request_with_input("no")
            .expect("custom equivalence loop should finish after resuming");
        assert_eq!(response.value.to_string(), expected);
    }
}

#[test]
fn resume_eval_request_with_input_can_finish_gatherby_function_list_key_loop() {
    let mut session = core_session();

    let outer_first = session
        .eval_request(
            "GatherBy[{first, second}, {Function[x, InputString[\"outer: \"]], Function[x, InputString[\"inner: \"]]}]",
        )
        .expect_err("GatherBy should suspend on the first outer key");
    assert_input_prompt(&outer_first.error, "outer: ");

    let inner_first = session
        .resume_eval_request_with_input("same")
        .expect_err("GatherBy should continue to the first inner key");
    assert_input_prompt(&inner_first.error, "inner: ");

    let outer_second = session
        .resume_eval_request_with_input("left")
        .expect_err("GatherBy should continue to the second outer key");
    assert_input_prompt(&outer_second.error, "outer: ");

    let inner_second = session
        .resume_eval_request_with_input("same")
        .expect_err("GatherBy should continue to the second inner key");
    assert_input_prompt(&inner_second.error, "inner: ");

    let response = session
        .resume_eval_request_with_input("right")
        .expect("GatherBy should preserve all completed keys after resuming");
    assert_eq!(
        response.value.to_string(),
        "List[List[List[first], List[second]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_delete_duplicates_by_key_loop() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "DeleteDuplicatesBy[{first, second}, Function[x, InputString[\"dedup-key: \"]]]",
        )
        .expect_err("DeleteDuplicatesBy should suspend on the first key");
    assert_input_prompt(&first.error, "dedup-key: ");

    let second = session
        .resume_eval_request_with_input("same")
        .expect_err("DeleteDuplicatesBy should continue to the second key");
    assert_input_prompt(&second.error, "dedup-key: ");

    let response = session
        .resume_eval_request_with_input("same")
        .expect("DeleteDuplicatesBy should retain its first representative after resuming");
    assert_eq!(response.value.to_string(), "List[first]");
}

#[test]
fn resume_eval_request_with_input_can_finish_foldpair_operator_and_projector() {
    let mut session = core_session();

    let pair = session
        .eval_request(
            "FoldPair[Function[List[InputString[\"pair: \"], Slot[2]]], seed, {a, b}, Function[List[InputString[\"project: \"], Slot[1]]]]",
        )
        .expect_err("FoldPair operator should suspend before producing the pair");
    assert_input_prompt(&pair.error, "pair: ");

    let projector = session
        .resume_eval_request_with_input("x")
        .expect_err("FoldPair projector should run after the operator resumes");
    assert_input_prompt(&projector.error, "project: ");

    let second_pair = session
        .resume_eval_request_with_input("p")
        .expect_err("FoldPair should continue to the second item");
    assert_input_prompt(&second_pair.error, "pair: ");

    let second_projector = session
        .resume_eval_request_with_input("y")
        .expect_err("FoldPair should project the second pair");
    assert_input_prompt(&second_projector.error, "project: ");

    let response = session
        .resume_eval_request_with_input("q")
        .expect("FoldPair should finish after both items");

    assert_eq!(response.value.to_string(), "List[\"q\", List[\"y\", b]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_foldwhile_step_and_test() {
    let mut session = core_session();

    let step = session
        .eval_request(
            "FoldWhile[Function[List[Slot[1], InputString[\"step: \"], Slot[2]]], seed, {a, b}, Function[SameQ[InputString[\"test: \"], \"yes\"]]]",
        )
        .expect_err("FoldWhile operator should suspend");
    assert_input_prompt(&step.error, "step: ");

    let test = session
        .resume_eval_request_with_input("s")
        .expect_err("FoldWhile test should run after the step resumes");
    assert_input_prompt(&test.error, "test: ");

    let second_step = session
        .resume_eval_request_with_input("yes")
        .expect_err("truthy resumed test should continue FoldWhile");
    assert_input_prompt(&second_step.error, "step: ");

    let second_test = session
        .resume_eval_request_with_input("t")
        .expect_err("FoldWhile should test the second step");
    assert_input_prompt(&second_test.error, "test: ");

    let response = session
        .resume_eval_request_with_input("no")
        .expect("falsy second test should finish FoldWhile");

    assert_eq!(
        response.value.to_string(),
        "List[List[seed, \"s\", a], \"t\", b]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_sequencefoldlist_callback_loop() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "SequenceFoldList[Function[List[Slot[1], InputString[\"seq: \"], Slot[2]]], {seed}, {a, b}]",
        )
        .expect_err("SequenceFoldList callback should suspend");
    assert_input_prompt(&failure.error, "seq: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("SequenceFoldList should continue to the second item");
    assert_input_prompt(&second.error, "seq: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("SequenceFoldList should preserve completed history after resuming");

    assert_eq!(
        response.value.to_string(),
        "List[seed, List[seed, \"one\", a], List[List[seed, \"one\", a], \"two\", b]]"
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
