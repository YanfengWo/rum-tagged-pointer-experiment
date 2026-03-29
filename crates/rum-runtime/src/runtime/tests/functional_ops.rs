use super::*;

#[test]
fn through_threads_arguments_through_current_head_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Through[{f, g}[x, y]]")
            .unwrap()
            .value
            .to_string(),
        "List[f[x, y], g[x, y]]"
    );
    assert_eq!(
        session
            .eval_text("Through[p[f[g, h], k][x], f]")
            .unwrap()
            .value
            .to_string(),
        "p[f[g[x], h[x]], k]"
    );
    assert_eq!(
        session
            .eval_text("Through[p[f, g][x], Plus]")
            .unwrap()
            .value
            .to_string(),
        "p[f, g][x]"
    );
    assert_eq!(
        session
            .eval_text("Through[(f + g)[x]]")
            .unwrap()
            .value
            .to_string(),
        "Plus[f[x], g[x]]"
    );
    assert_eq!(
        session
            .eval_text("Through[{Identity, Length}[{1, 2}]]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 2], 2]"
    );
    assert_eq!(
        session
            .eval_text("Through[p[f[Identity, Length], k][{1, 2}], f]")
            .unwrap()
            .value
            .to_string(),
        "p[f[List[1, 2], 2], k]"
    );
    assert_eq!(
        session
            .eval_text("Through[p[q[f, g], r][x], q]")
            .unwrap()
            .value
            .to_string(),
        "p[q[f[x], g[x]], r]"
    );
}

#[test]
fn operate_supports_head_chain_updates_and_operator_form() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Operate[q][f[x]]")
            .unwrap()
            .value
            .to_string(),
        "q[f][x]"
    );
    assert_eq!(
        session
            .eval_text("Operate[q, p[f][x], 2]")
            .unwrap()
            .value
            .to_string(),
        "q[p][f][x]"
    );
    assert_eq!(
        session
            .eval_text("Operate[q, f[x], 0]")
            .unwrap()
            .value
            .to_string(),
        "q[f[x]]"
    );
    assert_eq!(
        session
            .eval_text("Operate[q, f[x], 3]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("Operate[Identity, p[f][x], 2]")
            .unwrap()
            .value
            .to_string(),
        "p[f][x]"
    );
    assert_eq!(
        session
            .eval_text("Operate[q, 2][p[f][x]]")
            .unwrap()
            .value
            .to_string(),
        "q[p][f][x]"
    );
    assert_eq!(
        session
            .eval_text("Operate[q, 0][f[x]]")
            .unwrap()
            .value
            .to_string(),
        "q[f[x]]"
    );
    assert_eq!(
        session
            .eval_text("Operate[q, p[r[f]][x], 2]")
            .unwrap()
            .value
            .to_string(),
        "q[p][r[f]][x]"
    );
}

