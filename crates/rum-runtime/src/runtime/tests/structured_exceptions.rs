use super::*;

#[test]
fn exception_type_registry_round_trips_through_loader_snapshot() {
    let mut session = Session::new();
    session.exception_types.insert("NetError".to_string(), None);
    session
        .exception_types
        .insert("Timeout".to_string(), Some("NetError".to_string()));

    let snapshot = session.capture_loader_snapshot();
    session.exception_types.clear();
    session.exception_types.insert("Other".to_string(), None);
    session.restore_loader_snapshot(snapshot);

    assert_eq!(
        session.exception_types,
        BTreeMap::from([
            ("NetError".to_string(), None),
            ("Timeout".to_string(), Some("NetError".to_string()),),
        ])
    );
}
