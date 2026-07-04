#[test]
fn file_template_apply_catalog_capability_stays_file_write() {
    let capability = crate::catalog::STRING_BUILTINS
        .iter()
        .find(|spec| spec.name == "FileTemplateApply")
        .expect("FileTemplateApply should be cataloged")
        .capability;

    assert_eq!(capability, rum_runtime::kernel_api::Capability::FileWrite);
}
