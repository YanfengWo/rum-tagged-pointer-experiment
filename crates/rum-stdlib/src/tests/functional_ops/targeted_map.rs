use super::*;

#[test]
fn mapat_mapthread_and_subsetmap_cover_current_first_pass_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("MapAt[f, {{a, b}, {c, d}}, {All, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, f[b]], List[c, f[d]]]"
    );
    assert_eq!(
        session
            .eval_text("MapAt[f, <|a -> 1, b -> <|c -> 2|>|>, {Key[b], Key[c]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, Association[Rule[c, f[2]]]]]"
    );
    assert_eq!(
        session
            .eval_text("MapThread[f][{{a, b}, {c, d}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, c], f[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("MapThread[f, {{{a, b}, {c, d}}, {{e, f}, {g, h}}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[a, e], f[b, f]], List[f[c, g], f[d, h]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "SubsetMap[Function[List[Last[Slot[1]], First[Slot[1]]]], {a, b, c, d}, {2, 4}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[a, d, c, b]"
    );
}

#[test]
fn targeted_mapping_reports_only_explicit_missing_or_incompatible_paths() {
    let mut session = core_session();

    let missing_map_at = session.eval_text("MapAt[f, {a, b}, 3]").unwrap();
    assert_eq!(missing_map_at.value.to_string(), "List[a, b]");
    assert!(missing_map_at.messages.iter().any(|message| {
        message.symbol == "MapAt"
            && message.tag == "partw"
            && message.text == "Part 3 of List[a, b] does not exist."
    }));

    let all_map_at = session.eval_text("MapAt[f, {a, b}, {All, 3}]").unwrap();
    assert_eq!(all_map_at.value.to_string(), "List[a, b]");
    assert!(all_map_at.messages.is_empty());

    let successful_map_at = session.eval_text("MapAt[f, {a, b}, 1]").unwrap();
    assert_eq!(successful_map_at.value.to_string(), "List[f[a], b]");
    assert!(successful_map_at.messages.is_empty());

    let duplicate_subset = session
        .eval_text("SubsetMap[Reverse, {a, b}, {{1}, {1}}]")
        .unwrap();
    assert_eq!(duplicate_subset.value.to_string(), "List[a, b]");
    assert!(duplicate_subset.messages.iter().any(|message| {
        message.symbol == "SubsetMap"
            && message.tag == "path"
            && message.text == "Duplicate subset path List[1] is not allowed."
    }));

    let overlapping_subset = session
        .eval_text("SubsetMap[Reverse, {a, b}, {{1}, {1, 1}}]")
        .unwrap();
    assert_eq!(overlapping_subset.value.to_string(), "List[a, b]");
    assert!(overlapping_subset.messages.iter().any(|message| {
        message.symbol == "SubsetMap"
            && message.tag == "path"
            && message.text == "Overlapping subset paths List[1] and List[1, 1] are not allowed."
    }));

    let missing_subset = session
        .eval_text("SubsetMap[Reverse, {a, b}, {{3}}]")
        .unwrap();
    assert_eq!(missing_subset.value.to_string(), "List[a, b]");
    assert!(missing_subset.messages.iter().any(|message| {
        message.symbol == "SubsetMap"
            && message.tag == "path"
            && message.text.contains("part 3 of List[a, b] does not exist")
    }));

    let successful_subset = session
        .eval_text("SubsetMap[Reverse, {a, b, c}, {{1}, {3}}]")
        .unwrap();
    assert_eq!(successful_subset.value.to_string(), "List[c, b, a]");
    assert!(successful_subset.messages.is_empty());
}

#[test]
fn targeted_mapping_builtins_report_current_argument_forms() {
    let mut session = core_session();

    let map_at = session.eval_text("MapAt[f, expr, bad]").unwrap();
    assert_eq!(map_at.value.to_string(), "MapAt[f, expr, bad]");
    assert!(
        map_at
            .messages
            .iter()
            .any(|message| message.symbol == "MapAt" && message.tag == "argform")
    );

    let map_thread = session.eval_text("MapThread[f, expr, 0]").unwrap();
    assert_eq!(map_thread.value.to_string(), "MapThread[f, expr, 0]");
    assert!(
        map_thread
            .messages
            .iter()
            .any(|message| message.symbol == "MapThread" && message.tag == "argform")
    );

    let subset_map = session.eval_text("SubsetMap[f, expr, bad]").unwrap();
    assert_eq!(subset_map.value.to_string(), "SubsetMap[f, expr, bad]");
    assert!(
        subset_map
            .messages
            .iter()
            .any(|message| message.symbol == "SubsetMap" && message.tag == "argform")
    );
}
