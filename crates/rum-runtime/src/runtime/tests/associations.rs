use super::*;

#[test]
fn association_and_associationq_cover_first_pass_rule_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Association[a -> 1, b :> 2]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], RuleDelayed[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1, b -> 2, a -> 3]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Association[{a -> 1, b -> 2}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Association[{a -> 1, {b -> 2}, Association[c -> 3]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 2], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("<|a -> 1, b -> 2, a -> 3|>")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("AssociationQ[Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1, b -> 2, a -> 3]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 3], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[{Association[a -> 1]}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[f[Association[a -> 1]]]")
            .unwrap()
            .value
            .to_string(),
        "f[List[Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> Association[b -> 2]]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, Association[Rule[b, 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1], Association]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1], All]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[{Association[a -> 1]}, All]")
            .unwrap()
            .value
            .to_string(),
        "List[List[Rule[a, 1]]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1], 1]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1], Infinity]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Normal[Association[a -> 1], List]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Normal[Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("AssociationQ[List[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn lookup_keys_values_and_key_cover_first_pass_association_queries() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1, b -> 2], b]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; assoc = Association[k :> (x = x + 1)]; {assoc[k], Lookup[assoc, k], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 2]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], b, 99]")
            .unwrap()
            .value
            .to_string(),
        "99"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], b]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> Association[b -> 2]], {Key[a], Key[b]}]")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[b, 2]], Missing[\"KeyAbsent\", b]]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], Key[b], 99]")
            .unwrap()
            .value
            .to_string(),
        "99"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], Key[b]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1, b -> 2], {b, a, c}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, Missing[\"KeyAbsent\", c]]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[a][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a, b}][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, Missing[\"KeyAbsent\", b]]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], {a, b}, 99]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 99]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[Association[a -> 1], b, 99, Hold]")
            .unwrap()
            .value
            .to_string(),
        "99"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Lookup[Association[a :> (x = x + 1)], a, missing, Hold], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Hold[Set[x, Plus[x, 1]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{Association[a -> 1], Association[a -> 2, b -> 3]}, a]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{Association[a -> 1], Association[b -> 2]}, a, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 0]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{Association[a -> 1], Association[b -> 2]}, a, 0, Hold]")
            .unwrap()
            .value
            .to_string(),
        "List[Hold[1], 0]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a -> 1, b -> 2}, a]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Lookup[a -> 1, c]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", c]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a -> 1, b -> 2}, Key[b]]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a -> 1, b -> 2}, Key[c]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", c]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{a -> 1, b -> 2}, {b, a, c}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, Missing[\"KeyAbsent\", c]]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{{a -> 1}, {b -> 2}}, a, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 0]"
    );
    assert_eq!(
        session
            .eval_text("Lookup[{Association[a -> 1], {b -> 2}}, a, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 0]"
    );
    assert_eq!(
        session
            .eval_text(
                "Lookup[{Association[a -> Association[b -> 2]], Association[a -> Association[b -> 3]]}, {Key[a], Key[b]}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[Association[Rule[b, 2]], Missing[\"KeyAbsent\", b]], List[Association[Rule[b, 3]], Missing[\"KeyAbsent\", b]]]"
    );
    assert_eq!(
        session
            .eval_text("Keys[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Keys[List[a -> 1, b -> 2], Hold]")
            .unwrap()
            .value
            .to_string(),
        "List[Hold[a], Hold[b]]"
    );
    assert_eq!(
        session
            .eval_text("Keys[{Association[a -> 1], {b -> 2}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a], List[b]]"
    );
    assert_eq!(
        session
            .eval_text("Values[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Values[Association[a :> (x = x + 1), b -> 2]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 1]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Values[Association[a :> (x = x + 1)], Hold], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[Hold[Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("Values[{Association[a -> 1], {b -> 2}}, Hold]")
            .unwrap()
            .value
            .to_string(),
        "List[List[Hold[1]], List[Hold[2]]]"
    );
    assert_eq!(
        session.eval_text("Key[a]").unwrap().value.to_string(),
        "Key[a]"
    );
    assert_eq!(
        session
            .eval_text("Key[a][Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Key[b][Association[a -> 1]]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Key[2][Association[2 -> \"two\", 1 -> \"one\"]]")
            .unwrap()
            .value
            .to_string(),
        "\"two\""
    );
    let outcome = session.eval_text("Key[a][b]").unwrap();
    assert_eq!(outcome.value.to_string(), "Key[a][b]");
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Key"
            && message.tag == "invak"
            && message.text == "The argument b is not a valid Association."
    }));
    let outcome = session.eval_text("Key[a][Association[a -> 1], b]").unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Key[a][Association[Rule[a, 1]], b]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Key"
            && message.tag == "argx"
            && message.text == "Key[a] called with 2 arguments; 1 argument is expected."
    }));
    assert_eq!(
        session
            .eval_text("Association[a -> 1, b -> 2][a]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1][b]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> Association[b -> 2]][a, b]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> Association[b -> 2]][a, c]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", c]"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1][b, c]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", b]"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1][a, b]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]][a, b]"
    );
    assert_eq!(
        session
            .eval_text("Association[a -> 1][]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1]]"
    );
}
