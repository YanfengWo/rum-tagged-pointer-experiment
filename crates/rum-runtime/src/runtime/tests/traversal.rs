use super::*;

#[test]
fn level_supports_standard_levelspecs_and_depth_first_ordering() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2, h[3]], 1]")
            .unwrap()
            .value
            .to_string(),
        "List[g[1], 2, h[3]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2, h[3]], 2]")
            .unwrap()
            .value
            .to_string(),
        "List[1, g[1], 2, 3, h[3]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2, h[3]], {2}]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 3]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[1], 2, h[3]], -1]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, 3]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[], {-1}]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[]], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "List[g[]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[g[]], {-3}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[g[]]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[a], 0]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[a], {0}]")
            .unwrap()
            .value
            .to_string(),
        "List[f[a]]"
    );
    assert_eq!(
        session
            .eval_text("Level[{}, 0, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Level[{}, {0}]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
}

#[test]
fn count_memberq_and_freeq_share_standard_traversal_semantics() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Count[f[g[1], 2, h[3]], _Integer]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g[1], 2, h[3]], _Integer, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(
        session
            .eval_text("Count[f[1, g[2]], x_Symbol, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g[]], g[], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Count[f[g[]], f[g[]], {-2}]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[g[1]], _Integer]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[g[1]], _Integer, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[f[g[]], f[g[]], {-3}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[g[1]], _Integer]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[g[1]], _Integer, {1}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[g[]], g[], {-1}]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[f[1], f]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn level_supports_heads_and_apply_head_forms() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Level[f[1, g[2]], 1, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[f, 1, g[2]]"
    );
    assert_eq!(
        session
            .eval_text("Level[f[1, 2], {1}, Plus]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

#[test]
fn position_supports_standard_levelspecs_heads_and_limits() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Position[f[g[1], 2, h[3]], _Integer]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 1], List[2], List[3, 1]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1], f[x_Integer], 0, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "List[List[]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1, g[2]], x_Symbol]")
            .unwrap()
            .value
            .to_string(),
        "List[List[0], List[2, 0]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[g[1], 2, h[3]], _Integer, Infinity, 2]")
            .unwrap()
            .value
            .to_string(),
        "List[List[1, 1], List[2]]"
    );
    assert_eq!(
        session
            .eval_text("Position[f[1, g[2]], x_Symbol, Infinity, Heads -> False]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn traversal_uses_values_only_association_semantics_and_keyed_positions() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Level[Association[a -> 1, b -> f[2]], Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2, f[2]]"
    );
    assert_eq!(
        session
            .eval_text("Level[Association[a -> 1, b -> f[2]], Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Association, 1, f, 2, f[2]]"
    );
    assert_eq!(
        session
            .eval_text("Count[Association[a -> 1, b -> f[2]], _Symbol, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        session
            .eval_text("Count[Association[a -> 1, b -> f[2]], _Symbol, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("MemberQ[Association[a -> 1, b -> f[2]], _Symbol, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("FreeQ[Association[a -> 1, b -> f[2]], _Symbol, Infinity, Heads -> True]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
    assert_eq!(
        session
            .eval_text("Position[Association[a -> 1, b -> f[2]], _Integer, Infinity]")
            .unwrap()
            .value
            .to_string(),
        "List[List[Key[a]], List[Key[b], 1]]"
    );
}
