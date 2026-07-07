use super::*;

#[test]
fn map_scan_and_comap_family_cover_current_first_pass_subset() {
    let mut session = core_session();

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
            .eval_text("Map[f][{a, b}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a], f[b]]"
    );
    assert_eq!(
        session
            .eval_text("Map[f, {a, b}]")
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
            .eval_text("Map[h, f[], {-1}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Map[h, f[], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "h[f[]]"
    );
    assert_eq!(
        session
            .eval_text("Map[h, f[g[]], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "f[h[g[]]]"
    );
    assert_eq!(
        session
            .eval_text("Map[h, f[g[]], {-3}]")
            .unwrap()
            .value
            .to_string(),
        "h[f[g[]]]"
    );
    assert_eq!(
        session
            .eval_text("MapIndexed[h, f[], {-1}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("MapIndexed[h, f[], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "h[f[], List[]]"
    );
    assert_eq!(
        session
            .eval_text("MapIndexed[h, f[g[]], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "f[h[g[], List[1]]]"
    );
    assert_eq!(
        session
            .eval_text("MapIndexed[h, f[g[]], {-3}]")
            .unwrap()
            .value
            .to_string(),
        "h[f[g[]], List[]]"
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
