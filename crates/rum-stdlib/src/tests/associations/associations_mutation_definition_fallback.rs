use super::*;
use rum_runtime::SessionRequestError;

#[test]
fn definition_fallback_prepares_every_discovered_shape_for_both_mutation_heads() {
    struct Case {
        name: &'static str,
        setup: &'static str,
        associate_target: &'static str,
        drop_target: &'static str,
        associate_lookup: &'static str,
        drop_lookup: &'static str,
        catalog: &'static str,
    }

    let cases = [
        Case {
            name: "DownValue",
            setup: "i = 2; at[2] = <|a -> 1, drop -> 2|>; dt[2] = <|a -> 1, drop -> 2|>",
            associate_target: "at[i]",
            drop_target: "dt[i]",
            associate_lookup: "at[2]",
            drop_lookup: "dt[2]",
            catalog: "DownValues",
        },
        Case {
            name: "SubValue",
            setup: "p = 1; q = 2; at[1][2] = <|a -> 1, drop -> 2|>; dt[1][2] = <|a -> 1, drop -> 2|>",
            associate_target: "at[p][q]",
            drop_target: "dt[p][q]",
            associate_lookup: "at[1][2]",
            drop_lookup: "dt[1][2]",
            catalog: "SubValues",
        },
        Case {
            name: "evaluated head alias",
            setup: "i = 2; aAlias = at; dAlias = dt; at[2] = <|a -> 1, drop -> 2|>; dt[2] = <|a -> 1, drop -> 2|>",
            associate_target: "aAlias[i]",
            drop_target: "dAlias[i]",
            associate_lookup: "at[2]",
            drop_lookup: "dt[2]",
            catalog: "DownValues",
        },
        Case {
            name: "delayed head alias",
            setup: "i = 2; aAlias := at; dAlias := dt; at[2] = <|a -> 1, drop -> 2|>; dt[2] = <|a -> 1, drop -> 2|>",
            associate_target: "aAlias[i]",
            drop_target: "dAlias[i]",
            associate_lookup: "at[2]",
            drop_lookup: "dt[2]",
            catalog: "DownValues",
        },
        Case {
            name: "delayed index",
            setup: "iDelayed := 2; at[2] = <|a -> 1, drop -> 2|>; dt[2] = <|a -> 1, drop -> 2|>",
            associate_target: "at[iDelayed]",
            drop_target: "dt[iDelayed]",
            associate_lookup: "at[2]",
            drop_lookup: "dt[2]",
            catalog: "DownValues",
        },
        Case {
            name: "held target",
            setup: "SetAttributes[at, HoldFirst]; SetAttributes[dt, HoldFirst]; i = 2; at[i] = <|a -> 1, drop -> 2|>; dt[i] = <|a -> 1, drop -> 2|>",
            associate_target: "at[i]",
            drop_target: "dt[i]",
            associate_lookup: "at[i]",
            drop_lookup: "dt[i]",
            catalog: "DownValues",
        },
    ];

    for case in cases {
        let mut session = core_session();
        let outcome = session
            .eval_text(&format!(
                "{}; ar = AssociateTo[{}, k -> 7]; dr = KeyDropFrom[{}, drop]; \
                 {{ar === <|a -> 1, drop -> 2, k -> 7|>, \
                   {} === <|a -> 1, drop -> 2, k -> 7|>, \
                   Length[{}[at]] === 1, \
                   dr === <|a -> 1|>, {} === <|a -> 1|>, Length[{}[dt]] === 1}}",
                case.setup,
                case.associate_target,
                case.drop_target,
                case.associate_lookup,
                case.catalog,
                case.drop_lookup,
                case.catalog,
            ))
            .unwrap();

        assert_eq!(
            outcome.value.to_string(),
            "List[True, True, True, True, True, True]",
            "{} should use the prepared fallback for both entry points",
            case.name
        );
        assert!(
            outcome.messages.is_empty(),
            "{}: {:?}",
            case.name,
            outcome.messages
        );
    }
}

