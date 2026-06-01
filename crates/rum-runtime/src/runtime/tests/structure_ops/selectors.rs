use super::*;

mod part_paths;

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
            .eval_text("Part[Association[a -> 1, b -> 2], 0]")
            .unwrap()
            .value
            .to_string(),
        "Association"
    );
    let zero_in_part_list = session
        .eval_text("Part[Association[a -> 1, b -> 2], {0}]")
        .unwrap();
    assert_eq!(
        zero_in_part_list.value.to_string(),
        "Part[Association[Rule[a, 1], Rule[b, 2]], List[0]]"
    );
    assert!(
        zero_in_part_list
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "partw")
    );
    let mixed_zero_part_list = session
        .eval_text("Part[Association[a -> 1, b -> 2], {1, 0}]")
        .unwrap();
    assert_eq!(
        mixed_zero_part_list.value.to_string(),
        "Part[Association[Rule[a, 1], Rule[b, 2]], List[1, 0]]"
    );
    assert!(
        mixed_zero_part_list
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "partw")
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
            .eval_text("Extract[{1}][f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "a"
    );
    assert_eq!(
        session
            .eval_text("Extract[{{1}, {2}}][f[a, b]]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session.eval_text("Extract[x]").unwrap().value.to_string(),
        "Function[Extract[Slot[1], x]]"
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
    let invalid_nested_all = session.eval_text("Extract[f[a, b, c], {{{All}}}]").unwrap();
    assert_eq!(
        invalid_nested_all.value.to_string(),
        "Extract[f[a, b, c], List[List[List[All]]]]"
    );
    assert!(
        invalid_nested_all
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "psl1")
    );
    let invalid_deeper_nested = session.eval_text("Extract[f[a, b, c], {{{{1}}}}]").unwrap();
    assert_eq!(
        invalid_deeper_nested.value.to_string(),
        "Extract[f[a, b, c], List[List[List[List[1]]]]]"
    );
    assert!(
        invalid_deeper_nested
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "psl1")
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

    let empty_part = session.eval_text("Part[]").unwrap();
    assert_eq!(empty_part.value.to_string(), "Part[]");
    assert!(
        empty_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "argm")
    );

    let empty_extract = session.eval_text("Extract[]").unwrap();
    assert_eq!(empty_extract.value.to_string(), "Extract[]");
    assert!(
        empty_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "argt")
    );

    let too_many_extract = session.eval_text("Extract[x, {}, h, q]").unwrap();
    assert_eq!(
        too_many_extract.value.to_string(),
        "Extract[x, List[], h, q]"
    );
    assert!(
        too_many_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "argt")
    );

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

    let zero_step_part = session.eval_text("Part[f[a, b], 1 ;; 2 ;; 0]").unwrap();
    assert_eq!(
        zero_step_part.value.to_string(),
        "Part[f[a, b], Span[1, 2, 0]]"
    );
    assert!(
        zero_step_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "take")
    );

    let invalid_span_part = session.eval_text("Part[f[a, b], 1 ;; 2 ;; x]").unwrap();
    assert_eq!(
        invalid_span_part.value.to_string(),
        "Part[f[a, b], Span[1, 2, x]]"
    );
    assert!(
        invalid_span_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "span")
    );

    let invalid_upto_part = session.eval_text("Part[f[a, b], 1 ;; UpTo[-1]]").unwrap();
    assert_eq!(
        invalid_upto_part.value.to_string(),
        "Part[f[a, b], Span[1, UpTo[-1]]]"
    );
    assert!(
        invalid_upto_part
            .messages
            .iter()
            .any(|message| message.symbol == "UpTo" && message.tag == "innf")
    );
    assert!(
        invalid_upto_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "span")
    );

    let bad_key_part = session.eval_text("Part[f[a], Key[x]]").unwrap();
    assert_eq!(bad_key_part.value.to_string(), "Part[f[a], Key[x]]");
    assert!(
        bad_key_part
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "pspec1")
    );

    let bad_key_part_list = session.eval_text("Part[f[a, b], {Key[x], 1}]").unwrap();
    assert_eq!(
        bad_key_part_list.value.to_string(),
        "Part[f[a, b], List[Key[x], 1]]"
    );
    assert!(
        bad_key_part_list
            .messages
            .iter()
            .any(|message| message.symbol == "Part"
                && message.tag == "pspec1"
                && message.text.contains("List[Key[x], 1]"))
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
    assert!(
        too_deep_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract"
                && message.tag == "partd"
                && message.text.contains("List[All, 2]")
                && !message.text.contains("Extract[f[a]"))
    );

    let too_deep_extract_multi = session.eval_text("Extract[f[a], {{1}, {All, 2}}]").unwrap();
    assert_eq!(
        too_deep_extract_multi.value.to_string(),
        "Extract[f[a], List[List[1], List[All, 2]]]"
    );
    assert!(
        too_deep_extract_multi
            .messages
            .iter()
            .any(|message| message.symbol == "Extract"
                && message.tag == "partd"
                && message.text.contains("List[All, 2]")
                && !message.text.contains("List[List[1], List[All, 2]]"))
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

    let zero_step_extract = session
        .eval_text("Extract[f[a, b], {1 ;; 2 ;; 0}]")
        .unwrap();
    assert_eq!(
        zero_step_extract.value.to_string(),
        "Extract[f[a, b], List[Span[1, 2, 0]]]"
    );
    assert!(
        zero_step_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "take")
    );

    let invalid_span_extract = session
        .eval_text("Extract[f[a, b], {1 ;; 2 ;; x}]")
        .unwrap();
    assert_eq!(
        invalid_span_extract.value.to_string(),
        "Extract[f[a, b], List[Span[1, 2, x]]]"
    );
    assert!(
        invalid_span_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "span")
    );

    let invalid_upto_extract = session
        .eval_text("Extract[f[a, b], {1 ;; UpTo[-1]}]")
        .unwrap();
    assert_eq!(
        invalid_upto_extract.value.to_string(),
        "Extract[f[a, b], List[Span[1, UpTo[-1]]]]"
    );
    assert!(
        invalid_upto_extract
            .messages
            .iter()
            .any(|message| message.symbol == "UpTo" && message.tag == "innf")
    );
    assert!(
        invalid_upto_extract
            .messages
            .iter()
            .any(|message| message.symbol == "Extract" && message.tag == "span")
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
