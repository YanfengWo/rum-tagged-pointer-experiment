use super::*;

#[test]
fn associateto_keydropfrom_and_keyvaluepattern_cover_first_pass_mutation_surface() {
    let mut session = Session::new();

    let literal_associate = session
        .eval_text("AssociateTo[Association[a -> 1, b -> 2], b -> 9]")
        .unwrap();
    assert_eq!(
        literal_associate.value.to_string(),
        "AssociateTo[Association[Rule[a, 1], Rule[b, 2]], Rule[b, 9]]"
    );
    assert!(
        literal_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    let literal_drop = session
        .eval_text("KeyDropFrom[Association[a -> 1, b -> 2, c -> 3], {a, c}]")
        .unwrap();
    assert_eq!(literal_drop.value.to_string(), "Association[Rule[b, 2]]");
    assert!(
        literal_drop
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "write")
    );
    session.eval_text("assoc = Association[a -> 1]").unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[assoc, b -> 2]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    session.eval_text("ClearAll[f]").unwrap();
    session.eval_text("f[1] = Association[a -> 1]").unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[f[1], b -> 2]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session.eval_text("f[1]").unwrap().value.to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    session
        .eval_text("f[1] = Association[a -> 1, b -> 2]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[f[1], a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session.eval_text("f[1]").unwrap().value.to_string(),
        "Association[Rule[b, 2]]"
    );
    session.eval_text("Protect[f]").unwrap();
    let blocked_compound_associate = session.eval_text("AssociateTo[f[1], c -> 3]").unwrap();
    assert_eq!(
        blocked_compound_associate.value.to_string(),
        "AssociateTo[f[1], Rule[c, 3]]"
    );
    assert!(
        blocked_compound_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    let protected_compound_drop = session.eval_text("KeyDropFrom[f[1], b]").unwrap();
    assert_eq!(protected_compound_drop.value.to_string(), "Association[]");
    assert!(
        protected_compound_drop
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "write")
    );
    session.eval_text("Unprotect[f]").unwrap();
    assert_eq!(
        session.eval_text("f[1]").unwrap().value.to_string(),
        "Association[Rule[b, 2]]"
    );
    session.eval_text("ClearAll[f]").unwrap();
    assert_eq!(
        session
            .eval_text("x = 0; assoc = Association[]; {AssociateTo[assoc, a :> (x = x + 1)], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[RuleDelayed[a, Set[x, Plus[x, 1]]]]"
    );
    session.eval_text("assoc = Association[a -> 1]").unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[assoc, {Association[b -> 2], c -> 3}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    session
        .eval_text("assoc = Association[a -> 1, b -> 2, c -> 3]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[assoc, {a, c}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; assoc = Association[a :> (x = x + 1), b -> 2]; {KeyDropFrom[assoc, b], assoc, x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    session
        .eval_text("assoc = Association[a -> 1, Key[a] -> 7, b -> 2]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[assoc, {Key[a], b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[Key[a], 7]]"
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[Key[a], 7]]"
    );
    session.eval_text("parts = {Association[a -> 1]}").unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[parts[[1]], b -> 2]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, 2]]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1], Rule[b, 2]]]"
    );
    session
        .eval_text("parts = {Association[a -> 1], Association[a -> 2]}")
        .unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[parts[[{1, 2}]], b -> 3]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, 3]], Association[Rule[a, 2], Rule[b, 3]]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1], Rule[b, 3]], Association[Rule[a, 2], Rule[b, 3]]]"
    );
    session
        .eval_text("parts = {Association[a -> 1], Association[a -> 2]}")
        .unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[parts[[All]], b -> 3]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, 3]], Association[Rule[a, 2], Rule[b, 3]]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1], Rule[b, 3]], Association[Rule[a, 2], Rule[b, 3]]]"
    );
    session
        .eval_text("parts = {Association[a -> 1], Association[a -> 2]}")
        .unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[parts[[1 ;; 2]], b -> 4]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, 1], Rule[b, 4]], Association[Rule[a, 2], Rule[b, 4]]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1], Rule[b, 4]], Association[Rule[a, 2], Rule[b, 4]]]"
    );
    session
        .eval_text("parts = {Association[a -> 1], 1}")
        .unwrap();
    let invalid_part_associate = session
        .eval_text("AssociateTo[parts[[{1, 2}]], b -> 3]")
        .unwrap();
    assert!(
        invalid_part_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invak")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1]], 1]"
    );
    let invalid_all_associate = session
        .eval_text("AssociateTo[parts[[All]], b -> 3]")
        .unwrap();
    assert_eq!(
        invalid_all_associate.value.to_string(),
        "AssociateTo[Part[parts, All], Rule[b, 3]]"
    );
    assert!(
        invalid_all_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invak")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1]], 1]"
    );
    session
        .eval_text("nested = Association[a -> Association[b -> 1]]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("AssociateTo[nested[[Key[a]]], c -> 3]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, Association[Rule[b, 1], Rule[c, 3]]]]"
    );
    assert_eq!(
        session.eval_text("nested").unwrap().value.to_string(),
        "Association[Rule[a, Association[Rule[b, 1], Rule[c, 3]]]]"
    );
    session.eval_text("assoc = Association[a -> 1]").unwrap();
    let missing_key_associate = session
        .eval_text("AssociateTo[assoc[[Key[b]]], c -> 3]")
        .unwrap();
    assert_eq!(
        missing_key_associate.value.to_string(),
        "AssociateTo[Part[assoc, Key[b]], Rule[c, 3]]"
    );
    assert!(
        missing_key_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "keynx")
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[a, 1]]"
    );
    session
        .eval_text("nested = Association[a -> Association[b -> 1]]")
        .unwrap();
    let missing_nested_key_associate = session
        .eval_text("AssociateTo[nested[[Key[a], Key[c]]], d -> 4]")
        .unwrap();
    assert_eq!(
        missing_nested_key_associate.value.to_string(),
        "AssociateTo[Part[nested, Key[a], Key[c]], Rule[d, 4]]"
    );
    assert!(
        missing_nested_key_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "keynx")
    );
    assert_eq!(
        session.eval_text("nested").unwrap().value.to_string(),
        "Association[Rule[a, Association[Rule[b, 1]]]]"
    );
    session.eval_text("parts = {Association[a -> 1]}").unwrap();
    let missing_part_associate = session
        .eval_text("AssociateTo[parts[[2]], b -> 2]")
        .unwrap();
    assert_eq!(
        missing_part_associate.value.to_string(),
        "AssociateTo[Part[parts, 2], Rule[b, 2]]"
    );
    assert!(
        missing_part_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "partw")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1]]]"
    );
    let atom_part_associate = session
        .eval_text("x = 1; AssociateTo[x[[1]], b -> 2]")
        .unwrap();
    assert_eq!(
        atom_part_associate.value.to_string(),
        "AssociateTo[Part[x, 1], Rule[b, 2]]"
    );
    assert!(!atom_part_associate.messages.iter().any(|message| {
        message.symbol == "AssociateTo" && (message.tag == "partw" || message.tag == "partd")
    }));
    session
        .eval_text("parts = {Association[a -> 1, b -> 2]}")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[parts[[1]], a]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[b, 2]]]"
    );
    session
        .eval_text("parts = {Association[a -> 1, b -> 2], Association[a -> 3, c -> 4]}")
        .unwrap();
    let invalid_multi_drop = session.eval_text("KeyDropFrom[parts[[All]], a]").unwrap();
    assert_eq!(
        invalid_multi_drop.value.to_string(),
        "KeyDropFrom[Part[parts, All], a]"
    );
    assert!(
        invalid_multi_drop
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    let invalid_list_drop = session
        .eval_text("KeyDropFrom[parts[[{1, 2}]], a]")
        .unwrap();
    assert_eq!(
        invalid_list_drop.value.to_string(),
        "KeyDropFrom[Part[parts, List[1, 2]], a]"
    );
    assert!(
        invalid_list_drop
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    let invalid_span_drop = session
        .eval_text("KeyDropFrom[parts[[1 ;; 2]], a]")
        .unwrap();
    assert_eq!(
        invalid_span_drop.value.to_string(),
        "KeyDropFrom[Part[parts, Span[1, 2]], a]"
    );
    assert!(
        invalid_span_drop
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1], Rule[b, 2]], Association[Rule[a, 3], Rule[c, 4]]]"
    );
    session.eval_text("parts = {Association[a -> 1]}").unwrap();
    session.eval_text("Protect[parts]").unwrap();
    let blocked_part_associate = session
        .eval_text("AssociateTo[parts[[1]], b -> 2]")
        .unwrap();
    assert!(
        blocked_part_associate
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "wrsym")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1]]]"
    );
    session.eval_text("Unprotect[parts]").unwrap();
    session
        .eval_text("nested = Association[a -> Association[b -> 1, c -> 2]]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("KeyDropFrom[nested[[Key[a]]], b]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[c, 2]]"
    );
    assert_eq!(
        session.eval_text("nested").unwrap().value.to_string(),
        "Association[Rule[a, Association[Rule[c, 2]]]]"
    );
    session.eval_text("assoc = Association[a -> 1]").unwrap();
    let missing_key_drop = session
        .eval_text("KeyDropFrom[assoc[[Key[b]]], c]")
        .unwrap();
    assert_eq!(
        missing_key_drop.value.to_string(),
        "KeyDropFrom[Part[assoc, Key[b]], c]"
    );
    assert!(
        missing_key_drop
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    assert!(
        !missing_key_drop
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "keynx")
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[a, 1]]"
    );
    session.eval_text("parts = {Association[a -> 1]}").unwrap();
    let missing_part_drop = session.eval_text("KeyDropFrom[parts[[2]], a]").unwrap();
    assert_eq!(
        missing_part_drop.value.to_string(),
        "KeyDropFrom[Part[parts, 2], a]"
    );
    assert!(
        missing_part_drop
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "partw")
    );
    assert!(
        missing_part_drop
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    assert_eq!(
        session.eval_text("parts").unwrap().value.to_string(),
        "List[Association[Rule[a, 1]]]"
    );
    let atom_part_drop = session.eval_text("x = 1; KeyDropFrom[x[[1]], a]").unwrap();
    assert_eq!(
        atom_part_drop.value.to_string(),
        "KeyDropFrom[Part[x, 1], a]"
    );
    assert!(
        atom_part_drop
            .messages
            .iter()
            .any(|message| message.symbol == "Part" && message.tag == "partd")
    );
    assert!(
        atom_part_drop
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    session.eval_text("ClearAll[assoc]").unwrap();
    let missing_value = session.eval_text("AssociateTo[assoc, b -> 2]").unwrap();
    assert_eq!(
        missing_value.value.to_string(),
        "AssociateTo[assoc, Rule[b, 2]]"
    );
    assert!(
        missing_value
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "blnoval")
    );
    session.eval_text("assoc := Association[a -> 1]").unwrap();
    let delayed_value = session.eval_text("AssociateTo[assoc, b -> 2]").unwrap();
    assert_eq!(
        delayed_value.value.to_string(),
        "AssociateTo[assoc, Rule[b, 2]]"
    );
    assert!(
        delayed_value
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "blnoval")
    );
    session.eval_text("assoc = 1").unwrap();
    let invalid_value = session.eval_text("AssociateTo[assoc, b -> 2]").unwrap();
    assert_eq!(
        invalid_value.value.to_string(),
        "AssociateTo[assoc, Rule[b, 2]]"
    );
    assert!(
        invalid_value
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invak")
    );
    let invalid_spec = session.eval_text("AssociateTo[assoc, a]").unwrap();
    assert_eq!(invalid_spec.value.to_string(), "AssociateTo[assoc, a]");
    assert!(
        invalid_spec
            .messages
            .iter()
            .any(|message| message.symbol == "AssociateTo" && message.tag == "invlb")
    );
    session.eval_text("ClearAll[assoc]").unwrap();
    let missing_drop_target = session.eval_text("KeyDropFrom[assoc, a]").unwrap();
    assert_eq!(
        missing_drop_target.value.to_string(),
        "KeyDropFrom[assoc, a]"
    );
    assert!(
        missing_drop_target
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "blnoval")
    );
    session.eval_text("assoc = {a -> 1, b -> 2}").unwrap();
    let invalid_drop_target = session.eval_text("KeyDropFrom[assoc, a]").unwrap();
    assert_eq!(
        invalid_drop_target.value.to_string(),
        "KeyDropFrom[assoc, a]"
    );
    assert!(
        invalid_drop_target
            .messages
            .iter()
            .any(|message| message.symbol == "KeyDropFrom" && message.tag == "invak")
    );
    session.eval_text("assoc = Association[b -> 2]").unwrap();
    session.eval_text("Protect[assoc]").unwrap();
    let blocked = session.eval_text("AssociateTo[assoc, c -> 9]").unwrap();
    assert_eq!(blocked.value.to_string(), "AssociateTo[assoc, Rule[c, 9]]");
    assert!(
        blocked
            .messages
            .iter()
            .any(|message| { message.symbol == "AssociateTo" && message.tag == "wrsym" })
    );
    assert_eq!(
        session.eval_text("assoc").unwrap().value.to_string(),
        "Association[Rule[b, 2]]"
    );
    session
        .eval_text("Unprotect[assoc]; ClearAll[assoc]")
        .unwrap();
}
