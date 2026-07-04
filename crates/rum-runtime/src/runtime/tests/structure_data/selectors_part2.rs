use super::*;

#[test]
fn part_and_extract_report_branch_selector_failures() {
    let mut session = Session::new();

    let part_all_branch = session.eval_text("Part[f[g[a, b], h[c]], All, 2]").unwrap();
    assert_eq!(
        part_all_branch.value.to_string(),
        "Part[f[g[a, b], h[c]], All, 2]"
    );
    assert!(
        part_all_branch
            .messages
            .iter()
            .any(|message| message.symbol == "Part"
                && message.tag == "partw"
                && message.text.contains("2 of h[c]"))
    );

    let part_multi_branch = session
        .eval_text("Part[f[g[a, b], h[c]], {1, 2}, {1, 2}]")
        .unwrap();
    assert_eq!(
        part_multi_branch.value.to_string(),
        "Part[f[g[a, b], h[c]], List[1, 2], List[1, 2]]"
    );
    assert!(
        part_multi_branch
            .messages
            .iter()
            .any(|message| message.symbol == "Part"
                && message.tag == "partw"
                && message.text.contains("List[1, 2] of h[c]"))
    );

    let extract_branch = session
        .eval_text("Extract[f[g[a, b], h[c]], {{{1, 2}, 2}}]")
        .unwrap();
    assert_eq!(
        extract_branch.value.to_string(),
        "Extract[f[g[a, b], h[c]], List[List[List[1, 2], 2]]]"
    );
    assert!(
        extract_branch
            .messages
            .iter()
            .any(|message| message.symbol == "Extract"
                && message.tag == "partw"
                && message.text.contains("2 of h[c]"))
    );
}

#[test]
fn part_and_extract_emit_single_take_for_explicit_invalid_spans() {
    let mut session = Session::new();

    let invalid_part_span = session
        .eval_text("Part[f[a, b, c], All ;; All ;; -1]")
        .unwrap();
    assert_eq!(
        invalid_part_span.value.to_string(),
        "Part[f[a, b, c], Span[All, All, -1]]"
    );
    assert_eq!(
        invalid_part_span
            .messages
            .iter()
            .filter(|message| message.symbol == "Part" && message.tag == "take")
            .count(),
        1
    );

    let invalid_extract_span = session
        .eval_text("Extract[f[a, b, c], {All ;; All ;; -1}]")
        .unwrap();
    assert_eq!(
        invalid_extract_span.value.to_string(),
        "Extract[f[a, b, c], List[Span[All, All, -1]]]"
    );
    assert_eq!(
        invalid_extract_span
            .messages
            .iter()
            .filter(|message| message.symbol == "Extract" && message.tag == "take")
            .count(),
        1
    );
}

#[test]
fn part_and_extract_take_messages_render_span_ranges() {
    let mut session = Session::new();

    let part_range = session.eval_text("Part[f[a], 1 ;; 5]").unwrap();
    assert_eq!(part_range.value.to_string(), "Part[f[a], Span[1, 5]]");
    assert!(part_range.messages.iter().any(|message| {
        message.symbol == "Part"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through 5 in f[a]."
    }));

    let part_reverse_all = session
        .eval_text("Part[f[a, b, c], All ;; All ;; -1]")
        .unwrap();
    assert_eq!(
        part_reverse_all.value.to_string(),
        "Part[f[a, b, c], Span[All, All, -1]]"
    );
    assert!(part_reverse_all.messages.iter().any(|message| {
        message.symbol == "Part"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through -1 in f[a, b, c]."
    }));

    let extract_range = session.eval_text("Extract[f[a], {1 ;; 5}]").unwrap();
    assert_eq!(
        extract_range.value.to_string(),
        "Extract[f[a], List[Span[1, 5]]]"
    );
    assert!(extract_range.messages.iter().any(|message| {
        message.symbol == "Extract"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through 5 in f[a]."
    }));
}

