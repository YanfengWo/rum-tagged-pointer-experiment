use super::super::*;
use super::support::*;

// Moved from crates/rum-runtime/src/runtime/tests/package_tools.rs.

#[test]
fn dump_save_context_strings_roundtrip_public_and_private_contexts() {
    let dir = TempDirGuard::new("dumpsave-contexts");
    let file = dir.path().join("bootstrap.mx");
    let mut session = core_session();

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

    let mut restored = core_session();
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
