use super::*;

#[test]
fn context_variables_and_builtin_inspection_reflect_session_contexts() {
    let mut session = session_with_test_control_builtins();
    session.set_current_context("Rules");

    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"System`\", \"Global`\"]"
    );

    let info = session.inspect("Identity").unwrap();
    assert_eq!(info.qualified_name, "System`Identity");
    assert_eq!(info.context, "System`");
    assert_eq!(info.display_name, "Identity");
    assert!(info.is_builtin);
}

#[test]
fn context_path_and_packages_assignments_update_session_fields() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("Rules`x = 10").unwrap();
    session.eval_text("Global`x = 20").unwrap();

    assert_eq!(
        session
            .eval_text("$ContextPath = {\"Rules\", \"System`\", \"Rules`\"}")
            .unwrap()
            .value
            .to_string(),
        "List[\"Rules\", \"System`\", \"Rules`\"]"
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Rules`\", \"System`\"]"
    );
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");

    session
        .eval_text("$Packages = {\"Pkg\", \"Pkg`\", \"Other`\"}")
        .unwrap();
    assert_eq!(
        session.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"Pkg`\", \"Other`\"]"
    );
}

#[test]
fn explicitly_protected_session_configuration_variables_reject_assignments() {
    let mut session = session_with_test_control_builtins();

    let context_path = session.eval_text("$ContextPath").unwrap().value.to_string();
    let packages = session.eval_text("$Packages").unwrap().value.to_string();
    let path = session.eval_text("$Path").unwrap().value.to_string();

    session
        .eval_text("Protect[$ContextPath, $Packages, $Path]")
        .unwrap();

    for (source, target, expected) in [
        (
            "$ContextPath = {\"System`\", \"Leak`\"}",
            "$ContextPath",
            context_path,
        ),
        ("$Packages = {\"Leak`\"}", "$Packages", packages),
        ("$Path = {\"/tmp/rum-protected-path\"}", "$Path", path),
    ] {
        let outcome = session.eval_text(source).unwrap();
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "Set" && message.tag == "wrsym"),
            "{source}: {:?}",
            outcome.messages
        );
        assert_eq!(
            session.eval_text(target).unwrap().value.to_string(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn path_assignment_remains_writable_without_explicit_protect() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Attributes[$Path]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    let outcome = session
        .eval_text("$Path = {\"/tmp/rum-a\", \"/tmp/rum-a\", \"/tmp/rum-b\"}")
        .unwrap();
    assert!(outcome.messages.is_empty());
    assert_eq!(
        session.eval_text("$Path").unwrap().value.to_string(),
        "List[\"/tmp/rum-a\", \"/tmp/rum-b\"]"
    );
}

#[test]
fn bare_symbols_search_context_path_before_current_context() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("Global`x = 10").unwrap();
    session.set_current_context("Rules");
    session.eval_text("Rules`x = 20").unwrap();
    session.eval_text("local = 30").unwrap();

    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");

    let info = session.inspect("local").unwrap();
    assert_eq!(info.qualified_name, "Rules`local");
    assert_eq!(info.context, "Rules`");
    assert_eq!(
        session.eval_text("Rules`x").unwrap().value.to_string(),
        "20"
    );
}

#[test]
fn context_builtin_reports_session_and_symbol_contexts_without_evaluating_symbols() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("x = 1").unwrap();

    assert_eq!(
        session.eval_text("Context[]").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session.eval_text("Context[x]").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session
            .eval_text("Context[Identity]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Context[\"Identity\"]")
            .unwrap()
            .value
            .to_string(),
        "\"System`\""
    );
    assert_eq!(
        session
            .eval_text("Context[\"MissingSymbol\"]")
            .unwrap()
            .value
            .to_string(),
        "Context[\"MissingSymbol\"]"
    );
}

#[test]
fn contexts_lists_known_session_contexts() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("BeginPackage[\"Demo`\"]").unwrap();
    session.eval_text("public = 1").unwrap();
    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("hidden = 2").unwrap();
    session.eval_text("End[]").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    let contexts = session.eval_text("Contexts[]").unwrap().value.to_string();
    assert!(contexts.contains("\"System`\""));
    assert!(contexts.contains("\"Global`\""));
    assert!(contexts.contains("\"Demo`\""));
    assert!(contexts.contains("\"Demo`Private`\""));
}

#[test]
fn begin_and_end_balance_context_and_context_path() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("Begin[\"Rules`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Rules`\", \"System`\", \"Global`\"]"
    );

    assert_eq!(
        session
            .eval_text("Begin[\"`Private`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("End[]").unwrap().value.to_string(),
        "\"Rules`Private`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Rules`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Rules`\", \"System`\", \"Global`\"]"
    );
}

