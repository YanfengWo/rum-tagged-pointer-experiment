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
fn replacepart_rhs_guards_cover_keyed_association_position_forms() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "{ReplacePart[Association[\"k\" -> 1], Key[\"k\"] -> (\"v\" /; True)], \
                  ReplacePart[Association[\"k\" -> 1], {Key[\"k\"]} :> (\"v\" /; False)], \
                  ReplacePart[Association[\"outer\" -> Association[\"k\" -> 1]], \
                    {Key[\"outer\"], Key[\"k\"]} :> (\"nested\" /; True)], \
                  ReplacePart[Association[\"k\" -> 1, \"j\" -> 2], \
                    {{Key[\"k\"]}, {Key[\"j\"]}} -> (\"both\" /; True)]}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[\"k\", \"v\"]], Association[Rule[\"k\", 1]], Association[Rule[\"outer\", Association[Rule[\"k\", \"nested\"]]]], Association[Rule[\"k\", \"both\"], Rule[\"j\", \"both\"]]]"
    );
}

#[test]
fn replacepart_bare_association_keys_remain_nonselecting_position_patterns() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text(
                "bareKeyGuardRuns = 0; \
                 result = ReplacePart[Association[\"k\" -> 1], \
                   \"k\" :> (\"v\" /; (bareKeyGuardRuns = bareKeyGuardRuns + 1; True))]; \
                 {result, bareKeyGuardRuns}",
            )
            .unwrap()
            .value
            .to_string(),
        "List[Association[Rule[\"k\", 1]], 0]"
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
