use super::*;

fn assert_value(session: &mut Session, source: &str, expected: &str) {
    let outcome = session.eval_text(source).unwrap();
    assert_eq!(outcome.value.to_string(), expected, "{source}");
    assert!(
        outcome.messages.is_empty(),
        "{source}: {:?}",
        outcome.messages
    );
}

#[test]
fn unset_and_postfix_unset_prepare_active_definition_identities() {
    let mut session = session_with_test_control_builtins();

    assert_value(
        &mut session,
        "ClearAll[iUnset, jUnset, fUnset, gUnset]; \
         iUnset = 2; fUnset[iUnset] = 701; Unset[fUnset[iUnset]]; \
         jUnset = 3; gUnset[jUnset] = 702; gUnset[jUnset] =.; \
         {fUnset[2], DownValues[fUnset], gUnset[3], DownValues[gUnset]}",
        "List[fUnset[2], List[], gUnset[3], List[]]",
    );
}

#[test]
fn unset_prepares_held_and_pattern_lhs_by_construction() {
    let mut session = session_with_test_control_builtins();

    assert_value(
        &mut session,
        "ClearAll[xHeldUnset, heldUnset, patternUnset]; \
         xHeldUnset = 5; SetAttributes[heldUnset, HoldFirst]; \
         heldUnset[xHeldUnset] = heldValue; Unset[heldUnset[xHeldUnset]]; \
         patternUnset[x_] := patternValue[x]; Unset[patternUnset[y_]]; \
         {heldUnset[xHeldUnset], DownValues[heldUnset], \
          patternUnset[9], DownValues[patternUnset]}",
        "List[heldUnset[xHeldUnset], List[], patternUnset[9], List[]]",
    );
}

#[test]
fn unset_snapshot_identity_removes_without_rebinding_but_not_after_rebinding() {
    let mut session = session_with_test_control_builtins();

    assert_value(
        &mut session,
        "ClearAll[iSameUnset, sameUnset, iReboundUnset, reboundUnset]; \
         iSameUnset = 2; sameUnset[iSameUnset] = sameValue; \
         Unset[sameUnset[iSameUnset]]; \
         iReboundUnset = 2; reboundUnset[iReboundUnset] = originalValue; \
         iReboundUnset = 3; Unset[reboundUnset[iReboundUnset]]; \
         {sameUnset[2], DownValues[sameUnset], reboundUnset[2], \
          reboundUnset[3], DownValues[reboundUnset]}",
        "List[sameUnset[2], List[], originalValue, reboundUnset[3], List[RuleDelayed[HoldPattern[reboundUnset[2]], originalValue]]]",
    );
}

#[test]
fn unset_after_attribute_invalidation_addresses_only_the_new_identity() {
    let mut session = session_with_test_control_builtins();

    assert_value(
        &mut session,
        "ClearAll[xAttributeUnset, attributeUnset]; xAttributeUnset = 5; \
         attributeUnset[xAttributeUnset] = oldIdentity; \
         SetAttributes[attributeUnset, HoldFirst]; \
         attributeUnset[xAttributeUnset] = newIdentity; \
         Unset[attributeUnset[xAttributeUnset]]; \
         {attributeUnset[xAttributeUnset], attributeUnset[5], \
          DownValues[attributeUnset]}",
        "List[attributeUnset[xAttributeUnset], oldIdentity, List[RuleDelayed[HoldPattern[attributeUnset[5]], oldIdentity]]]",
    );
}

#[test]
fn unset_after_own_value_rebinding_and_clear_preserves_the_stored_snapshot() {
    let mut session = session_with_test_control_builtins();

    assert_value(
        &mut session,
        "ClearAll[iOwnUnset, ownUnset]; iOwnUnset = 1; \
         ownUnset[iOwnUnset] = originalIdentity; \
         iOwnUnset = 2; Unset[ownUnset[iOwnUnset]]; \
         Clear[iOwnUnset]; Unset[ownUnset[iOwnUnset]]; \
         {ownUnset[1], ownUnset[2], ownUnset[iOwnUnset], DownValues[ownUnset]}",
        "List[originalIdentity, ownUnset[2], ownUnset[iOwnUnset], List[RuleDelayed[HoldPattern[ownUnset[1]], originalIdentity]]]",
    );
}

#[test]
fn clear_removes_all_identities_and_reinstall_has_no_stale_unset_target() {
    let mut session = session_with_test_control_builtins();

    assert_value(
        &mut session,
        "ClearAll[iClearUnset, clearUnset]; iClearUnset = 1; \
         clearUnset[iClearUnset] = firstIdentity; iClearUnset = 2; \
         clearUnset[iClearUnset] = secondIdentity; Clear[clearUnset]; \
         iClearUnset = 3; clearUnset[iClearUnset] = reinstalledIdentity; \
         Unset[clearUnset[iClearUnset]]; \
         {clearUnset[1], clearUnset[2], clearUnset[3], DownValues[clearUnset]}",
        "List[clearUnset[1], clearUnset[2], clearUnset[3], List[]]",
    );
}
