use super::*;

#[test]
fn sequence_functional_builtins_report_current_argument_forms() {
    let mut session = Session::new();

    let block_map = session.eval_text("BlockMap[f, expr, 0]").unwrap();
    assert_eq!(block_map.value.to_string(), "BlockMap[f, expr, 0]");
    assert!(
        block_map
            .messages
            .iter()
            .any(|message| message.symbol == "BlockMap" && message.tag == "argform")
    );

    let accumulate = session.eval_text("Accumulate[a, b]").unwrap();
    assert_eq!(accumulate.value.to_string(), "Accumulate[a, b]");
    assert!(
        accumulate
            .messages
            .iter()
            .any(|message| message.symbol == "Accumulate" && message.tag == "argx")
    );

    let take_while = session.eval_text("TakeWhile[a, b, c]").unwrap();
    assert_eq!(take_while.value.to_string(), "TakeWhile[a, b, c]");
    assert!(
        take_while
            .messages
            .iter()
            .any(|message| message.symbol == "TakeWhile" && message.tag == "argtu")
    );

    let length_while = session.eval_text("LengthWhile[a, b, c]").unwrap();
    assert_eq!(length_while.value.to_string(), "LengthWhile[a, b, c]");
    assert!(
        length_while
            .messages
            .iter()
            .any(|message| message.symbol == "LengthWhile" && message.tag == "argtu")
    );

    let select_limit = session.eval_text("Select[{1, 2, 3}, OddQ, All]").unwrap();
    assert_eq!(
        select_limit.value.to_string(),
        "Select[List[1, 2, 3], OddQ, All]"
    );
    assert!(
        select_limit
            .messages
            .iter()
            .any(|message| message.symbol == "Select" && message.tag == "argform")
    );

    let mut limited_session = Session::new();
    limited_session.set_iteration_work_limit(2);
    let limited = limited_session
        .eval_text("Select[{2, 4, 6}, OddQ]")
        .unwrap();
    assert_eq!(
        limited.value.to_string(),
        "TerminatedEvaluation[\"IterationWorkLimit\"]"
    );

    let fold_while = session.eval_text("FoldWhile[f, x, list, test, 0]").unwrap();
    assert_eq!(
        fold_while.value.to_string(),
        "FoldWhile[f, x, list, test, 0]"
    );
    assert!(
        fold_while
            .messages
            .iter()
            .any(|message| message.symbol == "FoldWhile" && message.tag == "argform")
    );

    let sequence_fold = session.eval_text("SequenceFold[f, {x}, {a}, 1]").unwrap();
    assert_eq!(
        sequence_fold.value.to_string(),
        "SequenceFold[f, List[x], List[a], 1]"
    );
    assert!(
        sequence_fold
            .messages
            .iter()
            .any(|message| message.symbol == "SequenceFold" && message.tag == "argform")
    );

    let fold = session.eval_text("Fold[f, expr, extra, boom]").unwrap();
    assert_eq!(fold.value.to_string(), "Fold[f, expr, extra, boom]");
    assert!(fold.messages.iter().any(|message| message.symbol == "Fold"));

    let fold_list = session.eval_text("FoldList[f, expr, extra, boom]").unwrap();
    assert_eq!(
        fold_list.value.to_string(),
        "FoldList[f, expr, extra, boom]"
    );
    assert!(
        fold_list
            .messages
            .iter()
            .any(|message| message.symbol == "FoldList")
    );
}
