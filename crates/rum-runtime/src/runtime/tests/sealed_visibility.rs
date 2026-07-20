use super::*;

const VISIBILITY_PROBE: &str =
    "DefinePackage[\"Pkg`\", Public[{f}], f[x_] := helper[x] + 1; helper[x_] := x * 2]";

fn visibility_session(diagnostic: bool) -> Session {
    let mut session = session_with_test_control_builtins();
    let mut capabilities = vec![Capability::Pure, Capability::FileWrite];
    if diagnostic {
        capabilities.push(Capability::Diagnostic);
    }
    session.set_capability_policy(CapabilityPolicy::allow(capabilities));
    session
}

fn register_visibility_probe(session: &mut Session) -> Symbol {
    let outcome = session.eval_text(VISIBILITY_PROBE).unwrap();
    assert_eq!(outcome.value.to_string(), "\"Pkg`\"");
    assert!(outcome.messages.is_empty());

    session
        .sealed_units
        .iter()
        .find(|unit| unit.package_context == "Pkg`")
        .expect("visibility probe should be registered")
        .private_index["helper"]
}

fn eval_string_list(session: &mut Session, source: &str) -> Vec<String> {
    let outcome = session.eval_text(source).unwrap();
    assert!(
        outcome.messages.is_empty(),
        "{source}: {:?}",
        outcome.messages
    );
    expr_as_string_list(&outcome.value).unwrap_or_else(|| {
        panic!(
            "{source} should return a string list, got {}",
            outcome.value
        )
    })
}

fn inspect_held_symbol(session: &mut Session, head: &str, symbol: Symbol) -> Expr {
    session
        .evaluate(Expr::call(
            system_expr(head),
            vec![Expr::Atom(Atom::Symbol(symbol))],
        ))
        .unwrap()
}

fn register_message_probe(session: &mut Session) {
    let outcome = session
        .eval_text(
            "DefinePackage[\"MsgPkg`\", Public[{f}], helper::sealedVisibility270 = \"private template\"; f[] := helper]",
        )
        .unwrap();
    assert_eq!(outcome.value.to_string(), "\"MsgPkg`\"");
    assert!(outcome.messages.is_empty());
}

#[test]
fn names_hides_sealed() {
    let mut session = visibility_session(false);
    register_visibility_probe(&mut session);

    assert!(eval_string_list(&mut session, "Names[\"Pkg`Private`*\"]").is_empty());
    assert!(eval_string_list(&mut session, "Names[\"*helper*\"]").is_empty());
}

#[test]
fn names_reveals_with_diagnostic() {
    let mut session = visibility_session(true);
    register_visibility_probe(&mut session);

    assert_eq!(
        eval_string_list(&mut session, "Names[\"Pkg`Private`*\"]"),
        vec!["Pkg`Private`helper"]
    );
    assert_eq!(
        eval_string_list(&mut session, "Names[\"*helper*\"]"),
        vec!["Pkg`Private`helper"]
    );
}

#[test]
fn contexts_hides_private_context() {
    let mut session = visibility_session(false);
    register_visibility_probe(&mut session);

    assert!(
        !eval_string_list(&mut session, "Contexts[]")
            .iter()
            .any(|context| context == "Pkg`Private`")
    );
}

#[test]
fn contexts_reveals_with_diagnostic() {
    let mut session = visibility_session(true);
    register_visibility_probe(&mut session);

    assert!(
        eval_string_list(&mut session, "Contexts[]")
            .iter()
            .any(|context| context == "Pkg`Private`")
    );
}

#[test]
fn definition_hides_sealed_body() {
    let mut session = visibility_session(false);
    let sealed = register_visibility_probe(&mut session);
    let undefined = session
        .eval_text("Definition[\"Pkg`Private`helper\"]")
        .unwrap()
        .value;

    assert_eq!(undefined.to_string(), "HoldComplete[]");
    assert_eq!(
        inspect_held_symbol(&mut session, "Definition", sealed),
        undefined
    );
}

