use rum_runtime::Capability;

#[test]
fn date_and_time_object_predicates_remain_pure() {
    fn capability(name: &str) -> Capability {
        crate::catalog::PREDICATE_BUILTINS
            .iter()
            .find(|spec| spec.name == name)
            .expect("predicate builtin should be cataloged")
            .capability
    }

    assert_eq!(capability("DateObjectQ"), Capability::Pure);
    assert_eq!(capability("TimeObjectQ"), Capability::Pure);
}
