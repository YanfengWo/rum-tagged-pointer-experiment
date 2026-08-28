use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn held_part_update_resumes_two_selectors_without_replay_and_refetches_owner() {
    const SOURCE: &str = "x = {{1}, {2}}; seen = {}; \
        AddTo[x[[(AppendTo[seen, firstBefore]; InputString[\"first: \"]; \
                     AppendTo[seen, firstAfter]; x = {{10}, {20}}; 1), \
                  (AppendTo[seen, secondBefore]; InputString[\"second: \"]; \
                     AppendTo[seen, secondAfter]; x = {{30}, {40}}; 1)]], 5]; \
        {x, seen}";

    let mut resumed_session = core_session();
    let first = resumed_session
        .eval_request(SOURCE)
        .expect_err("the first selector should request input");
    assert_input_prompt(&first.error, "first: ");

    let second = resumed_session
        .resume_eval_request_with_input("one")
        .expect_err("the second selector should request input after the first resumes");
    assert_input_prompt(&second.error, "second: ");

    let resumed = resumed_session
        .resume_eval_request_with_input("two")
        .expect("the held update should finish after both selectors resume");
    assert_eq!(
        resumed.value.to_string(),
        "List[List[List[35], List[40]], List[firstBefore, firstAfter, secondBefore, secondAfter]]"
    );
    assert!(resumed.messages.is_empty());

    let mut queued_session = core_session();
    queued_session.queue_input_response("one");
    queued_session.queue_input_response("two");
    let queued = queued_session
        .eval_request(SOURCE)
        .expect("queued inputs should follow the same two-selector path");
    assert_eq!(queued.value, resumed.value);
    assert_eq!(queued.messages, resumed.messages);
}

#[test]
fn association_part_mutation_resumes_one_selector_without_replay_and_refetches_owner() {
    const SOURCE: &str = "parts = {Association[a -> 1]}; seen = {}; \
        AssociateTo[parts[[(AppendTo[seen, before]; InputString[\"selector: \"]; \
                              AppendTo[seen, after]; \
                              parts = {Association[a -> 10]}; 1)]], b -> 2]; \
        {parts, seen}";

    let mut session = core_session();
    let failure = session
        .eval_request(SOURCE)
        .expect_err("the association Part selector should request input");
    assert_input_prompt(&failure.error, "selector: ");

    let response = session
        .resume_eval_request_with_input("ready")
        .expect("the association Part mutation should finish after selector input");
    assert_eq!(
        response.value.to_string(),
        "List[List[Association[Rule[a, 10], Rule[b, 2]]], List[before, after]]"
    );
    assert!(response.messages.is_empty());
}

#[test]
fn associateto_requires_an_immediate_symbol_value_instead_of_suspending_delayed_values() {
    let mut session = core_session();
    session
        .eval_text("assoc := Association[a -> InputString[\"value: \"]]")
        .unwrap();

    let response = session
        .eval_request("AssociateTo[assoc, b -> 2]")
        .expect("AssociateTo should reject delayed values without trying to evaluate them");

    assert_eq!(response.value.to_string(), "AssociateTo[assoc, Rule[b, 2]]");
    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.messages[0].symbol, "AssociateTo");
    assert_eq!(response.messages[0].tag, "blnoval");
    match session
        .eval_request("assoc")
        .expect_err("delayed assoc should still request input")
        .error
    {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required error, got {other:?}"),
    }
}

