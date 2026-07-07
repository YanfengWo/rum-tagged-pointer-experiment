use super::*;

#[test]
fn reverse_covers_current_positive_level_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Reverse[{a, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "List[c, b, a]"
    );
    assert_eq!(
        session
            .eval_text("Reverse[f[a, b, c]]")
            .unwrap()
            .value
            .to_string(),
        "f[c, b, a]"
    );
    assert_eq!(
        session
            .eval_text("Reverse[{{a, b}, {c, d}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[b, a], List[d, c]]"
    );
    assert_eq!(
        session
            .eval_text("Reverse[{{a, b}, {c, d}}, {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[d, c], List[b, a]]"
    );
    assert_eq!(
        session
            .eval_text("Reverse[f[g[a, b], g[c, d]], 2]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b, a], g[d, c]]"
    );
    assert_eq!(
        session
            .eval_text("Reverse[{{a, b}, {c, d}}, All]")
            .unwrap()
            .value
            .to_string(),
        "List[List[d, c], List[b, a]]"
    );
}

#[test]
fn reverse_reports_current_invalid_boundaries() {
    let mut session = core_session();

    let atomic = session.eval_text("Reverse[a]").unwrap();
    assert_eq!(atomic.value.to_string(), "Reverse[a]");
    assert!(
        atomic
            .messages
            .iter()
            .any(|message| message.symbol == "Reverse" && message.tag == "normal")
    );

    let invalid_level = session.eval_text("Reverse[{a, b}, 0]").unwrap();
    assert_eq!(invalid_level.value.to_string(), "Reverse[List[a, b], 0]");
    assert!(
        invalid_level
            .messages
            .iter()
            .any(|message| message.symbol == "Reverse" && message.tag == "ilsmp")
    );
}

#[test]
fn transpose_covers_current_plain_list_matrix_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Transpose[{{a, b}, {c, d}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, c], List[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("Transpose[{{a, b}, {c, d}}, {1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b], List[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("Transpose[{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Transpose[{}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Transpose[{{}}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Transpose[{{{a, b}, {c, d}}, {{e, f}, {g, h}}}, {3, 2, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[List[a, e], List[c, g]], List[List[b, f], List[d, h]]]"
    );
    assert_eq!(
        session
            .eval_text("Transpose[{{{a, b}, {c, d}}, {{e, f}, {g, h}}}, {2, 1, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[List[a, b], List[e, f]], List[List[c, d], List[g, h]]]"
    );
}

#[test]
fn transpose_reports_current_invalid_boundaries() {
    let mut session = core_session();

    let non_list = session.eval_text("Transpose[f[g[a, b], g[c, d]]]").unwrap();
    assert_eq!(non_list.value.to_string(), "Transpose[f[g[a, b], g[c, d]]]");
    assert!(non_list.messages.is_empty());

    let ragged = session.eval_text("Transpose[{{a}, {b, c}}]").unwrap();
    assert_eq!(
        ragged.value.to_string(),
        "Transpose[List[List[a], List[b, c]]]"
    );
    assert!(
        ragged
            .messages
            .iter()
            .any(|message| message.symbol == "Transpose" && message.tag == "nmtx")
    );

    let invalid_permutation = session.eval_text("Transpose[{{a, b}}, x]").unwrap();
    assert_eq!(
        invalid_permutation.value.to_string(),
        "Transpose[List[List[a, b]], x]"
    );
    assert!(
        invalid_permutation
            .messages
            .iter()
            .any(|message| message.symbol == "Transpose" && message.tag == "permspec")
    );

    let rank_mismatch = session
        .eval_text("Transpose[{{a, b}, {c, d}}, {3, 2, 1}]")
        .unwrap();
    assert_eq!(
        rank_mismatch.value.to_string(),
        "Transpose[List[List[a, b], List[c, d]], List[3, 2, 1]]"
    );
    assert!(
        rank_mismatch
            .messages
            .iter()
            .any(|message| message.symbol == "Transpose" && message.tag == "permspec")
    );
}
