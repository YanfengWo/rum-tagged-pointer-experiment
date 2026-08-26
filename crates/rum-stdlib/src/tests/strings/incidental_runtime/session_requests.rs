use super::super::*;
use super::support::*;

// Moved from crates/rum-runtime/src/runtime/tests/session_requests/functional_tensor_string_resume.rs.

#[test]
fn resume_eval_request_with_input_can_finish_delayed_string_rules() {
    let mut session = core_session();

    let first = session
        .eval_request("StringReplace[{\"a\", \"a\"}, \"a\" :> InputString[\"replace: \"]]")
        .expect_err("StringReplace delayed rule should suspend on the first string");
    assert_input_prompt(&first.error, "replace: ");

    let second = session
        .resume_eval_request_with_input("x")
        .expect_err("StringReplace should continue to the second string after resuming");
    assert_input_prompt(&second.error, "replace: ");

    let response = session
        .resume_eval_request_with_input("y")
        .expect("StringReplace should finish after resumed delayed rules");
    assert_eq!(response.value.to_string(), "List[\"x\", \"y\"]");

    let first = session
        .eval_request("StringCases[{\"a\", \"b\"}, x_ :> InputString[\"case: \"]]")
        .expect_err("StringCases delayed rule should suspend on the first string");
    assert_input_prompt(&first.error, "case: ");

    let second = session
        .resume_eval_request_with_input("a")
        .expect_err("StringCases should continue to the second string after resuming");
    assert_input_prompt(&second.error, "case: ");

    let response = session
        .resume_eval_request_with_input("b")
        .expect("StringCases should finish after resumed delayed rules");
    assert_eq!(response.value.to_string(), "List[List[\"a\"], List[\"b\"]]");

    let first = session
        .eval_request(
            "text = StringJoin[StringRepeat[\"z\", 300], StringRepeat[\"a\", 16]]; StringLength[StringReplace[text, StringRepeat[\"a\", 16] :> InputString[\"wide replace: \"]]]",
        )
        .expect_err("wide literal StringReplace should preserve suspension");
    assert_input_prompt(&first.error, "wide replace: ");
    let response = session
        .resume_eval_request_with_input("x")
        .expect("wide literal StringReplace should resume");
    assert_eq!(response.value.to_string(), "301");

    let first = session
        .eval_request("StringCases[text, StringRepeat[\"a\", 16] :> InputString[\"wide case: \"]]")
        .expect_err("wide literal StringCases should preserve suspension");
    assert_input_prompt(&first.error, "wide case: ");
    let response = session
        .resume_eval_request_with_input("y")
        .expect("wide literal StringCases should resume");
    assert_eq!(response.value.to_string(), "List[\"y\"]");
}

#[test]
fn guarded_string_rules_resume_without_replaying_candidates_or_output() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "guardResumeCount = 0; StringReplace[\"aba\", {\"a\" :> (\"A\" /; (guardResumeCount = guardResumeCount + 1; True)), \"b\" :> (\"B\" /; (guardResumeCount = guardResumeCount + 1; InputString[\"replace guard: \"]; True))}]",
        )
        .expect_err("StringReplace should suspend inside the guard at b");
    assert_input_prompt(&first.error, "replace guard: ");

    let response = session
        .resume_eval_request_with_input("continue")
        .expect("StringReplace should resume the pending guard");
    assert_eq!(response.value.to_string(), "\"ABA\"");
    assert_eq!(
        session
            .eval_text("guardResumeCount")
            .unwrap()
            .value
            .to_string(),
        "3"
    );

    let first = session
        .eval_request(
            "caseGuardResumeCount = 0; StringCases[\"aba\", {\"a\" :> (\"A\" /; (caseGuardResumeCount = caseGuardResumeCount + 1; True)), \"b\" :> (\"B\" /; (caseGuardResumeCount = caseGuardResumeCount + 1; InputString[\"case guard: \"]; True))}]",
        )
        .expect_err("StringCases should suspend inside the guard at b");
    assert_input_prompt(&first.error, "case guard: ");

    let response = session
        .resume_eval_request_with_input("continue")
        .expect("StringCases should resume the pending guard");
    assert_eq!(response.value.to_string(), "List[\"A\", \"B\", \"A\"]");
    assert_eq!(
        session
            .eval_text("caseGuardResumeCount")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/session_requests/functional_tensor_string_resume.rs.

#[test]
fn resume_eval_request_with_input_can_finish_string_apply_list_callbacks() {
    let mut session = core_session();

    let first = session
        .eval_request("StringApply[Function[List[InputString[\"apply: \"]]], {\"ab\", \"cd\"}]")
        .expect_err("StringApply callback should suspend on the first string");
    assert_input_prompt(&first.error, "apply: ");

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("StringApply should continue to the second string after resuming");
    assert_input_prompt(&second.error, "apply: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("StringApply should finish after resumed callbacks");
    assert_eq!(response.value.to_string(), "List[\"one\", \"two\"]");
}

// Mid-walk suspension over a list: the strings before the suspending one
// resolve without suspending, so their outputs must already be accumulated
// when the frame is taken, and the walk must resume at the suspending index.

#[test]
fn string_replace_and_cases_resume_mid_walk_with_accumulated_output() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "StringReplace[{\"a\", \"b\", \"c\", \"b\", \"d\"}, \"b\" :> InputString[\"replace: \"]]",
        )
        .expect_err("StringReplace should suspend mid-walk on the first b");
    assert_input_prompt(&first.error, "replace: ");

    let second = session
        .resume_eval_request_with_input("x")
        .expect_err("StringReplace should suspend again on the second b after resuming");
    assert_input_prompt(&second.error, "replace: ");

    let response = session
        .resume_eval_request_with_input("y")
        .expect("StringReplace should finish the walk after the second b resolves");
    assert_eq!(
        response.value.to_string(),
        "List[\"a\", \"x\", \"c\", \"y\", \"d\"]"
    );

    let first = session
        .eval_request("StringCases[{\"a\", \"b\", \"a\"}, \"b\" :> InputString[\"case: \"]]")
        .expect_err("StringCases should suspend mid-walk on the middle string");
    assert_input_prompt(&first.error, "case: ");

    let response = session
        .resume_eval_request_with_input("x")
        .expect("StringCases should finish the walk after the middle string resolves");
    assert_eq!(
        response.value.to_string(),
        "List[List[], List[\"x\"], List[]]"
    );
}

