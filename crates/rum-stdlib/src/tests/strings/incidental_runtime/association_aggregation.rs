use super::super::*;

// Moved from crates/rum-runtime/src/runtime/tests/associations_aggregate_surface.rs.

#[test]
fn merge_catenate_counts_groupby_and_positionindex_cover_first_pass_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Merge[{Association[a -> 1, b -> 2], Association[a -> 3]}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 4], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{Association[a -> 1], {a -> 2, b -> 3}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{a -> 1, {a -> 2, b -> 3}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{{{a -> 1}, {a -> 2, b -> 3}}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3], Rule[b, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[{{Association[a -> 1], {a -> 2}}}, Total]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 3]]"
    );
    assert_eq!(
        session
            .eval_text("Merge[Total][{Association[a -> 1, b -> 2], Association[a -> 3]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 4], Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {Merge[{Association[a :> (x = x + 1)], Association[a :> (x = x + 1)]}, Function[v, v]], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, List[1, 2]]], 2]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {Merge[{Association[a :> (x = x + 1)], Association[a -> 2]}, HoldComplete], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[a, HoldComplete[List[Set[x, Plus[x, 1]], 2]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[{Association[a -> 1], Association[b -> 2]}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> {1, 2}, b -> {3}]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> 1, b -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> Missing[], b -> {2}]]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("Catenate[Association[a -> Missing[\"x\"], b -> Missing[]]]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    let outcome = session
        .eval_text("Catenate[Association[a -> {1}, b -> 2]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Catenate[Association[Rule[a, List[1]], Rule[b, 2]]]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Catenate"
            && message.tag == "invrp"
            && message.text == "The argument 2 is not a valid Association or a list."
    }));
    let outcome = session
        .eval_text("Catenate[Association[a -> Missing[], b -> 2]]")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "Catenate[Association[Rule[a, Missing[]], Rule[b, 2]]]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Catenate"
            && message.tag == "invrp"
            && message.text == "The argument 2 is not a valid Association or a list."
    }));
    assert_eq!(
        session
            .eval_text("Catenate[{Association[a -> {1, 2}], {3, 4}}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 3, 4]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Catenate[Association[a :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[1], 1]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Catenate[Association[a :> {x = x + 1}, b :> {x = x + 1}]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 2]"
    );
    let outcome = session
        .eval_text("x = 0; {Catenate[Association[a :> {x = x + 1}, b :> (x = x + 1)]], x}")
        .unwrap();
    assert_eq!(
        outcome.value.to_string(),
        "List[Catenate[Association[RuleDelayed[a, List[Set[x, Plus[x, 1]]]], RuleDelayed[b, Set[x, Plus[x, 1]]]]], 2]"
    );
    assert!(outcome.messages.iter().any(|message| {
        message.symbol == "Catenate"
            && message.tag == "invrp"
            && message.text == "The argument 2 is not a valid Association or a list."
    }));
    assert_eq!(
        session
            .eval_text(
                "x = 0; {Catenate[{Association[a :> (x = x + 1)], Association[b :> (x = x + 1)]}], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 2]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{a, b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2], Rule[b, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Counts[Association[a -> countedX, b -> countedX, c -> countedY]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, 2], Rule[countedY, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Counts[{a, b, a}, {a, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 2], Rule[b, 1], Rule[c, 0]]"
    );
    assert_eq!(
        session
            .eval_text(
                "Counts[Association[a -> countedX, b -> countedX, c -> countedY], {countedX, countedY, countedZ}]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, 2], Rule[countedY, 1], Rule[countedZ, 0]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Counts[Association[a :> (x = x + 1), b :> (x = x + 1)]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, 1], Rule[2, 1]], 2]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {Counts[Association[a :> (x = x + 1), b -> 1], {1, 2}], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, 2], Rule[2, 0]], 1]"
    );
    assert_eq!(
        session
            .eval_text("CountsBy[{a, bb, ccc}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, 1], Rule[2, 1], Rule[3, 1]]"
    );
    assert_eq!(
        session
            .eval_text("CountsBy[StringLength[ToString[#]] &][{a, bb, ccc}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, 1], Rule[2, 1], Rule[3, 1]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{a, bb, ccc, dd}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, List[a]], Rule[2, List[bb, dd]], Rule[3, List[ccc]]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[{a, bb, ccc, dd}, StringLength[ToString[#]] &, Last]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, a], Rule[2, dd], Rule[3, ccc]]"
    );
    assert_eq!(
        session
            .eval_text("GroupBy[StringLength[ToString[#]] &][{a, bb, ccc, dd}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[1, List[a]], Rule[2, List[bb, dd]], Rule[3, List[ccc]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[first -> countedX, second -> countedX, third -> countedY], Identity]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, Association[Rule[first, countedX], Rule[second, countedX]]], Rule[countedY, Association[Rule[third, countedY]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {GroupBy[Association[first :> (x = x + 1), second :> (x = x + 1)], Identity], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[1, Association[RuleDelayed[first, Set[x, Plus[x, 1]]]]], Rule[2, Association[RuleDelayed[second, Set[x, Plus[x, 1]]]]]], 2]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Identity][Association[first -> countedX, second -> countedX, third -> countedY]]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, Association[Rule[first, countedX], Rule[second, countedX]]], Rule[countedY, Association[Rule[third, countedY]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[first -> countedX, second -> countedX, third -> countedY], Identity, Identity]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, Association[Rule[first, countedX], Rule[second, countedX]]], Rule[countedY, Association[Rule[third, countedY]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[first -> countedX, second -> countedX, third -> countedY], Identity, HoldComplete]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, HoldComplete[Association[Rule[first, countedX], Rule[second, countedX]]]], Rule[countedY, HoldComplete[Association[Rule[third, countedY]]]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "GroupBy[Association[first -> countedX, second -> countedX, third -> countedY], Identity, Last]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, countedX], Rule[countedY, countedY]]"
    );
    assert_eq!(
        session
            .eval_text("PositionIndex[{a, b, a}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, List[1, 3]], Rule[b, List[2]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "PositionIndex[Association[first -> countedX, second -> countedX, third -> countedY]]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[countedX, List[first, second]], Rule[countedY, List[third]]]"
    );
}
