use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn missing_and_delete_missing_cover_first_pass_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Missing[\"KeyAbsent\", a]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"KeyAbsent\", a]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{1, Missing[], Missing[\"KeyAbsent\", a], 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[Association[a -> 1, b -> Missing[], c -> 2]]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 2]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{{1, Missing[]}, {2, 3}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1], List[2, 3]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[Association[a -> {Missing[], 1}, b -> Association[c -> Missing[], d -> 2]], Infinity]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, List[1]], Rule[b, Association[Rule[d, 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[Association[a -> {Missing[], 1}, b -> Association[c -> Missing[], d -> 2]], All]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, List[1]], Rule[b, Association[Rule[d, 2]]]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{a -> Missing[], b :> Missing[], c -> 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[c, 2]]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {DeleteMissing[Association[a :> (x = x + 1), b -> Missing[]]], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, Set[x, Plus[x, 1]]]], 0]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{{1, Missing[]}, {2, 3}}, 1, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, Missing[]], List[2, 3]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{{1, Missing[]}, {2, 3}}, 1, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[List[2, 3]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{{1, {Missing[]}}, {2, 3}}, 1, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, List[Missing[]]], List[2, 3]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{{1, {Missing[]}}, {2, 3}}, 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[2, 3]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{{1, {Missing[]}}, {2, 3}}, 1, All]")
            .unwrap()
            .value
            .to_string(),
        "List[List[2, 3]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[Association[a -> {1, Missing[]}, b -> {2, 3}], 1, 1]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, List[2, 3]]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[Association[a -> Missing[], b -> 2], 2, 0]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[Association[a :> Missing[], b -> 2], 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[b, 2]]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{f[Missing[]], 2}, 1, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("DeleteMissing[{a -> {Missing[], 1}, b -> {2}}, 1, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[b, List[2]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {DeleteMissing[Association[a :> {Missing[], x = x + 1}, b -> 2], 1, 0], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, List[Missing[], Set[x, Plus[x, 1]]]], Rule[b, 2]], 1]"
    );
    assert_eq!(
        session
            .eval_text(
                "x = 0; {DeleteMissing[Association[a :> {Missing[], x = x + 1}, b -> 2], 1, 1], x}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[b, 2]], 1]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {DeleteMissing[Association[a :> {1, x = x + 1}, b -> 2], 1, 1], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[RuleDelayed[a, List[1, Set[x, Plus[x, 1]]]], Rule[b, 2]], 1]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {DeleteMissing[Association[outer -> Association[a :> {Missing[], x = x + 1}, b -> 2]], 2, 1], x}")
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[outer, Association[Rule[b, 2]]]], 1]"
    );
    assert_eq!(
        session
            .eval_text("x = 0; {DeleteMissing[{a :> {Missing[], x = x + 1}, b -> 2}, 1, 1], x}")
            .unwrap()
            .value
            .to_string(),
        "List[List[RuleDelayed[a, List[Missing[], Set[x, Plus[x, 1]]]], Rule[b, 2]], 0]"
    );
    session
        .eval_text(
            "x = 0; dmAssoc = DeleteMissing[Association[a :> {Missing[], x = x + 1}, b -> 2], 2]",
        )
        .unwrap();
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "0");
    assert_eq!(
        session.eval_text("dmAssoc[a]").unwrap().value.to_string(),
        "List[1]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    session
        .eval_text(
            "x = 0; dmAssoc = DeleteMissing[Association[a :> {Missing[], x = x + 1}, b -> 2], 2, 1]",
        )
        .unwrap();
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "0");
    assert_eq!(
        session.eval_text("dmAssoc[a]").unwrap().value.to_string(),
        "List[1]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
}