#[test]
fn begin_package_and_end_package_restore_outer_context_and_export_package_context() {
    let mut session = session_with_test_control_builtins();

    assert_eq!(
        session
            .eval_text("BeginPackage[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Demo`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Demo`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Demo`\", \"System`\"]"
    );

    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("hidden = 1").unwrap();
    assert_eq!(
        session.eval_text("End[]").unwrap().value.to_string(),
        "\"Demo`Private`\""
    );

    session.eval_text("public = 2").unwrap();

    assert_eq!(
        session.eval_text("EndPackage[]").unwrap().value.to_string(),
        "Null"
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Global`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Demo`\", \"System`\", \"Global`\"]"
    );
    assert_eq!(session.eval_text("public").unwrap().value.to_string(), "2");
    assert_eq!(
        session.eval_text("hidden").unwrap().value.to_string(),
        "hidden"
    );
}

#[test]
fn begin_package_usage_messages_make_public_symbols_visible_from_private_context() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("BeginPackage[\"Demo`\"]").unwrap();
    session.eval_text("foo::usage = \"foo[x]\"").unwrap();
    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("foo[x_] := x").unwrap();
    session.eval_text("End[]").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    assert_eq!(
        session.eval_text("Demo`foo[7]").unwrap().value.to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("Demo`Private`foo[7]")
            .unwrap()
            .value
            .to_string(),
        "Demo`Private`foo[7]"
    );
}

#[test]
fn begin_package_dependencies_stay_on_private_context_path_when_already_loaded() {
    let mut session = session_with_test_control_builtins();

    session.eval_text("BeginPackage[\"DepA`\"]").unwrap();
    session
        .eval_text("depHead::usage = \"depHead[x]\"")
        .unwrap();
    session.eval_text("EndPackage[]").unwrap();
    session.eval_text("BeginPackage[\"DepB`\"]").unwrap();
    session.eval_text("depValue::usage = \"depValue\"").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    session
        .eval_text("BeginPackage[\"Main`\", {\"DepA`\", \"DepB`\"}]")
        .unwrap();
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Main`\", \"DepA`\", \"DepB`\", \"System`\"]"
    );
    session.eval_text("Begin[\"`Private`\"]").unwrap();
    session.eval_text("depHead[x_] := x").unwrap();
    session.eval_text("local = depHead[depValue]").unwrap();
    session.eval_text("End[]").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    assert_eq!(
        session
            .eval_text("DepA`depHead[7]")
            .unwrap()
            .value
            .to_string(),
        "7"
    );
    assert_eq!(
        session
            .eval_text("Main`Private`local")
            .unwrap()
            .value
            .to_string(),
        "DepB`depValue"
    );
    assert_eq!(
        session
            .eval_text("Main`Private`depHead[7]")
            .unwrap()
            .value
            .to_string(),
        "Main`Private`depHead[7]"
    );
}

#[test]
fn begin_package_expands_relative_context_specs_against_the_current_context() {
    let mut session = session_with_test_control_builtins();
    session.set_current_context("Outer`");

    assert_eq!(
        session
            .eval_text("BeginPackage[\"`Inner`\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Outer`Inner`\""
    );
    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Outer`Inner`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Outer`Inner`\", \"System`\"]"
    );

    session.eval_text("value = 7").unwrap();
    session.eval_text("EndPackage[]").unwrap();

    assert_eq!(
        session.eval_text("$Context").unwrap().value.to_string(),
        "\"Outer`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Outer`Inner`\", \"System`\", \"Global`\"]"
    );
    assert_eq!(session.eval_text("value").unwrap().value.to_string(), "7");
}

#[test]
fn get_loads_a_script_file_and_returns_the_last_value() {
    let dir = TempDirGuard::new("get");
    let file = dir.write(
        "script.rum",
        "Print[\"loading script\"]\nx = 1\npair = {\n  x,\n  2\n}\npair",
    );
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .unwrap();
    assert_eq!(outcome.value.to_string(), "List[1, 2]");
    assert_eq!(outcome.prints, vec!["loading script"]);
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
}

#[test]
fn get_propagates_exit_without_collapsing_it_to_failed_load() {
    let dir = TempDirGuard::new("get-exit");
    let file = dir.write("script.rum", "Exit[6]");
    let mut session = session_with_test_control_builtins();

    match session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .expect_err("Exit inside Get should escape the loader")
    {
        EvalError::Exit { code } => assert_eq!(code, 6),
        other => panic!("expected Exit control flow, got {other:?}"),
    }
    assert!(session.take_messages().is_empty());
}

