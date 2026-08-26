use super::*;
use rum_runtime::SessionRequestError;

fn assert_eval(
    session: &mut Session,
    source: &str,
    expected_value: &str,
    expected_messages: &[(&str, &str, &str)],
) {
    let outcome = session.eval_text(source).unwrap();
    assert_eq!(outcome.value.to_string(), expected_value, "{source}");
    let actual_messages = outcome
        .messages
        .iter()
        .map(|message| {
            (
                message.symbol.as_str(),
                message.tag.as_str(),
                message.text.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_messages, expected_messages, "{source}");
}

#[test]
fn association_part_and_extract_value_message_matrix_is_exact() {
    let mut session = core_session();

    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> 2], System`Key[b]]",
        "2",
        &[],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1], System`Key[b]]",
        "Missing[\"KeyAbsent\", Key[b]]",
        &[],
    );
    assert_eval(
        &mut session,
        "Extract[Association[a -> 1], {System`Key[b]}]",
        "Missing[\"KeyAbsent\", Key[b]]",
        &[(
            "Extract",
            "keyw",
            "Key b does not exist in Association[Rule[a, 1]].",
        )],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> 2], 1]",
        "1",
        &[],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> 2], -1]",
        "2",
        &[],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> 2], 3]",
        "Part[Association[Rule[a, 1], Rule[b, 2]], 3]",
        &[(
            "Part",
            "partw",
            "Part 3 of Association[Rule[a, 1], Rule[b, 2]] does not exist.",
        )],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> 2], All]",
        "Association[Rule[a, 1], Rule[b, 2]]",
        &[],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> 2, c -> 3], 2 ;; -1]",
        "Association[Rule[b, 2], Rule[c, 3]]",
        &[],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> 2], {2, 1}]",
        "Association[Rule[b, 2], Rule[a, 1]]",
        &[],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> 2], {System`Key[a], 2}]",
        "Part[Association[Rule[a, 1], Rule[b, 2]], List[Key[a], 2]]",
        &[(
            "Part",
            "pmix",
            "Integer and Key specifications cannot be mixed in the same part specification.",
        )],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1, b -> Association[c -> 3]], System`Key[b], System`Key[c]]",
        "3",
        &[],
    );
    assert_eval(
        &mut session,
        "Part[Association[a -> 1], All, 1]",
        "Part[Association[Rule[a, 1]], All, 1]",
        &[(
            "Part",
            "partd",
            "Part specification Part[Association[Rule[a, 1]], All, 1] is longer than depth of object.",
        )],
    );
    assert_eval(
        &mut session,
        "Extract[Association[a -> 1, b -> 2], {3}]",
        "Extract[Association[Rule[a, 1], Rule[b, 2]], List[3]]",
        &[(
            "Extract",
            "partw",
            "Part 3 of Association[Rule[a, 1], Rule[b, 2]] does not exist.",
        )],
    );
    assert_eval(
        &mut session,
        "Extract[Association[a -> 1], {All, 1}]",
        "Extract[Association[Rule[a, 1]], List[All, 1]]",
        &[(
            "Extract",
            "partd",
            "Part specification List[All, 1] is longer than depth of object.",
        )],
    );
    assert_eval(
        &mut session,
        "Extract[Association[a -> 1], {{System`Key[x]}, {System`Key[y]}}]",
        "List[Missing[\"KeyAbsent\", Key[x]], Missing[\"KeyAbsent\", Key[y]]]",
        &[
            (
                "Extract",
                "keyw",
                "Key x does not exist in Association[Rule[a, 1]].",
            ),
            (
                "Extract",
                "keyw",
                "Key y does not exist in Association[Rule[a, 1]].",
            ),
        ],
    );
}

#[test]
fn association_key_selector_suspension_resumes_the_original_part_call() {
    let mut session = core_session();
    let failure = session
        .eval_request("Part[Association[\"a\" -> 7], System`Key[InputString[\"key: \"]]]")
        .expect_err("InputString should suspend the selector evaluation");

    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "key: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("a")
        .expect("resumption should finish the original Part call");
    assert_eq!(response.value.to_string(), "7");
    assert!(response.messages.is_empty());
}