#[test]
fn resume_eval_request_with_input_can_finish_keyunion_after_suspending_fill_function() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "KeyUnion[{Association[a -> 1], Association[]}, Function[k, InputString[\"fill: \"]]]",
        )
        .expect_err("KeyUnion should suspend when the fill function requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "fill: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue KeyUnion after the fill function resolves");

    assert_eq!(
        response.value.to_string(),
        "List[Association[Rule[a, 1]], Association[Rule[a, \"done\"]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_merge_after_suspending_delayed_association_value() {
    let mut session = core_session();

    let failure = session
        .eval_request("Merge[{Association[a :> InputString[\"value: \"]]}, Function[v, v]]")
        .expect_err("Merge should suspend when its combiner evaluates a delayed association value");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue Merge after the delayed association value resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[a, List[\"done\"]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_catenate_after_suspending_delayed_association_value() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "Catenate[Association[a :> {InputString[\"value: \"]}, b :> {InputString[\"value: \"]}]]",
        )
        .expect_err("Catenate should suspend when a delayed association value requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("Catenate should continue to the second delayed value");
    assert_input_prompt(&second.error, "value: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("resume should continue Catenate after both delayed values resolve");

    assert_eq!(response.value.to_string(), "List[\"one\", \"two\"]");

    let first = session
        .eval_request(
            "Catenate[{Association[a :> InputString[\"step: \"]], Association[b :> InputString[\"step: \"]]}]",
        )
        .expect_err("list Catenate should suspend on the first association value");
    assert_input_prompt(&first.error, "step: ");

    let second = session
        .resume_eval_request_with_input("left")
        .expect_err("list Catenate should continue to the second association value");
    assert_input_prompt(&second.error, "step: ");

    let response = session
        .resume_eval_request_with_input("right")
        .expect("list Catenate should retain the first resumed value");
    assert_eq!(response.value.to_string(), "List[\"left\", \"right\"]");
}

#[test]
fn resume_eval_request_with_input_can_finish_counts_after_suspending_delayed_association_value() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "Counts[Association[a :> InputString[\"value: \"], b :> InputString[\"value: \"]], {\"one\", \"two\", \"other\"}]",
        )
        .expect_err("Counts should suspend when a delayed association value requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("Counts should continue to the second delayed association value");
    assert_input_prompt(&second.error, "value: ");

    let response = session
        .resume_eval_request_with_input("two")
        .expect("resume should continue Counts after both delayed association values resolve");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[\"one\", 1], Rule[\"two\", 1], Rule[\"other\", 0]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_countsby_after_multiple_key_suspensions() {
    let mut session = core_session();

    let first = session
        .eval_request("CountsBy[{a, b}, Function[x, InputString[\"key: \"]]]")
        .expect_err("CountsBy should suspend while keying the first item");
    assert_input_prompt(&first.error, "key: ");

    let second = session
        .resume_eval_request_with_input("same")
        .expect_err("CountsBy should continue to the second item after resuming");
    assert_input_prompt(&second.error, "key: ");

    let response = session
        .resume_eval_request_with_input("same")
        .expect("CountsBy should preserve its accumulated count after resuming");
    assert_eq!(response.value.to_string(), "Association[Rule[\"same\", 2]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_keygroupby_key_and_reducer_loops() {
    let mut session = core_session();

    let first_key = session
        .eval_request(
            "KeyGroupBy[Association[a -> 1, b -> 2], Function[k, InputString[\"key: \"]], Function[g, InputString[\"reduce: \"]]]",
        )
        .expect_err("KeyGroupBy should suspend while evaluating the first key");
    assert_input_prompt(&first_key.error, "key: ");

    let second_key = session
        .resume_eval_request_with_input("ka")
        .expect_err("KeyGroupBy should continue to the second key");
    assert_input_prompt(&second_key.error, "key: ");

    let first_reducer = session
        .resume_eval_request_with_input("kb")
        .expect_err("KeyGroupBy should start reducing after all keys are ready");
    assert_input_prompt(&first_reducer.error, "reduce: ");

    let second_reducer = session
        .resume_eval_request_with_input("va")
        .expect_err("KeyGroupBy should continue to the second reducer");
    assert_input_prompt(&second_reducer.error, "reduce: ");

    let response = session
        .resume_eval_request_with_input("vb")
        .expect("KeyGroupBy should preserve grouping and reduced prefixes after resuming");
    assert_eq!(
        response.value.to_string(),
        "Association[Rule[\"ka\", \"va\"], Rule[\"kb\", \"vb\"]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_groupby_after_suspending_delayed_association_value() {
    let mut session = core_session();

    let failure = session
        .eval_request("GroupBy[Association[a :> InputString[\"value: \"]], Identity]")
        .expect_err("GroupBy should suspend when keying a delayed association value");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue GroupBy after the delayed association value resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[\"done\", Association[RuleDelayed[a, InputString[\"value: \"]]]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_groupby_after_suspending_value_transform() {
    let mut session = core_session();

    let failure = session
        .eval_request("GroupBy[Association[a -> key], Identity -> (InputString[\"value: \"] &)]")
        .expect_err("GroupBy should suspend when the value transform requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue GroupBy after the value transform resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[key, Association[Rule[a, \"done\"]]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_groupby_list_key_and_value_loops() {
    let mut session = core_session();

    let key_a = session
        .eval_request(
            "GroupBy[{a, b}, Function[x, InputString[\"key: \"]] -> Function[x, InputString[\"value: \"]]]",
        )
        .expect_err("GroupBy should suspend while keying the first list item");
    assert_input_prompt(&key_a.error, "key: ");

    let value_a = session
        .resume_eval_request_with_input("ka")
        .expect_err("GroupBy should suspend while transforming the first list item");
    assert_input_prompt(&value_a.error, "value: ");

    let key_b = session
        .resume_eval_request_with_input("va")
        .expect_err("GroupBy should continue to the second list item");
    assert_input_prompt(&key_b.error, "key: ");

    let value_b = session
        .resume_eval_request_with_input("kb")
        .expect_err("GroupBy should transform the second list item after keying it");
    assert_input_prompt(&value_b.error, "value: ");

    let response = session
        .resume_eval_request_with_input("vb")
        .expect("GroupBy should preserve completed list groups after resuming");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[\"ka\", List[\"va\"]], Rule[\"kb\", List[\"vb\"]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_nested_groupby_after_multiple_key_suspensions() {
    let mut session = core_session();

    let failure = session
        .eval_request("GroupBy[Association[a :> InputString[\"value: \"]], {Identity, Identity}]")
        .expect_err("nested GroupBy should suspend while keying the first level");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("first")
        .expect_err("nested GroupBy should suspend again while keying the second level");

    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("second")
        .expect("resume should continue nested GroupBy after the second key resolves");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[\"first\", Association[Rule[\"second\", Association[RuleDelayed[a, InputString[\"value: \"]]]]]]]"
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_groupby_nested_and_reducer_group_loops() {
    for (source, expected) in [
        (
            r#"calls = 0;
               {GroupBy[{1, 2}, {Identity, Function[x, calls = calls + 1; If[calls == 1, InputString["group-stage: "], x]]}], calls}"#,
            "List[Association[Rule[1, Association[Rule[\"one\", List[1]]]], Rule[2, Association[Rule[2, List[2]]]]], 2]",
        ),
        (
            r#"calls = 0;
               {GroupBy[{1, 2}, Identity, Function[{group}, calls = calls + 1; If[calls == 1, InputString["group-stage: "], First[group]]]], calls}"#,
            "List[Association[Rule[1, \"one\"], Rule[2, 2]], 2]",
        ),
        (
            r#"calls = 0;
               {GroupBy[Association[a -> 1, b -> 2], {Identity, Function[x, calls = calls + 1; If[calls == 1, InputString["group-stage: "], x]]}], calls}"#,
            "List[Association[Rule[1, Association[Rule[\"one\", Association[Rule[a, 1]]]]], Rule[2, Association[Rule[2, Association[Rule[b, 2]]]]]], 2]",
        ),
        (
            r#"calls = 0;
               {GroupBy[Association[a -> 1, b -> 2], Identity, Function[{group}, calls = calls + 1; If[calls == 1, InputString["group-stage: "], Last[group]]]], calls}"#,
            "List[Association[Rule[1, \"one\"], Rule[2, 2]], 2]",
        ),
    ] {
        let mut session = core_session();
        let failure = session
            .eval_request(source)
            .expect_err("GroupBy phase should suspend on the first group");
        assert_input_prompt(&failure.error, "group-stage: ");

        let response = session
            .resume_eval_request_with_input("one")
            .expect("GroupBy phase should continue through the remaining group");
        assert_eq!(response.value.to_string(), expected);
    }
}

#[test]
fn resume_eval_request_with_input_reports_catenate_invalid_value_after_suspension() {
    let mut session = core_session();

    let failure = session
        .eval_request("Catenate[Association[a :> {InputString[\"value: \"]}, b -> 2]]")
        .expect_err("Catenate should suspend before discovering the later invalid scalar");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should report the invalid scalar after collecting delayed values");

    assert_eq!(
        response.value.to_string(),
        "Catenate[Association[RuleDelayed[a, List[InputString[\"value: \"]]], Rule[b, 2]]]"
    );
    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.messages[0].symbol, "Catenate");
    assert_eq!(response.messages[0].tag, "invrp");
    assert_eq!(
        response.messages[0].text,
        "The argument 2 is not a valid Association or a list."
    );
}

#[test]
fn resume_eval_request_with_input_can_finish_deletemissing_eager_delayed_predicate() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "DeleteMissing[Association[a :> {InputString[\"value: \"], Missing[]}, b -> 2], 1, 1]",
        )
        .expect_err(
            "DeleteMissing should suspend when eager delayed deletion predicate requests input",
        );

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should continue DeleteMissing after the delayed predicate resolves");

    assert_eq!(response.value.to_string(), "Association[Rule[b, 2]]");
}

#[test]
fn resume_eval_request_with_input_can_finish_nested_deletemissing_predicate() {
    let mut session = core_session();

    let failure = session
        .eval_request(
            "DeleteMissing[Association[outer -> Association[a :> {InputString[\"value: \"], Missing[]}, b -> 2]], 2, 1]",
        )
        .expect_err("nested DeleteMissing should suspend when an eager delayed predicate requests input");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "value: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("done")
        .expect("resume should rebuild the outer DeleteMissing result");

    assert_eq!(
        response.value.to_string(),
        "Association[Rule[outer, Association[Rule[b, 2]]]]"
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
