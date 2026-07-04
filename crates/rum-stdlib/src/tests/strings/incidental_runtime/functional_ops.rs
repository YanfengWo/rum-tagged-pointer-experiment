use super::super::*;

// Moved from crates/rum-runtime/src/runtime/tests/functional_ops/callable.rs.

#[test]
fn operator_wrappers_cover_current_curried_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("OperatorApplied[f][y][x]")
            .unwrap()
            .value
            .to_string(),
        "f[x, y]"
    );
    assert_eq!(
        session
            .eval_text("OperatorApplied[f, 3][a][b, c]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("CurryApplied[3][f][a, b][c]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("CurryApplied[f, {2, 1, 2}][a][b]")
            .unwrap()
            .value
            .to_string(),
        "f[b, a, b]"
    );
    assert_eq!(
        session
            .eval_text("ReverseApplied[f][a, b, c]")
            .unwrap()
            .value
            .to_string(),
        "f[c, b, a]"
    );
    assert_eq!(
        session
            .eval_text("ReverseApplied[2][f][a, b, c]")
            .unwrap()
            .value
            .to_string(),
        "f[b, a, c]"
    );
    assert_eq!(
        session
            .eval_text("NearestTo[3][{1, 2, 5}]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("NearestTo[3, 2][{1, 2, 5}]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1]"
    );
    assert_eq!(
        session
            .eval_text("NearestTo[3, {All, 1}][{1, 2, 5}]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
    assert_eq!(
        session
            .eval_text("NearestTo[3, All][{1, 2, 5}]")
            .unwrap()
            .value
            .to_string(),
        "NearestTo[3, All][List[1, 2, 5]]"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/functional_ops/sequence.rs.

#[test]
fn accumulate_blockmap_takewhile_and_fold_family_cover_current_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Accumulate[{1, 2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 3, 6]"
    );
    assert_eq!(
        session
            .eval_text("Accumulate[<|a -> 1, b -> 2|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, 3]]"
    );
    assert_eq!(
        session
            .eval_text("BlockMap[Apply[Plus], {1, 2, 3, 4}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[3, 7]"
    );
    assert_eq!(
        session
            .eval_text("BlockMap[Identity, {a, b, c}, 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, b], List[b, c]]"
    );
    assert_eq!(
        session
            .eval_text("TakeWhile[Positive][{1, 2, -1, 3}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("TakeWhile[f[1, 2, -1, 3], Positive]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("LengthWhile[<|a -> 1, b -> 2, c -> -1|>, Positive]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("List[Context[Select], Select[{1, 2, 3}, OddQ], Select[{1, 2, 3}, OddQ, 1], Select[OddQ][{1, 2, 3}], Select[OddQ, 1][{1, 2, 3}]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"System`\", List[1, 3], List[1], List[1, 3], List[1]]"
    );
    assert_eq!(
        session
            .eval_text("Select[f[1, 2, 3], OddQ, 1]")
            .unwrap()
            .value
            .to_string(),
        "f[1]"
    );
    assert_eq!(
        session
            .eval_text("Select[<|a -> 1, b -> 2, c -> 3|>, OddQ]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[c, 3]]"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[{2, 4, 5, 7}, OddQ]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[{2, 4}, OddQ]")
            .unwrap()
            .value
            .to_string(),
        "Missing[\"NotFound\"]"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[{2, 4}, OddQ, fallback]")
            .unwrap()
            .value
            .to_string(),
        "fallback"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[OddQ][{2, 3, 4}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("SelectFirst[<|a -> 2, b -> 3|>, OddQ]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("libs = {{\"ErrorUtil.wl\"}, {\"Parser.wl\"}}; SelectFirst[Range[Length[libs]], StringContainsQ[Last[libs[[#]]], \"Parser\"] &, $Failed]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Fold[f][{a, b, c}]")
            .unwrap()
            .value
            .to_string(),
        "f[f[a, b], c]"
    );
    assert_eq!(
        session
            .eval_text("Fold[f, x, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "f[f[x, a], b]"
    );
    assert_eq!(
        session
            .eval_text("FoldList[f, x, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[x, f[x, a], f[f[x, a], b]]"
    );
    assert_eq!(
        session
            .eval_text(
                "FoldPair[Function[List[Plus[Slot[1], Slot[2]], Subtract[Slot[1], Slot[2]]]], 10, {1, 2}]"
            )
            .unwrap()
            .value
            .to_string(),
        "11"
    );
    assert_eq!(
        session
            .eval_text(
                "FoldPair[Function[List[Plus[Slot[1], Slot[2]], Subtract[Slot[1], Slot[2]]]], 10, {1, 2}, Last]"
            )
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("FoldWhile[Subtract, 10, {1, 2, 20}, Positive]")
            .unwrap()
            .value
            .to_string(),
        "-13"
    );
    assert_eq!(
        session
            .eval_text("FoldWhile[Subtract, Positive][10, {1, 2, 20}]")
            .unwrap()
            .value
            .to_string(),
        "-13"
    );
    assert_eq!(
        session
            .eval_text("SequenceFold[Plus, {1}, {2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "6"
    );
    assert_eq!(
        session
            .eval_text("SequenceFoldList[List, {a, b}, {c, d}]")
            .unwrap()
            .value
            .to_string(),
        "List[a, b, List[a, b, c], List[b, List[a, b, c], d]]"
    );
}