#[test]
fn definition_reveals_with_diagnostic() {
    let mut session = visibility_session(true);
    let sealed = register_visibility_probe(&mut session);
    let rendered = inspect_held_symbol(&mut session, "Definition", sealed).to_string();

    assert!(rendered.contains("Pkg`Private`helper"), "{rendered}");
    assert!(rendered.contains("Times[x, 2]"), "{rendered}");
    assert_eq!(
        session
            .eval_text("Definition[\"Pkg`Private`helper\"]")
            .unwrap()
            .value
            .to_string(),
        "HoldComplete[]"
    );
}

#[test]
fn information_hides_sealed() {
    let mut session = visibility_session(false);
    let sealed = register_visibility_probe(&mut session);
    let undefined = session
        .eval_text("Information[\"Pkg`Private`helper\"]")
        .unwrap()
        .value;

    assert_eq!(
        inspect_held_symbol(&mut session, "Information", sealed),
        undefined
    );
    assert!(!undefined.to_string().contains("Times[x, 2]"));
}

#[test]
fn information_reveals_with_diagnostic() {
    let mut session = visibility_session(true);
    let sealed = register_visibility_probe(&mut session);
    let rendered = inspect_held_symbol(&mut session, "Information", sealed).to_string();

    assert!(rendered.contains("Rule[\"FullName\", \"Pkg`Private`helper\"]"));
    assert!(rendered.contains("Times[x, 2]"), "{rendered}");
    assert!(
        !session
            .eval_text("Information[\"Pkg`Private`helper\"]")
            .unwrap()
            .value
            .to_string()
            .contains("Times[x, 2]")
    );
}

#[test]
fn completion_hides_sealed() {
    let mut session = visibility_session(false);
    register_visibility_probe(&mut session);

    let candidates = session.complete_symbol_prefix("Pkg`Private");
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate == "Pkg`Private`")
    );
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate == "Pkg`Private`helper")
    );
}

#[test]
fn completion_reveals_with_diagnostic() {
    let mut session = visibility_session(true);
    register_visibility_probe(&mut session);

    let candidates = session.complete_symbol_prefix("Pkg`Private");
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate == "Pkg`Private`")
    );
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate == "Pkg`Private`helper")
    );
}

#[test]
fn message_catalog_hides_sealed() {
    let mut session = visibility_session(false);
    register_message_probe(&mut session);

    let messages = session.eval_text("Messages[]").unwrap().value.to_string();
    let tags = eval_string_list(&mut session, "MessageTags[]");
    let definitions = session
        .eval_text("MessageDefinitions[]")
        .unwrap()
        .value
        .to_string();
    assert!(!messages.contains("MsgPkg`Private`helper"));
    assert!(!tags.iter().any(|tag| tag == "sealedVisibility270"));
    assert!(!definitions.contains("sealedVisibility270"));
}

#[test]
fn message_catalog_reveals_with_diagnostic() {
    let mut session = visibility_session(true);
    register_message_probe(&mut session);

    let messages = session.eval_text("Messages[]").unwrap().value.to_string();
    let tags = eval_string_list(&mut session, "MessageTags[]");
    let definitions = session
        .eval_text("MessageDefinitions[]")
        .unwrap()
        .value
        .to_string();
    assert!(messages.contains("MsgPkg`Private`helper::sealedVisibility270"));
    assert!(tags.iter().any(|tag| tag == "sealedVisibility270"));
    assert!(definitions.contains("MsgPkg`Private`helper::sealedVisibility270"));
    assert!(definitions.contains("private template"));
}

