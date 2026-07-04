use super::*;

#[test]
fn through_threads_arguments_through_current_head_subset() {
    let mut session = core_session();

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
