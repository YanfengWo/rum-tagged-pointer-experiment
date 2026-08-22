use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn assert_eval(session: &mut Session, source: &str, expected: &str) {
    let outcome = session.eval_text(source).unwrap();
    assert_eq!(outcome.value.to_string(), expected);
}

#[test]
fn resumable_traversals_visit_each_completed_item_once() {
    let mut session = core_session();

    assert_eval(
        &mut session,
        "calls = 0; mapped = AssociationMap[(calls = calls + 1; #) &, Range[64]]; {Length[mapped], calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; selected = KeySelect[AssociationThread[Range[64], Range[64]], (calls = calls + 1; True) &]; {Length[selected], calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; taken = TakeWhile[Range[64], (calls = calls + 1; True) &]; {Length[taken], calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; length = LengthWhile[Range[64], (calls = calls + 1; True) &]; {length, calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; blocks = BlockMap[(calls = calls + 1; #) &, Range[64], 1]; {Length[blocks], calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "values = Accumulate[Range[64]]; {Length[values], Last[values]}",
        "List[64, 2080]",
    );
}
