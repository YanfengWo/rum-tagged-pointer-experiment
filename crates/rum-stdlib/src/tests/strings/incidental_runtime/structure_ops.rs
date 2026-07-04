use super::super::*;

// Moved from crates/rum-runtime/src/runtime/tests/structure_ops/dedup.rs.

#[test]
fn delete_duplicates_by_keeps_first_item_for_each_evaluated_key() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("DeleteDuplicatesBy[{1, 2, 3, 4}, Mod[#, 2] &]")
            .unwrap()
            .value
            .to_string(),
        "List[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("DeleteDuplicatesBy[f[1, 2, 3, 4], Mod[#, 2] &]")
            .unwrap()
            .value
            .to_string(),
        "f[1, 2]"
    );
    assert_eq!(
        session
            .eval_text("DeleteDuplicatesBy[{a, b, aa, c}, StringLength[ToString[#]] &]")
            .unwrap()
            .value
            .to_string(),
        "List[a, aa]"
    );
}
