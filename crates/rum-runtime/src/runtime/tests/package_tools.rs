use super::*;

#[test]
fn declare_package_declares_names_and_updates_context_path() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("DeclarePackage[\"Demo`\", {\"foo\", \"bar\"}]")
            .unwrap()
            .value
            .to_string(),
        "\"Demo`\""
    );
    assert_eq!(
        session.eval_text("$ContextPath").unwrap().value.to_string(),
        "List[\"Demo`\", \"System`\", \"Global`\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[\"Demo`*\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Demo`bar\", \"Demo`foo\"]"
    );
    assert_eq!(
        session
            .eval_text("Context[\"foo\"]")
            .unwrap()
            .value
            .to_string(),
        "\"Demo`\""
    );
}

#[test]
fn declared_package_stub_autoloads_when_symbol_is_used() {
    let dir = TempDirGuard::new("declare-package");
    let file = dir.write(
        "Demo.rum",
        "BeginPackage[\"Demo`\"]\nfoo = 42\nEndPackage[]",
    );
    let mut session = Session::new();
    session.add_package_search_path(dir.path());

    session
        .eval_text("DeclarePackage[\"Demo`\", {\"foo\"}]")
        .unwrap();

    assert_eq!(session.eval_text("foo").unwrap().value.to_string(), "42");
    assert!(
        session
            .loaded_packages()
            .iter()
            .any(|context| context == "Demo`")
    );
    assert_eq!(
        session
            .eval_text("FindFile[\"Demo`\"]")
            .unwrap()
            .value
            .to_string(),
        format!("\"{}\"", rum_path(&file))
    );
}

#[test]
fn save_writes_textual_state_that_get_can_roundtrip() {
    let dir = TempDirGuard::new("save");
    let file = dir.path().join("saved.wl");
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();
    session.eval_text("f[x_] := x + 1").unwrap();
    session.eval_text("Options[f] = {a -> 1}").unwrap();
    session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_}}")
        .unwrap();

    session
        .eval_text(&format!("Save[File[\"{}\"], f]", rum_path(&file)))
        .unwrap();

    let saved = fs::read_to_string(&file).unwrap();
    assert!(saved.contains("SetAttributes[f"));
    assert!(saved.contains("SetDelayed[f[Pattern[x, Blank[]]]"));
    assert!(saved.contains("Options[f]"));
    assert!(saved.contains("SyntaxInformation[f]"));

    let mut restored = Session::new();
    restored
        .eval_text(&format!("Get[File[\"{}\"]]", rum_path(&file)))
        .unwrap();

    assert_eq!(
        restored
            .eval_text("Attributes[f]")
            .unwrap()
            .value
            .to_string(),
        "List[HoldFirst]"
    );
    assert_eq!(
        restored.eval_text("Options[f]").unwrap().value.to_string(),
        "List[Rule[a, 1]]"
    );
    assert_eq!(
        restored
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
    assert_eq!(restored.eval_text("f[1]").unwrap().value.to_string(), "2");
}

#[test]
fn dump_save_overwrites_file_and_returns_saved_symbols() {
    let dir = TempDirGuard::new("dumpsave");
    let file = dir.write("saved.mx", "junk");
    let mut session = Session::new();

    session.eval_text("f[x_] := x + 2").unwrap();
    let outcome = session
        .eval_text(&format!("DumpSave[File[\"{}\"], f]", rum_path(&file)))
        .unwrap();
    assert_eq!(outcome.value.to_string(), "List[f]");

    let saved = fs::read_to_string(&file).unwrap();
    assert!(!saved.contains("junk"));
    assert!(saved.contains("SetDelayed[f[Pattern[x, Blank[]]], Plus[x, 2]]"));

    let mut restored = Session::new();
    restored
        .eval_text(&format!("Get[File[\"{}\"]]", rum_path(&file)))
        .unwrap();
    assert_eq!(restored.eval_text("f[1]").unwrap().value.to_string(), "3");
}

#[test]
fn save_and_dumpsave_honor_current_context_filter_and_attribute_options_subset() {
    let dir = TempDirGuard::new("save-filters");
    let save_file = dir.path().join("filtered.wl");
    let dump_file = dir.path().join("filtered.mx");
    let mut session = Session::new();

    session.eval_text("SetAttributes[f, HoldFirst]").unwrap();
    session.eval_text("f[x_] := x + 1").unwrap();
    session.eval_text("Hidden`helper[y_] := y + 10").unwrap();

    session
        .eval_text(&format!(
            "Save[File[\"{}\"], {{f, Hidden`helper}}, {{IncludedContexts -> {{\"Global`\"}}, ExcludedContexts -> None}}]",
            rum_path(&save_file)
        ))
        .unwrap();

    let saved = fs::read_to_string(&save_file).unwrap();
    assert!(saved.contains("SetDelayed[f[Pattern[x, Blank[]]]"));
    assert!(!saved.contains("Hidden`helper"));

    session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], f, {{SymbolAttributes -> False}}]",
            rum_path(&dump_file)
        ))
        .unwrap();
    let dumped = fs::read_to_string(&dump_file).unwrap();
    assert!(!dumped.contains("SetAttributes[f"));
    assert!(dumped.contains("SetDelayed[f[Pattern[x, Blank[]]]"));
}

#[test]
fn once_caches_per_session_and_per_location_key() {
    let mut session = Session::new();

    session.eval_text("count = 0").unwrap();

    assert_eq!(
        session
            .eval_text("Once[count = count + 1]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Once[count = count + 1]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("count").unwrap().value.to_string(), "1");

    assert_eq!(
        session
            .eval_text("Once[count = count + 1, \"alt\"]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Once[count = count + 1, \"alt\"]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(session.eval_text("count").unwrap().value.to_string(), "2");
}
