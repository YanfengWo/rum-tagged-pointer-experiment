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
fn dump_save_context_strings_roundtrip_public_and_private_contexts() {
    let dir = TempDirGuard::new("dumpsave-contexts");
    let file = dir.path().join("bootstrap.mx");
    let mut session = Session::new();

    session
        .eval_script(
            r#"
BeginPackage["Snap`"]
public::usage = "public"
Begin["`Private`"]
helper[x_] := x + 1
End[]
public[x_] := Snap`Private`helper[x]
EndPackage[]
"#,
        )
        .unwrap();
    assert_eq!(
        session
            .eval_text("Snap`public[4]")
            .unwrap()
            .value
            .to_string(),
        "5"
    );
    assert_eq!(
        session
            .eval_text("holContexts = Select[Contexts[], StringMatchQ[#, \"Snap`*\"] &]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Snap`\", \"Snap`Private`\"]"
    );

    let dumped = session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], Evaluate[holContexts]]",
            rum_path(&file)
        ))
        .unwrap();
    assert!(dumped.value.to_string().contains("Snap`public"));
    assert!(dumped.value.to_string().contains("Snap`Private`helper"));

    let mut restored = Session::new();
    restored
        .eval_text(&format!("Get[File[\"{}\"]]", rum_path(&file)))
        .unwrap();
    assert_eq!(
        restored
            .eval_text("Select[Contexts[], StringMatchQ[#, \"Snap`*\"] &]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Snap`\", \"Snap`Private`\"]"
    );
    assert_eq!(
        restored
            .eval_text("Select[$Packages, StringMatchQ[#, \"Snap`*\"] &]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Snap`\"]"
    );
    assert_eq!(
        restored
            .eval_text("Select[$ContextPath, StringMatchQ[#, \"Snap`*\"] &]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Snap`\"]"
    );
    assert_eq!(
        restored.eval_text("public[9]").unwrap().value.to_string(),
        "10"
    );
    let needs = restored.eval_text("Needs[\"Snap`\"]").unwrap();
    assert_eq!(needs.value.to_string(), "Null");
    assert_eq!(
        restored.eval_text("$Packages").unwrap().value.to_string(),
        "List[\"Snap`\"]"
    );
}

#[test]
fn dump_save_context_strings_skip_module_gensyms_but_keep_dollar_globals() {
    let dir = TempDirGuard::new("dumpsave-context-gensyms");
    let context_file = dir.path().join("context.mx");
    let explicit_file = dir.path().join("explicit.mx");
    let mut session = Session::new();

    session
        .eval_script(
            r#"
Begin["Sandbox`"]
f[n_] := Module[{a, b}, a = n; b = a + 1; a + b]
f[5]; f[6];
Sandbox`$keep = 42;
End[]
"#,
        )
        .unwrap();

    let generated_names = session
        .eval_text("Names[\"Sandbox`*$*\"]")
        .unwrap()
        .value
        .to_string();
    assert!(!contains_module_gensym_marker(&generated_names));

    session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], \"Sandbox`\"]",
            rum_path(&context_file)
        ))
        .unwrap();

    let saved = fs::read_to_string(&context_file).unwrap();
    assert!(saved.contains("Set[Sandbox`$keep, 42]"));
    assert!(!contains_module_gensym_marker(&saved));

    session.eval_text("Sandbox`manual$1 = 99").unwrap();
    session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], Sandbox`manual$1]",
            rum_path(&explicit_file)
        ))
        .unwrap();
    let explicit_saved = fs::read_to_string(&explicit_file).unwrap();
    assert!(explicit_saved.contains("Set[Sandbox`manual$1, 99]"));
}

#[test]
fn save_and_dumpsave_accept_names_pattern_objects() {
    let dir = TempDirGuard::new("save-name-patterns");
    let save_file = dir.path().join("saved.wl");
    let dump_file = dir.path().join("saved.mx");
    let mut session = Session::new();

    session.eval_text("Pkg`Alpha[x_] := x + 1").unwrap();
    session.eval_text("Pkg`Beta[x_] := x + 2").unwrap();
    session.eval_text("Other`Alpha[x_] := x + 10").unwrap();

    session
        .eval_text(&format!(
            "Save[File[\"{}\"], \"Pkg`A*\"]",
            rum_path(&save_file)
        ))
        .unwrap();
    let saved = fs::read_to_string(&save_file).unwrap();
    assert!(saved.contains("SetDelayed[Pkg`Alpha"));
    assert!(!saved.contains("Pkg`Beta"));
    assert!(!saved.contains("Other`Alpha"));

    let dumped = session
        .eval_text(&format!(
            "DumpSave[File[\"{}\"], RegularExpression[\"^Pkg`B.*$\"]]",
            rum_path(&dump_file)
        ))
        .unwrap();
    assert_eq!(dumped.value.to_string(), "List[Pkg`Beta]");
    let dumped = fs::read_to_string(&dump_file).unwrap();
    assert!(dumped.contains("SetDelayed[Pkg`Beta"));
    assert!(!dumped.contains("Pkg`Alpha"));
    assert!(!dumped.contains("Other`Alpha"));
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

#[test]
fn once_accepts_option_rules_in_second_position_and_respects_zero_persistence_subset() {
    let mut session = Session::new();

    session.eval_text("count = 0").unwrap();

    assert_eq!(
        session
            .eval_text("Once[count = count + 1, PersistenceTime -> None]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(
        session
            .eval_text("Once[count = count + 1, PersistenceTime -> None]")
            .unwrap()
            .value
            .to_string(),
        "1"
    );
    assert_eq!(session.eval_text("count").unwrap().value.to_string(), "1");

    assert_eq!(
        session
            .eval_text("Once[count = count + 1, {PersistenceTime -> 0}]")
            .unwrap()
            .value
            .to_string(),
        "2"
    );
    assert_eq!(
        session
            .eval_text("Once[count = count + 1, {PersistenceTime -> 0}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
    assert_eq!(session.eval_text("count").unwrap().value.to_string(), "3");
}

#[test]
fn save_rejects_unknown_options_and_once_reports_unknown_option_names() {
    let dir = TempDirGuard::new("save-unknown-option");
    let save_file = dir.path().join("unknown.wl");
    let mut session = Session::new();

    let save = session
        .eval_text(&format!(
            "Save[File[\"{}\"], f, Bogus -> 1]",
            rum_path(&save_file)
        ))
        .unwrap();
    assert_eq!(
        save.value.to_string(),
        format!(
            "Save[File[\"{}\"], f, Rule[Bogus, 1]]",
            rum_path(&save_file)
        )
    );
    assert!(
        save.messages
            .iter()
            .any(|message| message.symbol == "Save" && message.tag == "optx")
    );

    let once = session.eval_text("Once[1 + 1, Bogus -> 1]").unwrap();
    assert_eq!(once.value.to_string(), "2");
    assert!(
        once.messages
            .iter()
            .any(|message| message.symbol == "OptionValue" && message.tag == "nodef")
    );
}

fn contains_module_gensym_marker(text: &str) -> bool {
    text.as_bytes()
        .windows(2)
        .enumerate()
        .any(|(index, pair)| index > 0 && pair[0] == b'$' && pair[1].is_ascii_digit())
}
