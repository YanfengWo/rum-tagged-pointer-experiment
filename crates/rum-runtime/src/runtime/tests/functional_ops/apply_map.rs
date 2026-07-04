use super::*;

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
            .eval_text("Apply[h, f[], {-1}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Apply[h, f[], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "h[]"
    );
    assert_eq!(
        session
            .eval_text("Apply[h, f[g[]], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "f[h[]]"
    );
    assert_eq!(
        session
            .eval_text("Apply[h, f[g[]], {-3}]")
            .unwrap()
            .value
            .to_string(),
        "h[g[]]"
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
fn apply_map_family_reports_current_argument_forms() {
    let mut session = Session::new();

    let map_indexed = session.eval_text("MapIndexed[f, expr, bad]").unwrap();
    assert_eq!(map_indexed.value.to_string(), "MapIndexed[f, expr, bad]");
    assert!(
        map_indexed
            .messages
            .iter()
            .any(|message| message.symbol == "MapIndexed" && message.tag == "argform")
    );

    let map = session.eval_text("Map[f, expr, bad]").unwrap();
    assert_eq!(map.value.to_string(), "Map[f, expr, bad]");
    assert!(
        map.messages
            .iter()
            .any(|message| message.symbol == "Map" && message.tag == "argform")
    );

    let map_all = session.eval_text("MapAll[f, expr, bad]").unwrap();
    assert_eq!(map_all.value.to_string(), "MapAll[f, expr, bad]");
    assert!(
        map_all
            .messages
            .iter()
            .any(|message| message.symbol == "MapAll" && message.tag == "argform")
    );

    let scan = session.eval_text("Scan[f, expr, bad]").unwrap();
    assert_eq!(scan.value.to_string(), "Scan[f, expr, bad]");
    assert!(
        scan.messages
            .iter()
            .any(|message| message.symbol == "Scan" && message.tag == "argform")
    );

    let apply = session.eval_text("Apply[f, expr, bad]").unwrap();
    assert_eq!(apply.value.to_string(), "Apply[f, expr, bad]");
    assert!(
        apply
            .messages
            .iter()
            .any(|message| message.symbol == "Apply" && message.tag == "argform")
    );

    let map_apply = session.eval_text("MapApply[f, expr, bad]").unwrap();
    assert_eq!(map_apply.value.to_string(), "MapApply[f, expr, bad]");
    assert!(
        map_apply
            .messages
            .iter()
            .any(|message| message.symbol == "MapApply" && message.tag == "argform")
    );

    let comap = session.eval_text("Comap[fs, x, bad]").unwrap();
    assert_eq!(comap.value.to_string(), "Comap[fs, x, bad]");
    assert!(
        comap
            .messages
            .iter()
            .any(|message| message.symbol == "Comap" && message.tag == "argform")
    );

    let comap_apply = session.eval_text("ComapApply[fs, x, bad]").unwrap();
    assert_eq!(comap_apply.value.to_string(), "ComapApply[fs, x, bad]");
    assert!(
        comap_apply
            .messages
            .iter()
            .any(|message| message.symbol == "ComapApply" && message.tag == "argform")
    );
}