#[test]
fn definition_fallback_curried_operator_forms_store_the_prepared_identity() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "i = 2; at[2] = <|a -> 1|>; dt[2] = <|a -> 1, drop -> 2|>; \
             ar = AssociateTo[k -> 7][at[i]]; dr = KeyDropFrom[drop][dt[i]]; \
             {ar === at[2], dr === dt[2], Length[DownValues[at]] === 1, \
              Length[DownValues[dt]] === 1}",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[True, True, True, True]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn definition_fallback_reads_index_and_outer_definition_exactly_once() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "aIndexCount = 0; aReadCount = 0; \
             aIndex := (aIndexCount = aIndexCount + 1; 2); \
             at[2] := (aReadCount = aReadCount + 1; <|a -> 1|>); \
             ar = AssociateTo[at[aIndex], k -> 7]; \
             dIndexCount = 0; dReadCount = 0; \
             dIndex := (dIndexCount = dIndexCount + 1; 2); \
             dt[2] := (dReadCount = dReadCount + 1; <|a -> 1, drop -> 2|>); \
             dr = KeyDropFrom[dt[dIndex], drop]; \
             {ar === at[2], aIndexCount, aReadCount, Length[DownValues[at]], \
              dr === dt[2], dIndexCount, dReadCount, Length[DownValues[dt]]}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[True, 1, 1, 1, True, 1, 1, 1]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn definition_fallback_uses_canonical_owner_for_head_specific_protection_checks() {
    let mut associate_session = core_session();
    associate_session
        .eval_text(
            "i = 2; readCount = 0; alias = owner; \
             owner[2] := (readCount = readCount + 1; <|a -> 1|>); Protect[owner]",
        )
        .unwrap();
    let associate = associate_session
        .eval_text("AssociateTo[alias[i], k -> 7]")
        .unwrap();
    assert_eq!(
        associate.value.to_string(),
        "AssociateTo[alias[i], Rule[k, 7]]"
    );
    assert!(
        associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    assert_eq!(
        associate_session
            .eval_text("{readCount, Length[DownValues[owner]]}")
            .unwrap()
            .value
            .to_string(),
        "List[0, 1]"
    );

    let mut drop_session = core_session();
    drop_session
        .eval_text(
            "i = 2; readCount = 0; alias = owner; \
             owner[2] := (readCount = readCount + 1; <|a -> 1, drop -> 2|>); Protect[owner]",
        )
        .unwrap();
    let drop = drop_session
        .eval_text("KeyDropFrom[alias[i], drop]")
        .unwrap();
    assert_eq!(drop.value.to_string(), "Association[Rule[a, 1]]");
    assert!(
        drop.messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "write")
    );
    assert_eq!(
        drop_session
            .eval_text("{readCount, Length[DownValues[owner]]}")
            .unwrap()
            .value
            .to_string(),
        "List[1, 1]"
    );
}

#[test]
fn definition_fallback_after_attribute_invalidation_addresses_the_new_held_identity() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "i = 2; at[2] = <|old -> 1|>; dt[2] = <|old -> 1|>; \
             SetAttributes[at, HoldFirst]; SetAttributes[dt, HoldFirst]; \
             at[i] = <|fresh -> 2|>; dt[i] = <|fresh -> 2, drop -> 3|>; \
             AssociateTo[at[i], k -> 7]; KeyDropFrom[dt[i], drop]; \
             {at[2] === <|old -> 1|>, at[i] === <|fresh -> 2, k -> 7|>, \
              Length[DownValues[at]] === 2, dt[2] === <|old -> 1|>, \
              dt[i] === <|fresh -> 2|>, Length[DownValues[dt]] === 2}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[True, True, True, True, True, True]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn definition_fallback_after_index_rebind_and_clear_preserves_older_snapshots() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "i = 2; at[2] = <|old -> 2|>; dt[2] = <|old -> 2, drop -> 2|>; \
             i = 3; at[3] = <|fresh -> 3|>; dt[3] = <|fresh -> 3, drop -> 3|>; \
             AssociateTo[at[i], rebound -> 7]; KeyDropFrom[dt[i], drop]; \
             Clear[i]; at[i] = <|raw -> 4|>; dt[i] = <|raw -> 4, drop -> 4|>; \
             AssociateTo[at[i], cleared -> 8]; KeyDropFrom[dt[i], drop]; \
             {at[2] === <|old -> 2|>, at[3] === <|fresh -> 3, rebound -> 7|>, \
              at[i] === <|raw -> 4, cleared -> 8|>, Length[DownValues[at]] === 3, \
              dt[2] === <|old -> 2, drop -> 2|>, dt[3] === <|fresh -> 3|>, \
              dt[i] === <|raw -> 4|>, Length[DownValues[dt]] === 3}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[True, True, True, True, True, True, True, True]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn definition_fallback_after_owner_clear_uses_only_the_reinstalled_identity() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "i = 2; at[2] = <|old -> 1|>; dt[2] = <|old -> 1, drop -> 1|>; \
             Clear[at]; Clear[dt]; \
             at[2] = <|fresh -> 2|>; dt[2] = <|fresh -> 2, drop -> 2|>; \
             AssociateTo[at[i], k -> 7]; KeyDropFrom[dt[i], drop]; \
             {at[2] === <|fresh -> 2, k -> 7|>, Length[DownValues[at]] === 1, \
              dt[2] === <|fresh -> 2|>, Length[DownValues[dt]] === 1}",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "List[True, True, True, True]");
    assert!(outcome.messages.is_empty());
}

