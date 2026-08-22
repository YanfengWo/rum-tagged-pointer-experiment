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

#[test]
fn resumable_array_traversals_visit_each_generated_item_once() {
    let mut session = core_session();

    assert_eval(
        &mut session,
        "calls = 0; values = Array[(calls = calls + 1; #) &, 64]; {Length[values], calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; values = Outer[(calls = calls + 1; f[#1, #2]) &, Range[32], {x, y}]; {Length[values], Length[First[values]], calls}",
        "List[32, 2, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; values = ArrayReduce[(calls = calls + 1; #) &, Range[64], {}]; {Length[values], calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "terms = 0; reductions = 0; value = Inner[(terms = terms + 1; f[#1, #2]) &, Range[64], Range[64], (reductions = reductions + 1; h[#]) &]; {Head[value], terms, reductions}",
        "List[h, 64, 1]",
    );
}

#[test]
fn large_inner_callback_loop_does_not_recurse_through_the_rust_stack() {
    let mut session = core_session();

    assert_eval(
        &mut session,
        "Head[Inner[f, Range[5000], Range[5000], g]]",
        "g",
    );
}

#[test]
fn resumable_fold_traversals_visit_each_item_once() {
    let mut session = core_session();

    assert_eval(
        &mut session,
        "calls = 0; value = SequenceFold[(calls = calls + 1; #2) &, {0}, Range[64]]; {value, calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; values = SequenceFoldList[(calls = calls + 1; #2) &, {0}, Range[64]]; {Length[values], calls}",
        "List[65, 64]",
    );
    assert_eval(
        &mut session,
        "operators = 0; projectors = 0; value = FoldPair[(operators = operators + 1; {#2, #2}) &, 0, Range[64], (projectors = projectors + 1; Last[#]) &]; {value, operators, projectors}",
        "List[64, 64, 64]",
    );
    assert_eval(
        &mut session,
        "steps = 0; tests = 0; value = FoldWhile[(steps = steps + 1; #2) &, 0, Range[64], (tests = tests + 1; True) &]; {value, steps, tests}",
        "List[64, 64, 64]",
    );
}

#[test]
fn large_resumable_fold_loops_do_not_recurse_through_the_rust_stack() {
    let mut session = core_session();

    assert_eval(
        &mut session,
        "{SequenceFold[Function[Slot[2]], {0}, Range[5000]], FoldPair[Function[List[Slot[2], Slot[2]]], 0, Range[5000]], FoldWhile[Function[Slot[2]], 0, Range[5000], Function[True]]}",
        "List[5000, 5000, 5000]",
    );
}

#[test]
fn delayed_association_aggregates_evaluate_each_value_once() {
    let mut session = core_session();

    assert_eval(
        &mut session,
        "calls = 0; counts = Counts[Association[Table[RuleDelayed[i, (calls = calls + 1; 1)], {i, 1, 64}]]]; {Length[counts], calls}",
        "List[1, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; values = Catenate[Association[Table[RuleDelayed[i, {calls = calls + 1}], {i, 1, 64}]]]; {Length[values], calls}",
        "List[64, 64]",
    );
    assert_eval(
        &mut session,
        "calls = 0; values = Catenate[Table[Association[RuleDelayed[i, (calls = calls + 1)]], {i, 1, 64}]]; {Length[values], calls}",
        "List[64, 64]",
    );
}

#[test]
fn large_catenate_list_walk_does_not_recurse_through_the_rust_stack() {
    let mut session = core_session();

    assert_eval(
        &mut session,
        "Length[Catenate[Table[{i}, {i, 1, 5000}]]]",
        "5000",
    );
}
