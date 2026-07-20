use super::*;

const STATE_PACKAGE: &str = "DefinePackage[\"StatePkg`\", Public[{set, get}], state = 0; set[x_] := state = x; get[] := state]";
const VALUE_PACKAGE: &str = "DefinePackage[\"ValuePkg`\", Public[{mk, accepts, mutate}], SetAttributes[tag, Protected]; mk[x_] := tag[x]; accepts[tag[_]] := True; accepts[_] := False; mutate[] := (mk[x_] := tag[x + 1])]";

fn register(session: &mut Session, source: &str, context: &str) {
    let outcome = session.eval_text(source).unwrap();
    assert_eq!(outcome.value.to_string(), format!("\"{context}\""));
    assert!(outcome.messages.is_empty(), "{:?}", outcome.messages);
}

fn current_private(session: &Session, context: &str, name: &str) -> Symbol {
    session
        .sealed_units
        .iter()
        .find(|unit| unit.package_context == context)
        .unwrap_or_else(|| panic!("missing sealed unit {context}"))
        .private_index[name]
}

fn restore_snapshot(source: &Session) -> Session {
    let snapshot = source
        .snapshot_definitions_source()
        .expect("sealed snapshot should render");
    let mut restored = session_with_test_control_builtins();
    let response = restored
        .eval_snapshot_restore_request(&snapshot)
        .expect("sealed snapshot should restore");
    assert!(response.messages.is_empty(), "{:?}", response.messages);
    restored
}

fn assert_message(outcome: &EvalOutcome, symbol: &str, tag: &str) {
    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == tag),
        "expected {symbol}::{tag}, got {:?}",
        outcome.messages
    );
}

fn call_head_symbol(expr: &Expr) -> Symbol {
    let Expr::Call { head, .. } = expr else {
        panic!("expected a call, got {expr}")
    };
    expr_as_symbol(head)
        .copied()
        .unwrap_or_else(|| panic!("expected a symbol head, got {head}"))
}

#[test]
fn snapshot_roundtrip_restores_seal() {
    let mut source = session_with_test_control_builtins();
    register(&mut source, STATE_PACKAGE, "StatePkg`");
    assert_eq!(
        source
            .eval_text("StatePkg`set[42]")
            .unwrap()
            .value
            .to_string(),
        "42"
    );
    assert_eq!(
        source
            .eval_text("StatePkg`get[]")
            .unwrap()
            .value
            .to_string(),
        "42"
    );

    let mut restored = restore_snapshot(&source);
    let after = current_private(&restored, "StatePkg`", "state");
    // The restored session mints its own sealed identities; numeric reuse
    // across sessions is permitted (identities are session-scoped), so only
    // validity is asserted, not distinctness from the pre-restore identity.
    assert!(crate::runtime::scoping::packages::symbol_is_sealed_private(
        &after
    ));
    assert_eq!(
        restored
            .eval_text("StatePkg`get[]")
            .unwrap()
            .value
            .to_string(),
        "0"
    );
    assert_eq!(
        restored
            .eval_text("StatePkg`set[9]; StatePkg`get[]")
            .unwrap()
            .value
            .to_string(),
        "9"
    );
    let decoy = restored
        .resolve_symbol_name("StatePkg`Private`state", true)
        .expect("spelled private symbol should resolve to a decoy");
    assert_eq!(decoy.identity(), None);
    assert_ne!(decoy, after);
}