#[test]
fn save_excludes_sealed() {
    let dir = TempDirGuard::new("sealed-visibility-save");
    let save_file = dir.path().join("sealed-save.wl");
    let dump_file = dir.path().join("sealed-dump.mx");
    let mut session = visibility_session(false);
    register_visibility_probe(&mut session);

    let expected_snapshot = concat!(
        "DefinePackage[\"Pkg`\", Public[List[f]], CompoundExpression[SetDelayed[f[Pattern[x, Blank[]]], Plus[helper[x], 1]], SetDelayed[helper[Pattern[x, Blank[]]], Times[x, 2]]]]\n\n",
        "RestoreSealedReferences[Rule[\"Pkg`\", 1], HoldComplete[SetDelayed[Pkg`f[Pattern[x, Blank[]]], Plus[Pkg`Private`helper[x], 1]]]]\n",
    );
    let snapshot = session
        .snapshot_definitions_source()
        .expect("sealed snapshot should render");
    assert_eq!(snapshot, expected_snapshot);

    let save = session
        .eval_text(&format!(
            "Save[File[\"{}\"], \"Pkg`Private`\"]",
            rum_path(&save_file)
        ))
        .unwrap();
    assert_eq!(save.value.to_string(), "Null");
    assert!(!fs::read_to_string(&save_file).unwrap().contains("helper"));

    let dump = session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], \"Pkg`Private`\"]",
            rum_path(&dump_file)
        ))
        .unwrap();
    assert_eq!(dump.value.to_string(), "List[]");
    assert!(!fs::read_to_string(&dump_file).unwrap().contains("helper"));
    assert_eq!(session.snapshot_definitions_source().unwrap(), snapshot);

    session.set_capability_policy(CapabilityPolicy::allow([
        Capability::Pure,
        Capability::FileWrite,
        Capability::Diagnostic,
    ]));
    let revealed = session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], \"Pkg`Private`\"]",
            rum_path(&dump_file)
        ))
        .unwrap();
    assert_eq!(revealed.value.to_string(), "$Failed");
    assert!(
        revealed
            .messages
            .iter()
            .any(|message| message.symbol == "DumpSave" && message.tag == "sealedref")
    );
    assert!(
        !fs::read_to_string(&dump_file)
            .unwrap()
            .contains("Times[x, 2]")
    );
    assert_eq!(session.snapshot_definitions_source().unwrap(), snapshot);
}

#[test]
fn nameq_stays_false() {
    let mut session = visibility_session(false);
    register_visibility_probe(&mut session);

    assert_eq!(
        session
            .eval_text("NameQ[\"Pkg`Private`helper\"]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );
}

#[test]
fn wildcard_ops_skip_sealed() {
    let mut session = visibility_session(false);
    let sealed = register_visibility_probe(&mut session);

    session.eval_text("Clear[\"Pkg`Private`*\"]").unwrap();
    assert!(session.definitions.contains_key(&sealed));

    let mut attributes = session.attribute_set_for_symbol(&sealed);
    attributes.insert(Attribute::Protected);
    session.set_attribute_set_for_symbol(sealed, attributes);
    session.eval_text("Unprotect[\"Pkg`Private`*\"]").unwrap();
    assert!(session.symbol_is_protected(&sealed));
    assert!(session.definitions.contains_key(&sealed));
}

#[test]
fn nested_registration_keeps_registry_consistent() {
    let mut session = visibility_session(false);
    session
        .eval_text("DefinePackage[\"Shift`\", Public[{a}], a = 1]")
        .unwrap();
    session
        .eval_text("DefinePackage[\"Other`\", Public[{g}], g = 10]")
        .unwrap();
    session
        .eval_text("DefinePackage[\"Outer`\", Public[{f}], f = 100]")
        .unwrap();

    let outcome = session
        .eval_text(
            "DefinePackage[\"Outer`\", Public[{f}], DefinePackage[\"Other`\", Public[{g}], DefinePackage[\"Shift`\", Public[{a}], a = 2; Throw[\"shift\"]]; g = 20]; f = 200; Throw[\"outer\"]]",
        )
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "DefinePackage" && message.tag == "failed")
    );
    assert_eq!(session.sealed_units.len(), 1);
    assert_eq!(session.sealed_units[0].package_context, "Other`");
    assert_eq!(
        session.eval_text("Other`g").unwrap().value.to_string(),
        "20"
    );
}

#[test]
fn decoy_visibility_is_callers_own() {
    let mut session = visibility_session(false);
    register_visibility_probe(&mut session);
    session.eval_text("Pkg`Private`helper = 99").unwrap();

    assert_eq!(
        eval_string_list(&mut session, "Names[\"Pkg`Private`*\"]"),
        vec!["Pkg`Private`helper"]
    );
    assert_eq!(
        session.eval_text("Pkg`f[3]").unwrap().value.to_string(),
        "7"
    );
}
