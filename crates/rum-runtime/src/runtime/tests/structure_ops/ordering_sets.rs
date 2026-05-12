use super::*;

#[test]
fn sort_and_sortby_cover_current_top_level_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Sort[{b, a, c, a}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Sort[f[2, 10, 1, a, \"x\"]]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 10, \"x\", a]"
    );
    assert_eq!(
        session
            .eval_text("SortBy[{bb, a, ccc}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "List[a, bb, ccc]"
    );
    assert_eq!(
        session
            .eval_text("SortBy[f[bb, a, ccc], StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "f[a, bb, ccc]"
    );
    assert_eq!(
        session
            .eval_text("SortBy[StringLength[ToString[#]] &][{bb, a, ccc}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, bb, ccc]"
    );
}

#[test]
fn union_and_complement_cover_current_same_head_subset() {
    let mut session = Session::new();

    assert_eq!(
        session.eval_text("Union[]").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Union[{b, a, c, a}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Union[f[b, a], f[c, a]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Complement[{a, b, c, b}, {b}, {d}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Complement[f[a, b, c, b], f[b], f[d]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Complement[{b, a, c, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
}

#[test]
fn ordering_and_set_helpers_report_current_invalid_boundaries() {
    let mut session = Session::new();

    let sort_atom = session.eval_text("Sort[a]").unwrap();
    assert_eq!(sort_atom.value.to_string(), "Sort[a]");
    assert!(
        sort_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Sort" && message.tag == "normal")
    );

    let sort_by_atom = session.eval_text("SortBy[a, f]").unwrap();
    assert_eq!(sort_by_atom.value.to_string(), "SortBy[a, f]");
    assert!(
        sort_by_atom
            .messages
            .iter()
            .any(|message| message.symbol == "SortBy" && message.tag == "normal")
    );

    let union_atom = session.eval_text("Union[{a}, b]").unwrap();
    assert_eq!(union_atom.value.to_string(), "Union[List[a], b]");
    assert!(
        union_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Union" && message.tag == "normal")
    );

    let union_heads = session.eval_text("Union[{a}, f[b]]").unwrap();
    assert_eq!(union_heads.value.to_string(), "Union[List[a], f[b]]");
    assert!(
        union_heads
            .messages
            .iter()
            .any(|message| message.symbol == "Union" && message.tag == "heads")
    );

    let complement_heads = session.eval_text("Complement[f[a, b], g[b]]").unwrap();
    assert_eq!(
        complement_heads.value.to_string(),
        "Complement[f[a, b], g[b]]"
    );
    assert!(
        complement_heads
            .messages
            .iter()
            .any(|message| message.symbol == "Complement" && message.tag == "heads")
    );

    let complement_empty = session.eval_text("Complement[]").unwrap();
    assert_eq!(complement_empty.value.to_string(), "Complement[]");
    assert!(
        complement_empty
            .messages
            .iter()
            .any(|message| message.symbol == "Complement" && message.tag == "argm")
    );
}
