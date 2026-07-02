use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn set_operation_attributes_match_wk_anchors() {
    let mut session = Session::new();

    let union_attributes = eval_to_string(&mut session, "Attributes[Union]");
    let intersection_attributes = eval_to_string(&mut session, "Attributes[Intersection]");

    assert_eq!(
        union_attributes,
        "List[Flat, OneIdentity, Protected, ReadProtected]"
    );
    assert_eq!(intersection_attributes, union_attributes);
}

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
fn sort_with_ordering_function_evaluates_comparator() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Sort[{3, 1, 2}, Greater]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 2, 1]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{3, 1, 2}, #1 > #2 &]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 2, 1]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{3, 1, 2}, Order[#1, #2] === 1 &]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{c, a, b}, Order[#1, #2] === 1 &]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Sort[f[3, 1, 2], Less]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{1, 3, 2, 3, 1}, Less]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 1, 2, 3, 3]"
    );
    assert_eq!(
        session
            .eval_text("Sort[{3, 1, 2}, #1 + #2 &]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 1, 2]"
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
fn intersection_covers_wk_anchored_same_head_subset() {
    let mut session = Session::new();

    assert_eq!(
        eval_to_string(&mut session, "Intersection[{1, 2, 3}, {2, 3, 4}]"),
        "List[2, 3]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "Intersection[{1, 2, 3, 4}, {2, 3, 4}, {3, 4, 5}]"
        ),
        "List[3, 4]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Intersection[{3, 1, 2, 1}]"),
        "List[1, 2, 3]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Intersection[f[1, 2], f[2, 3], f[2, 4]]"),
        "f[2]"
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

    let sort_comparator_atom = session.eval_text("Sort[a, Greater]").unwrap();
    assert_eq!(sort_comparator_atom.value.to_string(), "Sort[a, Greater]");
    assert!(
        sort_comparator_atom
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

    let intersection_empty = session.eval_text("Intersection[]").unwrap();
    assert_eq!(intersection_empty.value.to_string(), "Intersection[]");
    assert!(
        intersection_empty
            .messages
            .iter()
            .any(|message| message.symbol == "Intersection" && message.tag == "argm")
    );

    let intersection_atom = session.eval_text("Intersection[{a}, b]").unwrap();
    assert_eq!(
        intersection_atom.value.to_string(),
        "Intersection[List[a], b]"
    );
    assert!(
        intersection_atom
            .messages
            .iter()
            .any(|message| message.symbol == "Intersection" && message.tag == "normal")
    );

    let intersection_heads = session.eval_text("Intersection[{a}, f[a]]").unwrap();
    assert_eq!(
        intersection_heads.value.to_string(),
        "Intersection[List[a], f[a]]"
    );
    assert!(
        intersection_heads
            .messages
            .iter()
            .any(|message| message.symbol == "Intersection" && message.tag == "heads")
    );
}