#[test]
fn snapshot_roundtrip_relinks_leaked_values() {
    let mut source = session_with_test_control_builtins();
    register(&mut source, VALUE_PACKAGE, "ValuePkg`");
    source.eval_text("thm = ValuePkg`mk[7]").unwrap();

    let mut restored = restore_snapshot(&source);
    let current_tag = current_private(&restored, "ValuePkg`", "tag");
    let thm = restored.eval_text("thm").unwrap().value;
    assert_eq!(call_head_symbol(&thm), current_tag);
    assert_eq!(
        restored
            .eval_text("ValuePkg`accepts[thm]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_ne!(
        call_head_symbol(&thm),
        Symbol::new("ValuePkg`Private`", "tag")
    );
}

#[test]
fn snapshot_persists_public_mutation() {
    let mut source = session_with_test_control_builtins();
    register(&mut source, VALUE_PACKAGE, "ValuePkg`");
    assert_eq!(
        source
            .eval_text("ValuePkg`mutate[]; ValuePkg`mk[7]")
            .unwrap()
            .value
            .to_string(),
        "ValuePkg`Private`tag[8]"
    );

    let mut restored = restore_snapshot(&source);
    let value = restored.eval_text("ValuePkg`mk[7]").unwrap().value;
    assert_eq!(value.to_string(), "ValuePkg`Private`tag[8]");
    assert_eq!(
        call_head_symbol(&value),
        current_private(&restored, "ValuePkg`", "tag")
    );
    assert_eq!(
        restored
            .eval_text("ValuePkg`accepts[ValuePkg`mk[7]]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
}

#[test]
fn snapshot_refuses_orphans() {
    let mut session = session_with_test_control_builtins();
    register(&mut session, VALUE_PACKAGE, "ValuePkg`");
    session.eval_text("thm = ValuePkg`mk[7]").unwrap();
    let stale = current_private(&session, "ValuePkg`", "tag");
    register(&mut session, VALUE_PACKAGE, "ValuePkg`");
    assert_ne!(stale, current_private(&session, "ValuePkg`", "tag"));

    let error = session
        .snapshot_definitions_source()
        .expect_err("stale sealed identity should refuse snapshot rendering");
    assert!(error.contains("ValuePkg`Private`tag"), "{error}");
    assert!(error.contains("owner key ValuePkg`"), "{error}");
}

#[test]
fn marker_refused_in_ordinary_evaluation() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("side = 0").unwrap();
    assert!(
        session
            .eval_snapshot_restore_request("Throw[\"restore failed\"]")
            .is_err()
    );
    assert!(!session.restoring_snapshot);

    let outcome = session
        .eval_text("RestoreSealedReferences[\"Missing`\", HoldComplete[side = 1]]")
        .unwrap();
    assert_message(&outcome, "RestoreSealedReferences", "mode");
    assert_eq!(session.eval_text("side").unwrap().value.to_string(), "0");

    let response = session
        .eval_snapshot_restore_request(
            "RestoreSealedReferences[\"Missing`\", HoldComplete[side = 2]]",
        )
        .unwrap();
    assert_eq!(response.value.to_string(), "$Failed");
    assert!(
        response
            .messages
            .iter()
            .any(|message| message.symbol == "RestoreSealedReferences" && message.tag == "unit")
    );
    assert!(!session.restoring_snapshot);
    assert_eq!(session.eval_text("side").unwrap().value.to_string(), "0");
}

#[test]
fn sealed_free_snapshot_bytes_unchanged() {
    let mut session = session_with_test_control_builtins();
    session
        .eval_text("a = 1; f[x_] := x + 1; SetAttributes[f, Orderless]")
        .unwrap();

    let expected = concat!(
        "Set[a, 1]\n\n",
        "SetAttributes[f, List[Orderless]]\n\n",
        "SetDelayed[f[Pattern[x, Blank[]]], Plus[x, 1]]\n",
    );
    assert_eq!(
        session
            .snapshot_definitions_source()
            .expect("sealed-free snapshot should render"),
        expected
    );
}

#[test]
fn save_refuses_sealed_references() {
    let dir = TempDirGuard::new("sealed-snapshot-save-refusal");
    let save_path = dir.path().join("sealed-save.wl");
    let dump_path = dir.path().join("sealed-dump.mx");
    let mut session = session_with_test_control_builtins();
    register(&mut session, VALUE_PACKAGE, "ValuePkg`");
    session.eval_text("thm = ValuePkg`mk[7]").unwrap();

    let save = session
        .eval_text(&format!("Save[File[\"{}\"], thm]", rum_path(&save_path)))
        .unwrap();
    assert_message(&save, "Save", "sealedref");
    assert!(!save_path.exists());

    let dump = session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], thm]",
            rum_path(&dump_path)
        ))
        .unwrap();
    assert_message(&dump, "DumpSave", "sealedref");
    assert!(!dump_path.exists());
}

#[test]
fn information_sealed_unit_row() {
    let mut session = session_with_test_control_builtins();
    register(&mut session, VALUE_PACKAGE, "ValuePkg`");
    let sealed = current_private(&session, "ValuePkg`", "tag");
    let information = |session: &mut Session| {
        session
            .evaluate(Expr::call(
                system_expr("Information"),
                vec![Expr::Atom(Atom::Symbol(sealed))],
            ))
            .unwrap()
            .to_string()
    };

    session.set_capability_policy(CapabilityPolicy::allow([
        Capability::Pure,
        Capability::Diagnostic,
    ]));
    let visible = information(&mut session);
    assert!(
        visible.contains("Rule[\"SealedUnit\", \"ValuePkg`\"]"),
        "{visible}"
    );

    session.set_capability_policy(CapabilityPolicy::allow([Capability::Pure]));
    let hidden = information(&mut session);
    assert!(!hidden.contains("SealedUnit"), "{hidden}");
}