#[test]
fn apply_and_mapapply_cover_operator_forms_levelspecs_and_association_values() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Apply[f][{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b]"
    );
    assert_eq!(
        session
            .eval_text("Apply[f, Heads -> True][p[q][x]]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("MapApply[f][{{a, b}, {c, d}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b], f[c, d]]"
    );
    assert_eq!(
        session
            .eval_text("MapApply[f, Heads -> True][p[q][x]]")
            .unwrap()
            .value
            .to_string(),
        "f[q][x]"
    );
    assert_eq!(
        session.eval_text("Apply[f, x]").unwrap().value.to_string(),
        "x"
    );
    assert_eq!(
        session
            .eval_text("Apply[f, g[h[a, b]], {0, 1}]")
            .unwrap()
            .value
            .to_string(),
        "f[f[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("Apply[f, g[h[a, b], k[c]], -2]")
            .unwrap()
            .value
            .to_string(),
        "g[f[a, b], f[c]]"
    );
    assert_eq!(
        session
            .eval_text("Apply[f, p[q][x], 1, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "f[q][x]"
    );
    assert_eq!(
        session
            .eval_text("Apply[f, <|a -> 1, b -> 2|>]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("Apply[f, <|a -> g[1, 2], b -> h[3]|>, {1}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, f[1, 2]], Rule[b, f[3]]]"
    );
    assert_eq!(
        session
            .eval_text("MapApply[f, <|a -> {1, 2}, b -> {3, 4}|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, f[1, 2]], Rule[b, f[3, 4]]]"
    );
}

#[test]
fn map_scan_and_mapthread_cover_current_first_pass_subset() {
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
            .eval_text("MapIndexed[f, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, List[1]], f[b, List[2]]]"
    );
    assert_eq!(
        session
            .eval_text("MapAll[f, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "f[List[f[a], f[b]]]"
    );
    assert_eq!(
        session
            .eval_text("MapAll[f, <|a -> g[1]|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, f[g[f[1]]]]]"
    );
    assert_eq!(
        session
            .eval_text("MapAt[f, {{a, b}, {c, d}}, {All, 2}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, f[b]], List[c, f[d]]]"
    );
    assert_eq!(
        session
            .eval_text("MapAt[f, <|a -> 1, b -> <|c -> 2|>|>, {Key[b], Key[c]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, Association[Rule[c, f[2]]]]]"
    );
    assert_eq!(
        session
            .eval_text("Map[f][{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a], f[b]]"
    );
    assert_eq!(
        session
            .eval_text("Map[f, g[h[a, b]], {0, 1}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[f[h[a, b]]]]"
    );
    assert_eq!(
        session
            .eval_text("Map[f, p[q][x], 1, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "f[p[q]][f[x]]"
    );
    assert_eq!(
        session
            .eval_text("Map[f, <|a -> 1, b -> g[2]|>]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, f[1]], Rule[b, f[g[2]]]]"
    );

    session.eval_text("seen = {}").unwrap();
    assert_eq!(
        session
            .eval_text("Scan[Function[seen = Append[seen, Slot[1]]], f[g[1], 2, h[3]], 2]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(
        session.eval_text("seen").unwrap().value.to_string(),
        "List[1, g[1], 2, 3, h[3]]"
    );

    assert_eq!(
        session
            .eval_text("MapThread[f][{{a, b}, {c, d}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, c], f[b, d]]"
    );
    assert_eq!(
        session
            .eval_text("MapThread[f, {{{a, b}, {c, d}}, {{e, f}, {g, h}}}, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[a, e], f[b, f]], List[f[c, g], f[d, h]]]"
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
            .eval_text("Inner[Times, {1, 2}, {3, 4}]")
            .unwrap()
            .value
            .to_string(),
        "11"
    );
    assert_eq!(
        session
            .eval_text("Inner[Times, {{1, 2}, {3, 4}}, {{5, 6}, {7, 8}}, Plus]")
            .unwrap()
            .value
            .to_string(),
        "List[List[19, 22], List[43, 50]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, a, b]], f[List[a, b, c]], f[List[b, c, c]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {{a, b, c}, {d, e, f}, {g, h, i}}, {1, 1}, Padding -> None]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[List[List[a, b, c], List[d, e, f], List[g, h, i]]]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, {1, 0, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, Missing[], b]], f[List[a, Missing[], c]], f[List[b, Missing[], c]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, 1, Padding -> \"Periodic\"]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[c, a, b]], f[List[a, b, c]], f[List[b, c, a]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, 1, Padding -> {{left, right}}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[left, a, b]], f[List[a, b, c]], f[List[b, c, right]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c}, 2, Padding -> {left, right}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[right, left, a, b, c]], f[List[left, a, b, c, left]], f[List[a, b, c, left, right]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[f, {a, b, c, d}, {1, 0, 1}, Padding -> None]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, Missing[], c]], f[List[b, Missing[], d]]]"
    );
    assert_eq!(
        session
            .eval_text(
                "ArrayFilter[f, {{a, b, c}, {d, e, f}, {g, h, i}}, {1, 1}, Padding -> {{None, None}, {left, right}}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[f[List[List[left, a, b], List[left, d, e], List[left, g, h]]], f[List[List[a, b, c], List[d, e, f], List[g, h, i]]], f[List[List[b, c, right], List[e, f, right], List[h, i, right]]]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayFilter[Identity, {a, b, c, d}, {1, 1, 1, 1}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[a, a, a, b], List[a, a, b, c], List[a, b, c, d], List[b, c, d, d]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayReduce[f, {{a, b}, {c, d}}, 1]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, c]], f[List[b, d]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayReduce[f, {{{a, b}, {c, d}}, {{e, f}, {g, h}}}, 1 ;; 2]")
            .unwrap()
            .value
            .to_string(),
        "List[f[List[a, c, e, g]], f[List[b, d, f, h]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayReduce[f, {{{a, b}, {c, d}}, {{e, f}, {g, h}}}, {{1, 2}, {3}}]")
            .unwrap()
            .value
            .to_string(),
        "f[List[List[a, b], List[c, d], List[e, f], List[g, h]]]"
    );
    assert_eq!(
        session
            .eval_text("ArrayReduce[f, {{a, b}, {c, d}}, {}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[f[a], f[b]], List[f[c], f[d]]]"
    );
}

#[test]
fn fold_and_iteration_families_cover_current_iteration_subset() {
    let mut session = Session::new();

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
    assert_eq!(
        session
            .eval_text("Nest[f, x, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[f[x]]"
    );
    assert_eq!(
        session
            .eval_text("Nest[f, 2][x]")
            .unwrap()
            .value
            .to_string(),
        "f[f[x]]"
    );
    assert_eq!(
        session
            .eval_text("NestList[f, x, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[x, f[x], f[f[x]]]"
    );
    assert_eq!(
        session
            .eval_text("NestList[f, 2][x]")
            .unwrap()
            .value
            .to_string(),
        "List[x, f[x], f[f[x]]]"
    );
    session.eval_text("step[0] = 1").unwrap();
    session.eval_text("step[2] = 1").unwrap();
    session.eval_text("step[1] = 1").unwrap();
    session.eval_text("test1[2] = True").unwrap();
    session.eval_text("test1[1] = False").unwrap();

    assert_eq!(
        session
            .eval_text("NestWhile[step, 0, Function[Unequal[Slot[1], 1]]]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("NestWhileList[step, 2, UnsameQ, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, 1]"
    );
    assert_eq!(
        session
            .eval_text("NestWhileList[step, test1][2]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1]"
    );
    assert_eq!(
        session
            .eval_text("FixedPoint[Identity, f[x]]")
            .unwrap()
            .value
            .to_string(),
        "f[x]"
    );
    assert_eq!(
        session
            .eval_text("FixedPoint[step, 2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("FixedPoint[step][2]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("FixedPoint[step, 2, 0]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("FixedPointList[Identity, x]")
            .unwrap()
            .value
            .to_string(),
        "List[x, x]"
    );
    assert_eq!(
        session
            .eval_text("FixedPointList[step, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[2, 1, 1]"
    );
    assert_eq!(
        session
            .eval_text("FixedPointList[step, 2, 0]")
            .unwrap()
            .value
            .to_string(),
        "List[2]"
    );
}

#[test]
fn blockmap_subsetmap_and_comap_family_cover_current_first_pass_subset() {
    let mut session = Session::new();

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
            .eval_text(
                "SubsetMap[Function[List[Last[Slot[1]], First[Slot[1]]]], {a, b, c, d}, {2, 4}]"
            )
            .unwrap()
            .value
            .to_string(),
        "List[a, d, c, b]"
    );
    assert_eq!(
        session
            .eval_text("Comap[{f, g}][x]")
            .unwrap()
            .value
            .to_string(),
        "List[f[x], g[x]]"
    );
    assert_eq!(
        session
            .eval_text("Comap[p[f, g], x, 0]")
            .unwrap()
            .value
            .to_string(),
        "p[f, g][x]"
    );
    assert_eq!(
        session
            .eval_text("Comap[<|ka -> f, kb -> g|>, x]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[ka, f[x]], Rule[kb, g[x]]]"
    );
    assert_eq!(
        session
            .eval_text("ComapApply[{f, g}, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a, b], g[a, b]]"
    );
    assert_eq!(
        session
            .eval_text("ComapApply[<|ka -> f, kb -> g|>, {a, b}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[ka, f[a, b]], Rule[kb, g[a, b]]]"
    );
}

#[test]
fn operator_wrappers_cover_current_curried_subset() {
    let mut session = Session::new();

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
}

#[test]
fn functional_operator_builtins_report_current_argument_forms() {
    let mut session = Session::new();

    let map_indexed = session.eval_text("MapIndexed[f, expr, bad]").unwrap();
    assert_eq!(map_indexed.value.to_string(), "MapIndexed[f, expr, bad]");
    assert!(map_indexed
        .messages
        .iter()
        .any(|message| message.symbol == "MapIndexed" && message.tag == "argform"));

    let map = session.eval_text("Map[f, expr, bad]").unwrap();
    assert_eq!(map.value.to_string(), "Map[f, expr, bad]");
    assert!(map
        .messages
        .iter()
        .any(|message| message.symbol == "Map" && message.tag == "argform"));

    let map_all = session.eval_text("MapAll[f, expr, bad]").unwrap();
    assert_eq!(map_all.value.to_string(), "MapAll[f, expr, bad]");
    assert!(map_all
        .messages
        .iter()
        .any(|message| message.symbol == "MapAll" && message.tag == "argform"));

    let map_at = session.eval_text("MapAt[f, expr, bad]").unwrap();
    assert_eq!(map_at.value.to_string(), "MapAt[f, expr, bad]");
    assert!(map_at
        .messages
        .iter()
        .any(|message| message.symbol == "MapAt" && message.tag == "argform"));

    let scan = session.eval_text("Scan[f, expr, bad]").unwrap();
    assert_eq!(scan.value.to_string(), "Scan[f, expr, bad]");
    assert!(scan
        .messages
        .iter()
        .any(|message| message.symbol == "Scan" && message.tag == "argform"));

    let apply = session.eval_text("Apply[f, expr, bad]").unwrap();
    assert_eq!(apply.value.to_string(), "Apply[f, expr, bad]");
    assert!(apply
        .messages
        .iter()
        .any(|message| message.symbol == "Apply" && message.tag == "argform"));

    let map_apply = session.eval_text("MapApply[f, expr, bad]").unwrap();
    assert_eq!(map_apply.value.to_string(), "MapApply[f, expr, bad]");
    assert!(map_apply
        .messages
        .iter()
        .any(|message| message.symbol == "MapApply" && message.tag == "argform"));

    let map_thread = session.eval_text("MapThread[f, expr, 0]").unwrap();
    assert_eq!(map_thread.value.to_string(), "MapThread[f, expr, 0]");
    assert!(map_thread
        .messages
        .iter()
        .any(|message| message.symbol == "MapThread" && message.tag == "argform"));

    let block_map = session.eval_text("BlockMap[f, expr, 0]").unwrap();
    assert_eq!(block_map.value.to_string(), "BlockMap[f, expr, 0]");
    assert!(block_map
        .messages
        .iter()
        .any(|message| message.symbol == "BlockMap" && message.tag == "argform"));

    let subset_map = session.eval_text("SubsetMap[f, expr, bad]").unwrap();
    assert_eq!(subset_map.value.to_string(), "SubsetMap[f, expr, bad]");
    assert!(subset_map
        .messages
        .iter()
        .any(|message| message.symbol == "SubsetMap" && message.tag == "argform"));

    let comap = session.eval_text("Comap[fs, x, bad]").unwrap();
    assert_eq!(comap.value.to_string(), "Comap[fs, x, bad]");
    assert!(comap
        .messages
        .iter()
        .any(|message| message.symbol == "Comap" && message.tag == "argform"));

    let comap_apply = session.eval_text("ComapApply[fs, x, bad]").unwrap();
    assert_eq!(comap_apply.value.to_string(), "ComapApply[fs, x, bad]");
    assert!(comap_apply
        .messages
        .iter()
        .any(|message| message.symbol == "ComapApply" && message.tag == "argform"));

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

    let inner = session.eval_text("Inner[f, a]").unwrap();
    assert_eq!(inner.value.to_string(), "Inner[f, a]");
    assert!(inner
        .messages
        .iter()
        .any(|message| message.symbol == "Inner" && message.tag == "argtu"));

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

    let operator_applied = session.eval_text("OperatorApplied[f, bad]").unwrap();
    assert_eq!(
        operator_applied.value.to_string(),
        "OperatorApplied[f, bad]"
    );
    assert!(operator_applied
        .messages
        .iter()
        .any(|message| message.symbol == "OperatorApplied" && message.tag == "argform"));

    let curry_applied = session.eval_text("CurryApplied[f, bad]").unwrap();
    assert_eq!(curry_applied.value.to_string(), "CurryApplied[f, bad]");
    assert!(curry_applied
        .messages
        .iter()
        .any(|message| message.symbol == "CurryApplied" && message.tag == "argform"));

    let reverse_applied = session.eval_text("ReverseApplied[f, bad]").unwrap();
    assert_eq!(reverse_applied.value.to_string(), "ReverseApplied[f, bad]");
    assert!(reverse_applied
        .messages
        .iter()
        .any(|message| message.symbol == "ReverseApplied" && message.tag == "argform"));

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

    let nest = session.eval_text("Nest[f, expr, bad]").unwrap();
    assert_eq!(nest.value.to_string(), "Nest[f, expr, bad]");
    assert!(nest
        .messages
        .iter()
        .any(|message| message.symbol == "Nest" && message.tag == "argform"));

    let nest_while = session.eval_text("NestWhile[f, expr, test, 0]").unwrap();
    assert_eq!(nest_while.value.to_string(), "NestWhile[f, expr, test, 0]");
    assert!(nest_while
        .messages
        .iter()
        .any(|message| message.symbol == "NestWhile" && message.tag == "argform"));

    let fixed_point = session.eval_text("FixedPoint[f, expr, bad]").unwrap();
    assert_eq!(fixed_point.value.to_string(), "FixedPoint[f, expr, bad]");
    assert!(fixed_point
        .messages
        .iter()
        .any(|message| message.symbol == "FixedPoint" && message.tag == "argform"));

    let fixed_point_list = session.eval_text("FixedPointList[f, expr, bad]").unwrap();
    assert_eq!(
        fixed_point_list.value.to_string(),
        "FixedPointList[f, expr, bad]"
    );
    assert!(fixed_point_list
        .messages
        .iter()
        .any(|message| message.symbol == "FixedPointList" && message.tag == "argform"));

    let operate = session.eval_text("Operate[f, expr, bad]").unwrap();
    assert_eq!(operate.value.to_string(), "Operate[f, expr, bad]");
    assert!(operate
        .messages
        .iter()
        .any(|message| message.symbol == "Operate" && message.tag == "argform"));

    let array_filter = session.eval_text("ArrayFilter[f, {a, b}, bad]").unwrap();
    assert_eq!(
        array_filter.value.to_string(),
        "ArrayFilter[f, List[a, b], bad]"
    );
    assert!(array_filter
        .messages
        .iter()
        .any(|message| message.symbol == "ArrayFilter" && message.tag == "argform"));

    let array_reduce = session.eval_text("ArrayReduce[f, {a, b}, bad]").unwrap();
    assert_eq!(
        array_reduce.value.to_string(),
        "ArrayReduce[f, List[a, b], bad]"
    );
    assert!(array_reduce
        .messages
        .iter()
        .any(|message| message.symbol == "ArrayReduce" && message.tag == "argform"));

    let array_reduce_grouped = session
        .eval_text("ArrayReduce[f, {{a, b}, {c, d}}, {{1}, {1}}]")
        .unwrap();
    assert_eq!(
        array_reduce_grouped.value.to_string(),
        "ArrayReduce[f, List[List[a, b], List[c, d]], List[List[1], List[1]]]"
    );
    assert!(array_reduce_grouped
        .messages
        .iter()
        .any(|message| message.symbol == "ArrayReduce" && message.tag == "argform"));
}