#[test]
fn nested_get_prefers_the_current_loader_directory_over_package_search_paths() {
    let root = TempDirGuard::new("nested-get-root");
    let fallback = TempDirGuard::new("nested-get-fallback");
    let main = root.write("main.rum", "Get[\"nested/inner.rum\"]");

    root.write("nested/inner.rum", "source = \"local\"\nsource");
    fallback.write("nested/inner.rum", "source = \"fallback\"\nsource");

    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([fallback.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_text(&format!("Get[\"{}\"]", rum_path(&main)))
            .unwrap()
            .value
            .to_string(),
        "\"local\""
    );
    assert_eq!(
        session.eval_text("source").unwrap().value.to_string(),
        "\"local\""
    );
}

#[test]
fn split_script_into_forms_stops_hard_syntax_errors_from_poisoning_later_forms() {
    let forms = split_script_into_forms("1 2\n3").unwrap();

    assert_eq!(forms, vec!["1 2".to_string(), "3".to_string()]);
}

#[test]
fn split_script_into_forms_skips_initial_shebang_trivia() {
    let forms = split_script_into_forms("#!/usr/bin/env rum\n\n1\n2").unwrap();

    assert_eq!(forms, vec!["1".to_string(), "2".to_string()]);
}

#[test]
fn split_script_into_forms_treats_multiline_comments_as_incomplete_trivia() {
    let forms = split_script_into_forms("1\n(* multi\n   line *)\n2").unwrap();

    assert_eq!(forms, vec!["1".to_string(), "2".to_string()]);
}

#[test]
fn eval_script_preserves_messages_emitted_before_a_later_error() {
    let mut session = session_with_test_control_builtins();

    let error = session
        .eval_script("FindFile[\"Missing`\"]\n1 ! 2")
        .expect_err("expected a later evaluation error");

    assert!(matches!(error, EvalError::Syntax(_)));
    assert!(
        session
            .take_messages()
            .iter()
            .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
    );
}

#[test]
fn get_rolls_back_loader_side_effects_and_collapses_failed_file_messages() {
    let dir = TempDirGuard::new("get-rollback");
    let file = dir.write(
        "broken.rum",
        "$Path = {\"/tmp/rum-loader-should-roll-back\"}\nBeginPackage[\"Broken`\"]\nFindFile[\"Missing`\"]\nflag = 1\nSetAttributes[f, HoldFirst]\n1 ! 2",
    );
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    let outcome = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .unwrap();

    assert_eq!(outcome.value.to_string(), "$Failed");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "syntax")
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
        session.eval_text("$Path").unwrap().value.to_string(),
        format!("List[\"{}\"]", dir.path().display())
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
fn get_cycle_termination_does_not_abort_remaining_file_forms() {
    let dir = TempDirGuard::new("get-cycle-termination");
    let file = dir.write(
        "cycle-termination.rum",
        "loopA := loopB\nloopB := loopA\nloopA\npostCycleFlag = 2\npostCycleFlag",
    );
    let mut session = session_with_test_control_builtins();

    let outcome = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
        .unwrap();

    assert_eq!(outcome.value.to_string(), "2");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "passlim")
    );
    assert!(
        !outcome
            .messages
            .iter()
            .any(|message| message.symbol == "Get" && message.tag == "load")
    );
    assert_eq!(
        session
            .eval_text("postCycleFlag")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
}

#[test]
fn get_treats_resource_termination_as_failed_load_and_rolls_back() {
    let dir = TempDirGuard::new("get-resource-limit");
    let file = dir.write(
        "resource-limit.rum",
        "preLimitFlag = 1\ngrow[x_] := grow[f[x]]\ngrow[1]\npostLimitFlag = 2",
    );
    let mut session = session_with_test_control_builtins();
    session.set_recursion_limit(32);

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
            .any(|message| message.symbol == "General" && message.tag == "reclim")
    );
    assert_eq!(
        session.eval_text("preLimitFlag").unwrap().value.to_string(),
        "preLimitFlag"
    );
    assert_eq!(
        session
            .eval_text("postLimitFlag")
            .unwrap()
            .value
            .to_string(),
        "postLimitFlag"
    );
    assert_eq!(
        session.eval_text("grow[1]").unwrap().value.to_string(),
        "grow[1]"
    );
}

#[test]
fn needs_loads_package_once_and_tracks_loaded_packages() {
    let dir = TempDirGuard::new("needs-once");
    dir.write(
        "Demo.rum",
        "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
    );
    let mut session = session_with_test_control_builtins();
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
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_text("FindFile[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        Expr::string(rum_path(&file)).to_string()
    );
    assert_eq!(
        session
            .eval_text(&format!("FindFile[File[\"{}\"]]", rum_path(&file)))
            .unwrap()
            .value
            .to_string(),
        Expr::string(rum_path(&file)).to_string()
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
    let mut session = session_with_test_control_builtins();
    session.set_package_search_paths([dir.path().to_path_buf()]);

    assert_eq!(
        session
            .eval_script("spec = \"Demo`\"\nFindFile[spec]")
            .unwrap()
            .value
            .to_string(),
        Expr::string(rum_path(&package_file)).to_string()
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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
    let mut session = session_with_test_control_builtins();

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
