use super::*;

#[test]
fn part_supports_exact_indices_nested_paths_and_head_access() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 2]")
            .unwrap()
            .value
            .to_string(),
        "b"
    );
    assert_eq!(
        session
            .eval_text("Part[f[g[a], h[b]], 2, 1]")
            .unwrap()
            .value
            .to_string(),
        "b"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {1, -1}]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 0]")
            .unwrap()
            .value
            .to_string(),
        "f"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], {}]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[g[a, b], h[c, d]], All, 2]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c, d], Span[2, All, 2]]")
            .unwrap()
            .value
            .to_string(),
        "f[b, d]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {All}]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], List[All]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {1 ;; 2}]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], List[Span[1, 2]]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {Span[1, All, 2], -1}]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], List[Span[1, All, 2], -1]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 0 ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], {0, 1, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[f, a, b]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 1 ;; 0]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 2 ;; 1]")
            .unwrap()
            .value
            .to_string(),
        "f[]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 0 ;; 0]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b], Span[0, 0]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], All ;; 1 ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[a]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b], 2 ;; All ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[b]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], All ;; All ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], Span[All, All, -1]]"
    );
    assert_eq!(
        session.eval_text("Part[1, All]").unwrap().value.to_string(),
        "Integer[]"
    );
    assert_eq!(
        session
            .eval_text("Part[\"a\", All]")
            .unwrap()
            .value
            .to_string(),
        "String[]"
    );
    assert_eq!(
        session.eval_text("Part[x, All]").unwrap().value.to_string(),
        "Symbol[]"
    );
    assert_eq!(
        session
            .eval_text("Part[x, 0 ;; 0]")
            .unwrap()
            .value
            .to_string(),
        "Symbol[]"
    );
    assert_eq!(
        session
            .eval_text("Part[1, 1 ;; All]")
            .unwrap()
            .value
            .to_string(),
        "Integer[]"
    );
    assert_eq!(
        session
            .eval_text("f[g[a, b], h[c, d]][[All, 2 ;; ;; 2]]")
            .unwrap()
            .value
            .to_string(),
        "f[g[b], h[d]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], {{1, 3}, {2}}]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], List[List[1, 3], List[2]]]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 1 ;; UpTo[5]]")
            .unwrap()
            .value
            .to_string(),
        "f[a, b, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], UpTo[2] ;; All]")
            .unwrap()
            .value
            .to_string(),
        "f[b, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 3 ;; UpTo[1] ;; -1]")
            .unwrap()
            .value
            .to_string(),
        "f[c, b, a]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], 1 ;; UpTo[Infinity] ;; 2]")
            .unwrap()
            .value
            .to_string(),
        "f[a, c]"
    );
    assert_eq!(
        session
            .eval_text("Part[f[a, b, c], UpTo[2]]")
            .unwrap()
            .value
            .to_string(),
        "Part[f[a, b, c], UpTo[2]]"
    );
}
