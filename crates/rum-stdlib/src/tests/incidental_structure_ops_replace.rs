use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn replacepart_supports_keyed_association_paths() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "ReplacePart[Association[a -> 1, b -> Association[c -> 3]], {Key[b], Key[c]} -> 9]"
            )
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, Association[Rule[c, 9]]]]"
    );
}

#[test]
fn replaceat_supports_sequential_paths_all_and_keyed_positions() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("ReplaceAt[f[a], x_ :> h[x], {{1}, {1}}]")
            .unwrap()
            .value
            .to_string(),
        "f[h[h[a]]]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceAt[f[g[a, b], h[c, d]], x_ :> q[x], {All, 2}]")
            .unwrap()
            .value
            .to_string(),
        "f[g[a, q[b]], h[c, q[d]]]"
    );
    assert_eq!(
        session
            .eval_text("ReplaceAt[Association[a -> 1, b -> Association[c -> 3]], _Integer -> z, {Key[b], Key[c]}]")
            .unwrap()
            .value
            .to_string(),
        "Association[Rule[a, 1], Rule[b, Association[Rule[c, z]]]]"
    );
}
