use super::*;

#[test]
fn part_supports_exact_indices_nested_paths_and_head_access() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 2]")
            .unwrap()
            .value
            .to_string(),
        "b"
    );
    assert_eq!(
        session
            .eval_text("Part[f[g[a], h[b]], 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "b"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {1, -1}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 0]")
            .unwrap()
            .value
            .to_string(),
        "f"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], {}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[g[a, b], h[c, d]], All, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c, d], Span[2, All, 2]]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {All}]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], List[All]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {1 ;; 2}]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], List[Span[1, 2]]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {Span[1, All, 2], -1}]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], List[Span[1, All, 2], -1]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 0 ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], {0, 1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[f, a, b]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 1 ;; 0]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 2 ;; 1]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 0 ;; 0]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b], Span[0, 0]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], All ;; 1 ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[a]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 2 ;; All ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[b]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], All ;; All ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], Span[All, All, -1]]"
    );
    assert_eq!(
        session.eval_text("Part[1, All]").unwrap().value.to_string(),
        "Integer[]"
    );
    assert_eq!(
        session
            .eval_text("Part[\"a\", All]")
            .unwrap()
            .value
            .to_string(),
        "String[]"
    );
    assert_eq!(
        session.eval_text("Part[x, All]").unwrap().value.to_string(),
        "Symbol[]"
    );
    assert_eq!(
        session
            .eval_text("Part[x, 0 ;; 0]")
            .unwrap()
            .value
            .to_string(),
        "Symbol[]"
    );
    assert_eq!(
        session
            .eval_text("Part[1, 1 ;; All]")
            .unwrap()
            .value
            .to_string(),
        "Integer[]"
    );
    assert_eq!(
        session
            .eval_text("f[g[a, b], h[c, d]][[All, 2 ;; ;; 2]]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {{1, 3}, {2}}]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], List[List[1, 3], List[2]]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 1 ;; UpTo[5]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], UpTo[2] ;; All]")
            .unwrap()
            .value
            .to_string(),
        "f[b, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 3 ;; UpTo[1] ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[c, b, a]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 1 ;; UpTo[Infinity] ;; 2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], UpTo[2]]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], UpTo[2]]"
    );
}

#[test]
fn keyed_part_and_extract_cover_association_paths() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[Association[a -> 1, b -> Association[c -> 3]], Key[b], Key[c]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> 1, b -> 2], {Key[b], Key[a]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2], Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> 1], Key[b]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", Key[b]]"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> Association[b -> 2]], {Key[a]}, Key[b]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> 1, b -> 2], 1]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> 1, b -> 2], -1]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> 1, b -> 2], All]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> f[x], b -> f[y]], All, 1]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, x], Rule[b, y]]"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> 1, b -> 2], 1 ;; 2]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Part[Association[a -> 1, b -> 2], {2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2], Rule[a, 1]]"
    );
    let mixed = session
        .eval_text("Part[Association[a -> 1, b -> 2], {Key[a], 2}]")
        .unwrap();
    assert_eq!(
        mixed.value.to_string(),
        "Part[Association[Rule[a, 1], Rule[b, 2]], List[Key[a], 2]]"
    );
    assert!(
        mixed
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "pmix")
    );
    assert_eq!(
        session
            .eval_text("Extract[Association[a -> 1, b -> Association[c -> 3]], {Key[b], Key[c]}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Extract[Association[a -> 1, b -> Association[c -> 3]], {{Key[b], Key[c]}, {Key[a]}}]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 1]"
    );
    let missing_key_extract = session
        .eval_text("Extract[Association[a -> 1], {Key[b]}]")
        .unwrap();
    assert_eq!(
        missing_key_extract.value.to_string(),
        "Missing[\"KeyAbsent\", Key[b]]"
    );
    assert!(
        missing_key_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "keyw")
    );
    let mixed_missing_key_extract = session
        .eval_text("Extract[Association[a -> 1], {{Key[b]}, {Key[a]}}]")
        .unwrap();
    assert_eq!(
        mixed_missing_key_extract.value.to_string(),
        "List[Missing[\"KeyAbsent\", Key[b]], 1]"
    );
    assert!(
        mixed_missing_key_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "keyw")
    );
    assert_eq!(
        session
            .eval_text("Extract[Association[a -> 1, b -> 2], {{1}, {2}}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Extract[Association[a -> f[x], b -> f[y]], {All, 1}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, x], Rule[b, y]]"
    );
}

