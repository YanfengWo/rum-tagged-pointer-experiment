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

#[test]
fn random_catalog_capabilities_match_dispatch_surface() {
    fn capability(name: &str) -> Capability {
        crate::catalog::RANDOM_BUILTINS
            .iter()
            .find(|spec| spec.name == name)
            .expect("random builtin should be cataloged")
            .capability
    }

    assert_eq!(capability("RandomInteger"), Capability::Pure);
    assert_eq!(capability("RandomChoice"), Capability::Pure);
    assert_eq!(capability("RandomSample"), Capability::Pure);
    assert_eq!(capability("BlockRandom"), Capability::Pure);
    assert_eq!(capability("RandomUuid"), Capability::Pure);
    assert_eq!(capability("UuidQ"), Capability::Pure);
    assert_eq!(capability("SeedRandom"), Capability::Pure);
    assert_eq!(capability("RandomBytes"), Capability::Random);
}
