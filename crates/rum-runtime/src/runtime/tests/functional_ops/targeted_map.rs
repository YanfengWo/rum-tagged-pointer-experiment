use super::*;

#[test]
fn mapat_mapthread_and_subsetmap_cover_current_first_pass_subset() {
    let mut session = Session::new();

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
fn targeted_mapping_builtins_report_current_argument_forms() {
    let mut session = Session::new();

    let map_at = session.eval_text("MapAt[f, expr, bad]").unwrap();
    assert_eq!(map_at.value.to_string(), "MapAt[f, expr, bad]");
    assert!(map_at
        .messages
        .iter()
        .any(|message| message.symbol == "MapAt" && message.tag == "argform"));

    let map_thread = session.eval_text("MapThread[f, expr, 0]").unwrap();
    assert_eq!(map_thread.value.to_string(), "MapThread[f, expr, 0]");
    assert!(map_thread
        .messages
        .iter()
        .any(|message| message.symbol == "MapThread" && message.tag == "argform"));

    let subset_map = session.eval_text("SubsetMap[f, expr, bad]").unwrap();
    assert_eq!(subset_map.value.to_string(), "SubsetMap[f, expr, bad]");
    assert!(subset_map
        .messages
        .iter()
        .any(|message| message.symbol == "SubsetMap" && message.tag == "argform"));
}