#[test]
fn symbol_and_part_mutation_fast_paths_remain_separate_from_definition_preparation() {
    let mut session = core_session();
    let outcome = session
        .eval_text(
            "a = <|x -> 1, drop -> 2|>; parts = {<|x -> 1, drop -> 2|>}; \
             AssociateTo[a, k -> 7]; KeyDropFrom[a, drop]; \
             AssociateTo[parts[[1]], k -> 7]; KeyDropFrom[parts[[1]], drop]; \
             {a, parts}",
        )
        .unwrap();

    assert_eq!(
        outcome.value.to_string(),
        "List[Association[Rule[x, 1], Rule[k, 7]], List[Association[Rule[x, 1], Rule[k, 7]]]]"
    );
    assert!(outcome.messages.is_empty());
}

#[test]
fn definition_fallback_resumes_preparation_and_outer_read_without_replay_for_both_heads() {
    for (name, source, first_prompt, second_prompt, expected) in [
        (
            "AssociateTo",
            "prepareCount = 0; readCount = 0; \
             at[2] := (readCount = readCount + 1; InputString[\"associate read: \"]; <|a -> 1|>); \
             result = AssociateTo[at[(prepareCount = prepareCount + 1; \
                 InputString[\"associate prepare: \"]; 2)], k -> 7]; \
             {result, at[2], prepareCount, readCount, Length[DownValues[at]]}",
            "associate prepare: ",
            "associate read: ",
            "List[Association[Rule[a, 1], Rule[k, 7]], Association[Rule[a, 1], Rule[k, 7]], 1, 1, 1]",
        ),
        (
            "KeyDropFrom",
            "prepareCount = 0; readCount = 0; \
             dt[2] := (readCount = readCount + 1; InputString[\"drop read: \"]; <|a -> 1, drop -> 2|>); \
             result = KeyDropFrom[dt[(prepareCount = prepareCount + 1; \
                 InputString[\"drop prepare: \"]; 2)], drop]; \
             {result, dt[2], prepareCount, readCount, Length[DownValues[dt]]}",
            "drop prepare: ",
            "drop read: ",
            "List[Association[Rule[a, 1]], Association[Rule[a, 1]], 1, 1, 1]",
        ),
    ] {
        let mut session = core_session();
        let first = session
            .eval_request(source)
            .expect_err("target preparation should request input");
        assert_input_prompt(&first.error, first_prompt, name);

        let second = session
            .resume_eval_request_with_input("prepared")
            .expect_err("the prepared outer definition read should request input");
        assert_input_prompt(&second.error, second_prompt, name);

        let completed = session
            .resume_eval_request_with_input("read")
            .expect("the mutation should finish after the outer read resumes");
        assert_eq!(completed.value.to_string(), expected, "{name}");
        assert!(
            completed.messages.is_empty(),
            "{name}: {:?}",
            completed.messages
        );
    }
}

fn assert_input_prompt(error: &SessionRequestError, expected: &str, name: &str) {
    match error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, expected, "{name}");
        }
        other => panic!("{name}: expected input request, got {other:?}"),
    }
}