#[test]
fn invalid_upto_span_diagnostics_are_not_duplicated_after_normalization() {
    let mut session = Session::new();

    let part_invalid_upto = session.eval_text("Part[f[a, b], 1 ;; UpTo[3/2]]").unwrap();
    assert_eq!(
        part_invalid_upto.value.to_string(),
        "Part[f[a, b], Span[1, UpTo[3/2]]]"
    );
    assert_eq!(
        part_invalid_upto
            .messages
            .iter()
            .filter(|message| message.symbol == "UpTo" && message.tag == "innf")
            .count(),
        1
    );
    assert_eq!(
        part_invalid_upto
            .messages
            .iter()
            .filter(|message| message.symbol == "Part" && message.tag == "span")
            .count(),
        1
    );

    let extract_invalid_upto = session
        .eval_text("Extract[f[a, b], {1 ;; UpTo[3/2]}]")
        .unwrap();
    assert_eq!(
        extract_invalid_upto.value.to_string(),
        "Extract[f[a, b], List[Span[1, UpTo[3/2]]]]"
    );
    assert_eq!(
        extract_invalid_upto
            .messages
            .iter()
            .filter(|message| message.symbol == "UpTo" && message.tag == "innf")
            .count(),
        1
    );
    assert_eq!(
        extract_invalid_upto
            .messages
            .iter()
            .filter(|message| message.symbol == "Extract" && message.tag == "span")
            .count(),
        1
    );
}

#[test]
fn direct_empty_span_selects_full_range() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[f[a, b], Span[]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], Span[0]]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], Span[], 1]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b], Span[], 1]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], Span[0], 1]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {Span[]}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b], {Span[0]}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a], h[b]], {Span[], 1}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[g[a], h[b]], {Span[0], 1}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Extract[1, {Span[]}]")
            .unwrap()
            .value
            .to_string(),
        "Integer[]"
    );
    assert_eq!(
        session
            .eval_text("Extract[1, {Span[0]}]")
            .unwrap()
            .value
            .to_string(),
        "Integer[]"
    );
}

#[test]
fn direct_single_endpoint_span_selects_prefix_or_suffix_ranges() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], Span[2]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], Span[-2]]")
            .unwrap()
            .value
            .to_string(),
        "f[b, c]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b, c], {Span[2]}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Extract[f[a, b, c], {Span[-2]}]")
            .unwrap()
            .value
            .to_string(),
        "f[b, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], Span[UpTo[2]]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], Span[UpTo[0]]]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], Span[UpTo[Infinity]]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Extract[1, {Span[UpTo[2]]}]")
            .unwrap()
            .value
            .to_string(),
        "Integer[]"
    );

    let invalid_positive = session.eval_text("Part[f[a, b, c], Span[4]]").unwrap();
    assert_eq!(
        invalid_positive.value.to_string(),
        "Part[f[a, b, c], Span[4]]"
    );
    assert!(invalid_positive.messages.iter().any(|message| {
        message.symbol == "Part"
            && message.tag == "take"
            && message.text == "Cannot take positions 1 through 4 in f[a, b, c]."
    }));

    let invalid_negative = session
        .eval_text("Extract[f[a, b, c], {Span[-4]}]")
        .unwrap();
    assert_eq!(
        invalid_negative.value.to_string(),
        "Extract[f[a, b, c], List[Span[-4]]]"
    );
    assert!(invalid_negative.messages.iter().any(|message| {
        message.symbol == "Extract"
            && message.tag == "take"
            && message.text == "Cannot take positions -4 through -1 in f[a, b, c]."
    }));
}

#[test]
fn invalid_span_messages_render_infix_specs() {
    let mut session = Session::new();

    let invalid_two_arg_span = session.eval_text("Part[f[a, b], Span[1, x]]").unwrap();
    assert_eq!(
        invalid_two_arg_span.value.to_string(),
        "Part[f[a, b], Span[1, x]]"
    );
    assert!(invalid_two_arg_span.messages.iter().any(|message| {
        message.symbol == "Part"
            && message.tag == "span"
            && message
                .text
                .starts_with("1 ;; x is not a valid Span specification.")
    }));

    let invalid_three_arg_span = session
        .eval_text("Extract[f[a, b], {1 ;; 2 ;; x}]")
        .unwrap();
    assert_eq!(
        invalid_three_arg_span.value.to_string(),
        "Extract[f[a, b], List[Span[1, 2, x]]]"
    );
    assert!(invalid_three_arg_span.messages.iter().any(|message| {
        message.symbol == "Extract"
            && message.tag == "span"
            && message
                .text
                .starts_with("1 ;; 2 ;; x is not a valid Span specification.")
    }));

    let invalid_arity_span = session
        .eval_text("Part[f[a, b], Span[1, 2, 3, 4]]")
        .unwrap();
    assert!(invalid_arity_span.messages.iter().any(|message| {
        message.symbol == "Part"
            && message.tag == "span"
            && message
                .text
                .starts_with("Span[1, 2, 3, 4] is not a valid Span specification.")
    }));
}
