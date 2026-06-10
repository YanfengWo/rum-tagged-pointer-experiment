use super::*;

#[test]
fn set_writes_association_function_and_part_keys() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("a = <||>; a[\"=\"] = 1; {a, Lookup[a, \"=\"], a[[\"=\"]]}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[\"=\", 1]], 1, 1]"
    );
    assert_eq!(
        session
            .eval_text("a = Association[x -> 1, y -> 2]; a[x] = 9; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, 9], Rule[y, 2]]"
    );
    assert_eq!(
        session
            .eval_text("k = \"x\"; a = <||>; a[k] = 1; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"x\", 1]]"
    );
    assert_eq!(
        session
            .eval_text("k = \"x\"; a = <||>; a[[Key[k]]] = 2; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[\"x\", 2]]"
    );
    assert_eq!(
        session
            .eval_text("a = Association[x -> Association[y -> 1]]; a[x, y] = 2; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, Association[Rule[y, 2]]]]"
    );
}

#[test]
fn set_association_lvalue_falls_back_or_reports_failed_nested_keys() {
    let mut session = Session::new();

    let missing = session.eval_text("a = <||>; a[x, y] = 2").unwrap();
    assert_eq!(missing.value.to_string(), "2");
    assert!(
        missing
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "kval")
    );
    assert_eq!(
        session.eval_text("a").unwrap().value.to_string(),
        "Association[]"
    );

    assert_eq!(
        session
            .eval_text("a = Association[x -> 1]; a[x, y] = 2; a")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[x, 1]]"
    );
    assert_eq!(
        session
            .eval_text("ClearAll[f]; f[1] = old; f[1] = new; {f[1], DownValues[f]}")
            .unwrap()
            .value
            .to_string(),
        "List[new, List[RuleDelayed[HoldPattern[f[1]], new]]]"
    );
}
