use super::*;

#[test]
fn check_can_catch_iteration_limit_messages_from_replace_repeated() {
    let mut session = Session::new();
    session.set_replace_repeated_limit(8);

    let outcome = session
        .eval_text("Check[1 //. {1 -> 2, 2 -> 1}, 99, \"General::iterlim\"]")
        .unwrap();

    assert_eq!(outcome.value.to_string(), "99");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| { message.symbol == "General" && message.tag == "iterlim" })
    );
}

#[test]
fn beginpackage_can_load_dependencies_via_needs() {
    let dir = TempDirGuard::new("needs-deps");
    dir.write(
        "Dep.rum",
        "BeginPackage[\"Dep`\"]\ndepValue = 7\nEndPackage[]",
    );
    dir.write(
        "Main.rum",
        "BeginPackage[\"Main`\", {\"Dep`\"}]\nvalue = Dep`depValue\nEndPackage[]",
    );
    let mut session = Session::new();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    session.eval_text("Needs[\"Main`\"]").unwrap();

    assert_eq!(session.eval_text("value").unwrap().value.to_string(), "7");
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"Main`\", \"Dep`\"]"
    );
}

#[test]
fn beginpackage_accepts_resolved_dependency_lists() {
    let dir = TempDirGuard::new("needs-resolved-deps");
    dir.write(
        "Dep.rum",
        "BeginPackage[\"Dep`\"]\ndepValue = 7\nEndPackage[]",
    );
    dir.write(
        "Main.rum",
        "deps = {\"Dep`\"}\nBeginPackage[\"Main`\", deps]\nvalue = Dep`depValue\nEndPackage[]",
    );
    let mut session = Session::new();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    session.eval_text("Needs[\"Main`\"]").unwrap();

    assert_eq!(session.eval_text("value").unwrap().value.to_string(), "7");
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"Main`\", \"Dep`\"]"
    );
}

#[test]
fn path_builtin_reflects_package_search_roots() {
    let dir = TempDirGuard::new("path");
    let mut session = Session::new();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session.eval_text("$Path").unwrap().value.to_string(),
        format!("List[{}]", Expr::string(dir.path().display().to_string()))
    );
}

#[test]
fn explicit_non_system_heads_do_not_trigger_builtin_dispatch() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Foo`If[True, Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "Foo`If[True, 1, 2]"
    );
}

#[test]
fn hold_family_respects_builtin_hold_attributes() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("Hold[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "Hold[Identity[1], Identity[2]]"
    );
    assert_eq!(
        session
            .eval_text("HoldFirst[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "HoldFirst[Identity[1], 2]"
    );
    assert_eq!(
        session
            .eval_text("HoldRest[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "HoldRest[1, Identity[2]]"
    );
    assert_eq!(
        session
            .eval_text("HoldAll[Identity[1], Identity[2]]")
            .unwrap()
            .value
            .to_string(),
        "HoldAll[Identity[1], Identity[2]]"
    );
}