#[test]
fn string_apply_resumes_mid_walk_with_accumulated_output() {
    let mut session = core_session();

    let first = session
        .eval_request(
            "StringApply[Function[If[SameQ[Slot[1], List[\"b\"]], InputString[\"apply: \"], StringJoin[Slot[1]]]], {\"a\", \"b\", \"c\"}]",
        )
        .expect_err("StringApply should suspend mid-walk on the middle string");
    assert_input_prompt(&first.error, "apply: ");

    let response = session
        .resume_eval_request_with_input("x")
        .expect("StringApply should finish the walk after the middle string resolves");
    assert_eq!(response.value.to_string(), "List[\"a\", \"x\", \"c\"]");
}

// Moved from crates/rum-runtime/src/runtime/tests/session_requests/transforms_resume.rs.

#[test]
fn resume_eval_request_with_input_can_finish_replacepart_pattern_bindings_in_delayed_rhs() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "ReplacePart[f[g[a]], {1, i_} :> StringJoin[InputString[\"rhs: \"], ToString[i]]]",
        )
        .expect_err(
            "ReplacePart should suspend when a delayed rhs with pattern bindings requests input",
        );

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "rhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue ReplacePart after the delayed rhs resolves");

    assert_eq!(response.value.to_string(), "f[g[\"done1\"]]");
}

// Moved from crates/rum-runtime/src/runtime/tests/session_requests/transforms_resume_associations.rs.

#[test]
fn resume_eval_request_with_input_can_finish_keysortby_after_suspending_sort_key() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "KeySortBy[{bb -> 2, a -> 1}, Function[If[SameQ[Slot[1], bb], StringLength[InputString[\"key: \"]], StringLength[ToString[Slot[1]]]]]]",
        )
        .expect_err("KeySortBy should suspend when a transformed sort key requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "key: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("xxxx")
        .expect("resume should continue KeySortBy after the transformed sort key resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[a, 1], Rule[bb, 2]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_keysortby_after_each_sort_key_suspends() {
    let mut session = core_session();

    let first = session
        .eval_request("KeySortBy[{c -> 3, a -> 1, b -> 2}, Function[InputString[\"key: \"]]]")
        .expect_err("KeySortBy should suspend on the first transformed sort key");
    assert_input_prompt(&first.error, "key: ");

    let second = session
        .resume_eval_request_with_input("3")
        .expect_err("KeySortBy should continue to the second transformed sort key");
    assert_input_prompt(&second.error, "key: ");

    let third = session
        .resume_eval_request_with_input("1")
        .expect_err("KeySortBy should continue to the third transformed sort key");
    assert_input_prompt(&third.error, "key: ");

    let response = session
        .resume_eval_request_with_input("2")
        .expect("KeySortBy should finish after all transformed sort keys resume");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
}
