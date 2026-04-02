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
fn callable_functional_builtins_report_current_argument_forms() {
    let mut session = Session::new();

    let operator_applied = session.eval_text("OperatorApplied[f, bad]").unwrap();
    assert_eq!(
        operator_applied.value.to_string(),
        "OperatorApplied[f, bad]"
    );
    assert!(
        operator_applied
            .messages
            .iter()
            .any(|message| message.symbol == "OperatorApplied" && message.tag == "argform")
    );

    let curry_applied = session.eval_text("CurryApplied[f, bad]").unwrap();
    assert_eq!(curry_applied.value.to_string(), "CurryApplied[f, bad]");
    assert!(
        curry_applied
            .messages
            .iter()
            .any(|message| message.symbol == "CurryApplied" && message.tag == "argform")
    );

    let reverse_applied = session.eval_text("ReverseApplied[f, bad]").unwrap();
    assert_eq!(reverse_applied.value.to_string(), "ReverseApplied[f, bad]");
    assert!(
        reverse_applied
            .messages
            .iter()
            .any(|message| message.symbol == "ReverseApplied" && message.tag == "argform")
    );

    let operate = session.eval_text("Operate[f, expr, bad]").unwrap();
    assert_eq!(operate.value.to_string(), "Operate[f, expr, bad]");
    assert!(
        operate
            .messages
            .iter()
            .any(|message| message.symbol == "Operate" && message.tag == "argform")
    );
}
