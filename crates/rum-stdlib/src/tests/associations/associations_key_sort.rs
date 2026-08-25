use super::*;
use rum_runtime::SessionRequestError;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn custom_key_sorts_preserve_entry_forms_ordering_and_stable_ties() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"x = 0;
               result = KeySort[
                 Association[aa :> (x = x + 1), b -> 2, cc -> 3, d :> (x = x + 10)],
                 Function[{left, right}, Less[StringLength[ToString[left]], StringLength[ToString[right]]]]
               ];
               {result, x}"#,
        ),
        "List[Association[Rule[b, 2], RuleDelayed[d, Set[x, Plus[x, 10]]], RuleDelayed[aa, Set[x, Plus[x, 1]]], Rule[cc, 3]], 0]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"x = 0;
               result = KeySort[
                 List[bb :> (x = x + 1), a -> 2, cc -> 3],
                 Function[{left, right}, Less[StringLength[ToString[left]], StringLength[ToString[right]]]]
               ];
               {result, x}"#,
        ),
        "List[Association[Rule[a, 2], RuleDelayed[bb, Set[x, Plus[x, 1]]], Rule[cc, 3]], 0]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "KeySort[Association[1 -> a, 3 -> c, 2 -> b], Greater]",
        ),
        "Association[Rule[3, c], Rule[2, b], Rule[1, a]]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            r#"x = 0;
               result = KeySortBy[
                 List[aa :> (x = x + 1), b -> 2, cc -> 3, d :> (x = x + 10)],
                 Function[key, StringLength[ToString[key]]],
                 Less
               ];
               {result, x}"#,
        ),
        "List[Association[Rule[b, 2], RuleDelayed[d, Set[x, Plus[x, 10]]], RuleDelayed[aa, Set[x, Plus[x, 1]]], Rule[cc, 3]], 0]"
    );
}

#[test]
fn custom_key_sorts_preserve_the_full_comparator_result_policy() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"calls = 0;
               input = Association[b -> 2, a -> 1];
               {
                 Keys[KeySort[input, Function[{x, y}, calls = calls + 1; 1]]],
                 Keys[KeySort[input, Function[{x, y}, calls = calls + 1; 0]]],
                 Keys[KeySort[input, Function[{x, y}, calls = calls + 1; -1]]],
                 Keys[KeySort[input, Function[{x, y}, calls = calls + 1; True]]],
                 Keys[KeySort[input, Function[{x, y}, calls = calls + 1; False]]],
                 Keys[KeySort[input, Function[{x, y}, calls = calls + 1; other]]],
                 Keys[KeySortBy[input, Identity, Function[{x, y}, calls = calls + 1; 1]]],
                 Keys[KeySortBy[input, Identity, Function[{x, y}, calls = calls + 1; 0]]],
                 Keys[KeySortBy[input, Identity, Function[{x, y}, calls = calls + 1; -1]]],
                 Keys[KeySortBy[input, Identity, Function[{x, y}, calls = calls + 1; True]]],
                 Keys[KeySortBy[input, Identity, Function[{x, y}, calls = calls + 1; False]]],
                 Keys[KeySortBy[input, Identity, Function[{x, y}, calls = calls + 1; other]]],
                 calls
               }"#,
        ),
        "List[List[a, b], List[b, a], List[b, a], List[a, b], List[b, a], List[a, b], List[a, b], List[b, a], List[b, a], List[a, b], List[b, a], List[a, b], 12]"
    );
}

#[test]
fn custom_key_sort_by_decorates_once_in_input_order() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"trace = {};
               calls = 0;
               result = KeySortBy[
                 Association[3 -> c, 1 -> a, 2 -> b],
                 Function[key, calls = calls + 1; trace = Append[trace, key]; key],
                 Less
               ];
               {Keys[result], trace, calls}"#,
        ),
        "List[List[1, 2, 3], List[3, 1, 2], 3]"
    );
}

#[test]
fn custom_key_sorts_obey_the_merge_bound_without_growing_the_stack() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            r#"n = 1001;
               input = AssociationThread[Range[n] -> Range[n]];
               calls = 0;
               ordering = Function[{left, right}, calls = calls + 1; Less[left, right]];
               sorted = KeySort[input, ordering];
               keySortCalls = calls;
               calls = 0;
               keyCalls = 0;
               sortedBy = KeySortBy[
                 input,
                 Function[key, keyCalls = keyCalls + 1; key],
                 ordering
               ];
               keySortByCalls = calls;
               SameQ[{Keys[sorted], Keys[sortedBy]}, {Range[n], Range[n]}] &&
                 Length[sorted] == n && Length[sortedBy] == n && keyCalls == n &&
                 keySortCalls > 0 && keySortCalls <= n * 10 &&
                 keySortByCalls > 0 && keySortByCalls <= n * 10"#,
        ),
        "True"
    );
}

