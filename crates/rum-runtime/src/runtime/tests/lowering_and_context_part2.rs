use super::*;

#[test]
fn get_rolls_back_loader_side_effects_and_collapses_failed_file_messages() {
    let dir = TempDirGuard::new("get-rollback");
    let file = dir.write(
        "broken.rum",
        "BeginPackage[\"Broken`\"]\nFindFile[\"Missing`\"]\nflag = 1\nSetAttributes[f, HoldFirst]\nself := self\nself",
    );
    let mut session = Session::new();

    let outcome = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "load")
    );
    assert!(
        !outcome
            .messages
            .iter()
            .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
    );
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[]"
    );
    assert_eq!(
        session.eval_text("Context[]").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session.eval_text("Broken`flag").unwrap().value.to_string(),
        "Broken`flag"
    );
    assert_eq!(
        session
            .eval_text("Attributes[Broken`f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn needs_loads_package_once_and_tracks_loaded_packages() {
    let dir = TempDirGuard::new("needs-once");
    dir.write(
        "Demo.rum",
        "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
    );
    let mut session = Session::new();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_text("Needs[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("token").unwrap().value.to_string(), "1");
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"Demo`\"]"
    );

    session.eval_text("Demo`token = 5").unwrap();
    session.eval_text("Needs[\"Demo`\"]").unwrap();

    assert_eq!(
        session.eval_text("Demo`token").unwrap().value.to_string(),
        "5"
    );
}

#[test]
fn findfile_resolves_package_names_and_file_wrappers() {
    let dir = TempDirGuard::new("findfile");
    let file = dir.write(
        "Demo.rum",
        "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
    );
    let mut session = Session::new();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_text("FindFile[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        Expr::string(file.display().to_string()).to_string()
    );
    assert_eq!(
        session
            .eval_text(&format!("FindFile[File[\"{}\"]]", rum_path(&file)))
            .unwrap()
            .value
            .to_string(),
        Expr::string(file.display().to_string()).to_string()
    );
    assert_eq!(
        session
            .eval_text("FindFile[\"Missing`\"]")
            .unwrap()
            .value
            .to_string(),
        "$Failed"
    );
}

#[test]
fn loader_builtins_accept_resolved_load_target_specs() {
    let dir = TempDirGuard::new("resolved-loader-specs");
    let package_file = dir.write(
        "Demo.rum",
        "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
    );
    let script_file = dir.write("load-target.rum", "value = 42\nvalue");
    let mut session = Session::new();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_script("spec = \"Demo`\"\nFindFile[spec]")
            .unwrap()
            .value
            .to_string(),
        Expr::string(package_file.display().to_string()).to_string()
    );
    assert_eq!(
        session
            .eval_script(&format!(
                "spec = File[\"{}\"]\nGet[spec]",
                rum_path(&script_file)
            ))
            .unwrap()
            .value
            .to_string(),
        "42"
    );
}

#[test]
fn missing_findfile_emits_runtime_message() {
    let mut session = Session::new();

    let outcome = session.eval_text("FindFile[\"Missing`\"]").unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
    );
}

#[test]
fn needs_supports_explicit_file_targets() {
    let dir = TempDirGuard::new("needs-file");
    let file = dir.write(
        "custom-package.rum",
        "BeginPackage[\"Custom`\"]\nflag = 9\nEndPackage[]",
    );
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text(&format!("Needs[\"Custom`\", \"{}\"]", rum_path(&file)))
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("flag").unwrap().value.to_string(), "9");

    session.eval_text("Custom`flag = 12").unwrap();
    session
        .eval_text(&format!(
            "Needs[\"Custom`\", File[\"{}\"]]",
            rum_path(&file)
        ))
        .unwrap();

    assert_eq!(
        session.eval_text("Custom`flag").unwrap().value.to_string(),
        "12"
    );
}

#[test]
fn needs_accepts_resolved_context_and_file_specs() {
    let dir = TempDirGuard::new("needs-resolved-specs");
    let file = dir.write(
        "custom-package.rum",
        "BeginPackage[\"Custom`\"]\nflag = 9\nEndPackage[]",
    );
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_script(&format!(
                "ctx = \"Custom`\"\nspec = File[\"{}\"]\nNeeds[ctx, spec]",
                rum_path(&file)
            ))
            .unwrap()
            .value
            .to_string(),
        "Null"
    );
    assert_eq!(session.eval_text("flag").unwrap().value.to_string(), "9");
}

#[test]
fn missing_needs_returns_failed_and_emits_messages() {
    let mut session = Session::new();

    let outcome = session.eval_text("Needs[\"Missing`\"]").unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "notfound")
    );
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Needs" && message.tag == "load")
    );
}