#[test]
fn extract_consumes_exact_paths_and_position_output() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Extract[f[g[a], b, h[c]], {1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "a"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a], b, h[c]], Position[f[g[a], b, h[c]], _Symbol, Infinity, Heads -> False]]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {{1}, {2}}, List]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[b]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a, b], h[c, d]], {All, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a, b], h[c, d]], {{All, 2}, {1, Span[1, All]}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[b, d], g[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {Span[0, -1]}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {{}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {{{}}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {{{1}}, {{2}}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a], f[b]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a, b], h[c, d]], {{{1}, {2}}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g[b]]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[1, {All}]")
            .unwrap()
            .value
            .to_string(),
        "Integer[]"
    );
    assert_eq!(
        session
            .eval_text("Extract[1, {{All}, {0 ;; 0}}]")
            .unwrap()
            .value
            .to_string(),
        "List[Integer[], Integer[]]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b, c], {1 ;; UpTo[2]}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b, c], {{1 ;; UpTo[2]}, {UpTo[2] ;; All}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b], f[b, c]]"
    );
}

#[test]
fn invalid_part_and_extract_specs_report_selector_messages() {
    let mut session = Session::new();

    let invalid_part = session.eval_text("Part[f[a, b, c], {1 ;; 2}]").unwrap();
    assert_eq!(
        invalid_part.value.to_string(),
        "Part[f[a, b, c], List[Span[1, 2]]]"
    );
    assert!(
        invalid_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "pkspec1")
    );

    let invalid_extract = session.eval_text("Extract[f[a], foo]").unwrap();
    assert_eq!(invalid_extract.value.to_string(), "Extract[f[a], foo]");
    assert!(
        invalid_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "psl1")
    );

    let missing_part = session.eval_text("Part[f[a], 2]").unwrap();
    assert_eq!(missing_part.value.to_string(), "Part[f[a], 2]");
    assert!(
        missing_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "partw")
    );

    let too_deep_part = session.eval_text("Part[f[a], All, 2]").unwrap();
    assert_eq!(too_deep_part.value.to_string(), "Part[f[a], All, 2]");
    assert!(
        too_deep_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "partd")
    );

    let bad_take_part = session.eval_text("Part[f[a], 1 ;; 5]").unwrap();
    assert_eq!(bad_take_part.value.to_string(), "Part[f[a], Span[1, 5]]");
    assert!(
        bad_take_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "take")
    );

    let bad_key_part = session.eval_text("Part[f[a], Key[x]]").unwrap();
    assert_eq!(bad_key_part.value.to_string(), "Part[f[a], Key[x]]");
    assert!(
        bad_key_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "pspec1")
    );

    let missing_extract = session.eval_text("Extract[f[a], {{1}, {2}}]").unwrap();
    assert_eq!(
        missing_extract.value.to_string(),
        "Extract[f[a], List[List[1], List[2]]]"
    );
    assert!(
        missing_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "partw")
    );

    let too_deep_extract = session.eval_text("Extract[f[a], {All, 2}]").unwrap();
    assert_eq!(
        too_deep_extract.value.to_string(),
        "Extract[f[a], List[All, 2]]"
    );
    assert!(
        too_deep_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "partd")
    );

    let bad_take_extract = session.eval_text("Extract[f[a], {1 ;; 2}]").unwrap();
    assert_eq!(
        bad_take_extract.value.to_string(),
        "Extract[f[a], List[Span[1, 2]]]"
    );
    assert!(
        bad_take_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "take")
    );

    let bad_key_extract = session.eval_text("Extract[f[a], {Key[x]}]").unwrap();
    assert_eq!(
        bad_key_extract.value.to_string(),
        "Extract[f[a], List[Key[x]]]"
    );
    assert!(
        bad_key_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "pspec1")
    );
}