#[test]
fn every_custom_key_sort_comparison_resumes_with_the_uninterrupted_trace() {
    for (uninterrupted_source, suspended_source) in [
        (
            r#"comparisonTrace = {}; comparisonCalls = 0;
               result = KeySort[
                 Association[4 -> d, 1 -> a, 3 -> c, 2 -> b, 5 -> e],
                 Function[{left, right}, comparisonCalls = comparisonCalls + 1; comparisonTrace = Append[comparisonTrace, {left, right}]; Less[left, right]]
               ];
               {result, comparisonTrace, comparisonCalls}"#,
            r#"comparisonTrace = {}; comparisonCalls = 0;
               result = KeySort[
                 Association[4 -> d, 1 -> a, 3 -> c, 2 -> b, 5 -> e],
                 Function[{left, right}, comparisonCalls = comparisonCalls + 1; comparisonTrace = Append[comparisonTrace, {left, right}]; InputString["association-compare: "]; Less[left, right]]
               ];
               {result, comparisonTrace, comparisonCalls}"#,
        ),
        (
            r#"keyTrace = {}; keyCalls = 0; comparisonTrace = {}; comparisonCalls = 0;
               result = KeySortBy[
                 Association[4 -> d, 1 -> a, 3 -> c, 2 -> b, 5 -> e],
                 Function[key, keyCalls = keyCalls + 1; keyTrace = Append[keyTrace, key]; key],
                 Function[{left, right}, comparisonCalls = comparisonCalls + 1; comparisonTrace = Append[comparisonTrace, {left, right}]; Less[left, right]]
               ];
               {result, keyTrace, comparisonTrace, keyCalls, comparisonCalls}"#,
            r#"keyTrace = {}; keyCalls = 0; comparisonTrace = {}; comparisonCalls = 0;
               result = KeySortBy[
                 Association[4 -> d, 1 -> a, 3 -> c, 2 -> b, 5 -> e],
                 Function[key, keyCalls = keyCalls + 1; keyTrace = Append[keyTrace, key]; key],
                 Function[{left, right}, comparisonCalls = comparisonCalls + 1; comparisonTrace = Append[comparisonTrace, {left, right}]; InputString["association-compare: "]; Less[left, right]]
               ];
               {result, keyTrace, comparisonTrace, keyCalls, comparisonCalls}"#,
        ),
    ] {
        let mut uninterrupted_session = core_session();
        let uninterrupted = uninterrupted_session
            .eval_text(uninterrupted_source)
            .expect("uninterrupted custom Association sort should finish");
        let expected_comparisons = uninterrupted_session
            .eval_text("comparisonCalls")
            .expect("uninterrupted sort should retain its comparison count")
            .value
            .to_string()
            .parse::<usize>()
            .expect("comparison count should be an exact non-negative integer");
        assert!(expected_comparisons > 0);
        assert!(expected_comparisons <= 5 * 3);

        let mut session = core_session();
        let mut outcome = session.eval_request(suspended_source);
        let mut suspensions = 0;

        let response = loop {
            match outcome {
                Ok(response) => break response,
                Err(failure) => {
                    assert_input_prompt(&failure.error, "association-compare: ");
                    suspensions += 1;
                    assert!(suspensions <= 5 * 3);
                    outcome = session.resume_eval_request_with_input("ignored");
                }
            }
        };

        assert_eq!(response.value, uninterrupted.value);
        assert_eq!(suspensions, expected_comparisons);
        assert!(session.pending_input_request().is_none());
    }
}

#[test]
fn every_custom_key_sort_by_key_resumes_with_the_uninterrupted_trace() {
    let uninterrupted_source = r#"keyTrace = {}; keyCalls = 0; comparisonTrace = {}; comparisonCalls = 0;
        result = KeySortBy[
          Association[4 -> d, 1 -> a, 3 -> c, 2 -> b, 5 -> e],
          Function[key, keyCalls = keyCalls + 1; keyTrace = Append[keyTrace, key]; key],
          Function[{left, right}, comparisonCalls = comparisonCalls + 1; comparisonTrace = Append[comparisonTrace, {left, right}]; Less[left, right]]
        ];
        {result, keyTrace, comparisonTrace, keyCalls, comparisonCalls}"#;
    let suspended_source = r#"keyTrace = {}; keyCalls = 0; comparisonTrace = {}; comparisonCalls = 0;
        result = KeySortBy[
          Association[4 -> d, 1 -> a, 3 -> c, 2 -> b, 5 -> e],
          Function[key, keyCalls = keyCalls + 1; keyTrace = Append[keyTrace, key]; StringLength[InputString["association-key: "]]],
          Function[{left, right}, comparisonCalls = comparisonCalls + 1; comparisonTrace = Append[comparisonTrace, {left, right}]; Less[left, right]]
        ];
        {result, keyTrace, comparisonTrace, keyCalls, comparisonCalls}"#;

    let mut uninterrupted_session = core_session();
    let uninterrupted = uninterrupted_session
        .eval_text(uninterrupted_source)
        .expect("uninterrupted custom KeySortBy should finish");

    let mut session = core_session();
    let mut outcome = session.eval_request(suspended_source);
    for input in ["xxxx", "x", "xxx", "xx", "xxxxx"] {
        let failure = outcome.expect_err("each custom KeySortBy key should request input");
        assert_input_prompt(&failure.error, "association-key: ");
        outcome = session.resume_eval_request_with_input(input);
    }
    let response = outcome.expect("custom KeySortBy should finish after every key resumes");

    assert_eq!(response.value, uninterrupted.value);
    assert!(session.pending_input_request().is_none());
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected);
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }
}
