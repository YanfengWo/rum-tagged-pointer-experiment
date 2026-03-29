use super::*;

#[test]
fn accumulate_blockmap_takewhile_and_fold_family_cover_current_subset() {
    let mut session = Session::new();

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

#[test]
fn sequence_functional_builtins_report_current_argument_forms() {
    let mut session = Session::new();

    let block_map = session.eval_text("BlockMap[f, expr, 0]").unwrap();
    assert_eq!(block_map.value.to_string(), "BlockMap[f, expr, 0]");
    assert!(block_map
        .messages
        .iter()
        .any(|message| message.symbol == "BlockMap" && message.tag == "argform"));

    let accumulate = session.eval_text("Accumulate[a, b]").unwrap();
    assert_eq!(accumulate.value.to_string(), "Accumulate[a, b]");
    assert!(accumulate
        .messages
        .iter()
        .any(|message| message.symbol == "Accumulate" && message.tag == "argx"));

    let take_while = session.eval_text("TakeWhile[a, b, c]").unwrap();
    assert_eq!(take_while.value.to_string(), "TakeWhile[a, b, c]");
    assert!(take_while
        .messages
        .iter()
        .any(|message| message.symbol == "TakeWhile" && message.tag == "argtu"));

    let length_while = session.eval_text("LengthWhile[a, b, c]").unwrap();
    assert_eq!(length_while.value.to_string(), "LengthWhile[a, b, c]");
    assert!(length_while
        .messages
        .iter()
        .any(|message| message.symbol == "LengthWhile" && message.tag == "argtu"));

    let fold_while = session.eval_text("FoldWhile[f, x, list, test, 0]").unwrap();
    assert_eq!(
        fold_while.value.to_string(),
        "FoldWhile[f, x, list, test, 0]"
    );
    assert!(fold_while
        .messages
        .iter()
        .any(|message| message.symbol == "FoldWhile" && message.tag == "argform"));

    let sequence_fold = session.eval_text("SequenceFold[f, {x}, {a}, 1]").unwrap();
    assert_eq!(
        sequence_fold.value.to_string(),
        "SequenceFold[f, List[x], List[a], 1]"
    );
    assert!(sequence_fold
        .messages
        .iter()
        .any(|message| message.symbol == "SequenceFold" && message.tag == "argform"));

    let fold = session.eval_text("Fold[f, expr, extra, boom]").unwrap();
    assert_eq!(fold.value.to_string(), "Fold[f, expr, extra, boom]");
    assert!(fold.messages.iter().any(|message| message.symbol == "Fold"));

    let fold_list = session.eval_text("FoldList[f, expr, extra, boom]").unwrap();
    assert_eq!(
        fold_list.value.to_string(),
        "FoldList[f, expr, extra, boom]"
    );
    assert!(fold_list
        .messages
        .iter()
        .any(|message| message.symbol == "FoldList"));
}
