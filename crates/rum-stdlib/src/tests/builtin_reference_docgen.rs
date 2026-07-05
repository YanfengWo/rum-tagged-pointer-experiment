use crate::docgen;

#[test]
fn builtin_reference_is_in_sync() {
    let generated =
        docgen::generate_builtin_reference().expect("builtin reference generation should succeed");
    let committed = include_str!("../../../../docs/reference/builtins.md");

    assert!(
        committed == generated.markdown,
        "builtin reference drift; run `{}`",
        docgen::REGEN_COMMAND
    );
}
