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
