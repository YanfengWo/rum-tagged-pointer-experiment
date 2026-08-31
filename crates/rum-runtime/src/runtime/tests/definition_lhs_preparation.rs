use super::*;
use crate::runtime::test_builtins::register_test_arithmetic_builtins;

fn definition_lhs_session() -> Session {
    let mut session = session_with_test_control_builtins();
    register_test_arithmetic_builtins(&mut session);
    session
}

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
fn active_definition_lhs_positions_complete_own_value_evaluation_for_both_set_forms() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[iImmediate, iDelayed, fImmediate, fDelayed, fImmediateDelayedSet, \
                  fDelayedSet]; \
         iImmediate = 2; iDelayed := 1 + 1; \
         fImmediate[iImmediate] = immediate; \
         fDelayed[iDelayed] = delayed; \
         fImmediateDelayedSet[iImmediate] := immediateDelayedSet; \
         fDelayedSet[iDelayed] := delayedSet; \
         {fImmediate[2], DownValues[fImmediate], \
          fDelayed[2], DownValues[fDelayed], \
          fImmediateDelayedSet[2], DownValues[fImmediateDelayedSet], \
          fDelayedSet[2], DownValues[fDelayedSet]}",
        "List[immediate, List[RuleDelayed[HoldPattern[fImmediate[2]], immediate]], delayed, List[RuleDelayed[HoldPattern[fDelayed[2]], delayed]], immediateDelayedSet, List[RuleDelayed[HoldPattern[fImmediateDelayedSet[2]], immediateDelayedSet]], delayedSet, List[RuleDelayed[HoldPattern[fDelayedSet[2]], delayedSet]]]",
    );
}

#[test]
fn held_definition_lhs_positions_keep_raw_symbols_and_source_lookup_reaches_them() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[xHeld, hFirst, hRest, hAll, hComplete]; xHeld = 5; \
         SetAttributes[hFirst, HoldFirst]; hFirst[xHeld, xHeld, xHeld] = first; \
         SetAttributes[hRest, HoldRest]; hRest[xHeld, xHeld, xHeld] = rest; \
         SetAttributes[hAll, HoldAll]; hAll[xHeld, xHeld] = all; \
         SetAttributes[hComplete, HoldAllComplete]; hComplete[xHeld, xHeld] = complete; \
         {hFirst[xHeld, xHeld, xHeld], DownValues[hFirst], \
          hRest[xHeld, xHeld, xHeld], DownValues[hRest], \
          hAll[xHeld, xHeld], DownValues[hAll], \
          hComplete[xHeld, xHeld], DownValues[hComplete]}",
        "List[first, List[RuleDelayed[HoldPattern[hFirst[xHeld, 5, 5]], first]], rest, List[RuleDelayed[HoldPattern[hRest[5, xHeld, xHeld]], rest]], all, List[RuleDelayed[HoldPattern[hAll[xHeld, xHeld]], all]], complete, List[RuleDelayed[HoldPattern[hComplete[xHeld, xHeld]], complete]]]",
    );
}

#[test]
fn definition_lhs_preparation_obeys_evaluate_unevaluated_and_sequence_controls() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[xControl, hEvaluate, hCompleteEvaluate, hUnevaluated, hSequence, hSequenceHold]; \
         xControl = 5; \
         SetAttributes[hEvaluate, HoldAll]; hEvaluate[Evaluate[xControl]] = escaped; \
         SetAttributes[hCompleteEvaluate, HoldAllComplete]; \
         hCompleteEvaluate[Evaluate[xControl]] = blocked; \
         hUnevaluated[Unevaluated[xControl]] = stripped; \
         hSequence[Sequence[a, b]] = spliced; \
         SetAttributes[hSequenceHold, SequenceHold]; \
         hSequenceHold[Sequence[a, b]] = sequenceHeld; \
         {hEvaluate[Evaluate[xControl]], DownValues[hEvaluate], \
          hCompleteEvaluate[Evaluate[xControl]], DownValues[hCompleteEvaluate], \
          hUnevaluated[Unevaluated[xControl]], DownValues[hUnevaluated], \
          hSequence[Sequence[a, b]], DownValues[hSequence], \
          hSequenceHold[Sequence[a, b]], DownValues[hSequenceHold]}",
        "List[escaped, List[RuleDelayed[HoldPattern[hEvaluate[5]], escaped]], blocked, List[RuleDelayed[HoldPattern[hCompleteEvaluate[Evaluate[xControl]]], blocked]], stripped, List[RuleDelayed[HoldPattern[hUnevaluated[xControl]], stripped]], spliced, List[RuleDelayed[HoldPattern[hSequence[a, b]], spliced]], sequenceHeld, List[RuleDelayed[HoldPattern[hSequenceHold[Sequence[a, b]]], sequenceHeld]]]",
    );
}

#[test]
fn active_nested_calls_dispatch_held_nested_calls_do_not_and_callable_heads_prepare_first() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[xNested, nested, activeOuter, heldOuter, headFactory, callable, subCallable]; \
         xNested = 5; nested[1] = 2; \
         activeOuter[nested[1]] = active; \
         SetAttributes[heldOuter, HoldAll]; heldOuter[nested[1]] = held; \
         headFactory[] = callable; SetAttributes[callable, HoldAll]; \
         headFactory[][xNested] = callableHead; \
         SetAttributes[subCallable, SubValuesHoldAll]; \
         subCallable[0][xNested] = subCallableHead; \
         {activeOuter[nested[1]], DownValues[activeOuter], \
          heldOuter[nested[1]], DownValues[heldOuter], \
          callable[xNested], DownValues[callable], \
          subCallable[0][xNested], SubValues[subCallable]}",
        "List[active, List[RuleDelayed[HoldPattern[activeOuter[2]], active]], held, List[RuleDelayed[HoldPattern[heldOuter[nested[1]]], held]], callableHead, List[RuleDelayed[HoldPattern[callable[xNested]], callableHead]], subCallableHead, List[RuleDelayed[HoldPattern[subCallable[0][xNested]], subCallableHead]]]",
    );
}

#[test]
fn held_ground_flat_and_orderless_identities_are_reachable_from_both_forms() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[flatHead, orderlessHead, heldFlat, heldOrderless]; \
         SetAttributes[flatHead, Flat]; SetAttributes[orderlessHead, Orderless]; \
         SetAttributes[heldFlat, HoldAll]; SetAttributes[heldOrderless, HoldAll]; \
         heldFlat[flatHead[1, flatHead[2, 3]]] = flatValue; \
         heldOrderless[orderlessHead[3, 1, 2]] = orderlessValue; \
         {heldFlat[flatHead[1, flatHead[2, 3]]], heldFlat[flatHead[1, 2, 3]], \
          DownValues[heldFlat], \
          heldOrderless[orderlessHead[3, 1, 2]], heldOrderless[orderlessHead[1, 2, 3]], \
          DownValues[heldOrderless]}",
        "List[flatValue, flatValue, List[RuleDelayed[HoldPattern[heldFlat[flatHead[1, 2, 3]]], flatValue]], orderlessValue, orderlessValue, List[RuleDelayed[HoldPattern[heldOrderless[orderlessHead[1, 2, 3]]], orderlessValue]]]",
    );
}

#[test]
fn definition_lhs_pattern_regions_prepare_constraints_but_defer_binders_and_predicates() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[xBinder, yTyped, zTest, wCondition, binderHead, typedHead, testHead, \
                  conditionHead, typeAlias, predicateAlias, conditionAlias]; \
         xBinder = 9; typeAlias = Integer; predicateAlias = evaluatedPredicate; \
         conditionAlias = False; \
         binderHead[xBinder_] := binder; typedHead[yTyped_typeAlias] := typed; \
         SetDelayed[testHead[PatternTest[Pattern[zTest, Blank[]], predicateAlias]], tested]; \
         SetDelayed[Condition[conditionHead[Pattern[wCondition, Blank[]]], conditionAlias], conditioned]; \
         {DownValues[binderHead], DownValues[typedHead], \
          DownValues[testHead], DownValues[conditionHead]}",
        "List[List[RuleDelayed[HoldPattern[binderHead[Pattern[xBinder, Blank[]]]], binder]], List[RuleDelayed[HoldPattern[typedHead[Pattern[yTyped, Blank[Integer]]]], typed]], List[RuleDelayed[HoldPattern[testHead[PatternTest[Pattern[zTest, Blank[]], predicateAlias]]], tested]], List[RuleDelayed[HoldPattern[Condition[conditionHead[Pattern[wCondition, Blank[]]], conditionAlias]], conditioned]]]",
    );
}

#[test]
fn matcher_wrappers_keep_literal_pattern_regions_raw_and_prepare_nested_constraints() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[typeAlias, patternRegionCalls, patternRegionCall, fVerbatimRegion, \
                  fPatternSequenceRegion, fOrderlessPatternSequenceRegion, \
                  fKeyValuePatternRegion, fLongestRegion, fShortestRegion, \
                  fOptionsPatternRegion, verbatimCount, patternSequenceCount, \
                  orderlessPatternSequenceCount, keyValuePatternCount, longestCount, \
                  shortestCount, optionsPatternCount]; \
         typeAlias = Integer; patternRegionCalls = 0; \
         patternRegionCall[] := (patternRegionCalls = patternRegionCalls + 1; zzz); \
         fVerbatimRegion[Verbatim[Blank[typeAlias]], \
             Verbatim[patternRegionCall[]]] = verbatim; \
         verbatimCount = patternRegionCalls; patternRegionCalls = 0; \
         fPatternSequenceRegion[PatternSequence[patternRegionCall[], \
             Pattern[bPatternSequence, Blank[typeAlias]]]] = patternSequence; \
         patternSequenceCount = patternRegionCalls; patternRegionCalls = 0; \
         fOrderlessPatternSequenceRegion[OrderlessPatternSequence[patternRegionCall[], \
             Pattern[bOrderlessPatternSequence, Blank[typeAlias]]]] = \
             orderlessPatternSequence; \
         orderlessPatternSequenceCount = patternRegionCalls; patternRegionCalls = 0; \
         fKeyValuePatternRegion[System`KeyValuePattern[List[Rule[patternRegionCall[], \
             Pattern[bKeyValuePattern, Blank[typeAlias]]]]]] = keyValuePattern; \
         keyValuePatternCount = patternRegionCalls; patternRegionCalls = 0; \
         fLongestRegion[Longest[Repeated[Alternatives[patternRegionCall[], \
             Blank[typeAlias]]]]] = longest; \
         longestCount = patternRegionCalls; patternRegionCalls = 0; \
         fShortestRegion[Shortest[Repeated[Alternatives[patternRegionCall[], \
             Blank[typeAlias]]]]] = shortest; \
         shortestCount = patternRegionCalls; patternRegionCalls = 0; \
         fOptionsPatternRegion[OptionsPattern[patternRegionCall[]]] = optionsPattern; \
         optionsPatternCount = patternRegionCalls; \
         {{verbatimCount, DownValues[fVerbatimRegion]}, \
          {patternSequenceCount, DownValues[fPatternSequenceRegion]}, \
          {orderlessPatternSequenceCount, DownValues[fOrderlessPatternSequenceRegion]}, \
          {keyValuePatternCount, DownValues[fKeyValuePatternRegion]}, \
          {longestCount, DownValues[fLongestRegion]}, \
          {shortestCount, DownValues[fShortestRegion]}, \
          {optionsPatternCount, DownValues[fOptionsPatternRegion]}}",
        "List[List[0, List[RuleDelayed[HoldPattern[fVerbatimRegion[Verbatim[Blank[typeAlias]], Verbatim[patternRegionCall[]]]], verbatim]]], List[0, List[RuleDelayed[HoldPattern[fPatternSequenceRegion[PatternSequence[patternRegionCall[], Pattern[bPatternSequence, Blank[Integer]]]]], patternSequence]]], List[0, List[RuleDelayed[HoldPattern[fOrderlessPatternSequenceRegion[OrderlessPatternSequence[patternRegionCall[], Pattern[bOrderlessPatternSequence, Blank[Integer]]]]], orderlessPatternSequence]]], List[0, List[RuleDelayed[HoldPattern[fKeyValuePatternRegion[KeyValuePattern[List[Rule[patternRegionCall[], Pattern[bKeyValuePattern, Blank[Integer]]]]]]], keyValuePattern]]], List[0, List[RuleDelayed[HoldPattern[fLongestRegion[Longest[Repeated[Alternatives[patternRegionCall[], Blank[Integer]]]]]], longest]]], List[0, List[RuleDelayed[HoldPattern[fShortestRegion[Shortest[Repeated[Alternatives[patternRegionCall[], Blank[Integer]]]]]], shortest]]], List[0, List[RuleDelayed[HoldPattern[fOptionsPatternRegion[OptionsPattern[patternRegionCall[]]]], optionsPattern]]]]",
    );
}

#[test]
fn active_lhs_and_pattern_test_pattern_operands_still_prepare_once() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[activeCalls, patternOperandCalls, activeCall, patternOperand, \
                  activeHead, patternTestHead, typeAlias, predicateAlias]; \
         activeCalls = 0; patternOperandCalls = 0; typeAlias = Integer; \
         predicateAlias = evaluatedPredicate; \
         activeCall[] := (activeCalls = activeCalls + 1; 2); \
         patternOperand := (patternOperandCalls = patternOperandCalls + 1; \
                            Blank[typeAlias]); \
         activeHead[activeCall[]] = active; \
         SetDelayed[patternTestHead[System`PatternTest[patternOperand, predicateAlias]], tested]; \
         {activeCalls, DownValues[activeHead], patternOperandCalls, \
          DownValues[patternTestHead]}",
        "List[1, List[RuleDelayed[HoldPattern[activeHead[2]], active]], 1, List[RuleDelayed[HoldPattern[patternTestHead[PatternTest[Blank[Integer], predicateAlias]]], tested]]]",
    );
}

#[test]
fn one_metadata_snapshot_controls_every_argument_even_after_an_earlier_mutation() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[snapshotHead, mutateSnapshotHead, snapshotX, snapshotCalls]; \
         snapshotX = 5; snapshotCalls = 0; \
         mutateSnapshotHead[] := (snapshotCalls = snapshotCalls + 1; \
                                  SetAttributes[snapshotHead, HoldAll]; token); \
         snapshotHead[mutateSnapshotHead[], snapshotX] = value; \
         {DownValues[snapshotHead], snapshotHead[token, 5], snapshotCalls, \
          Attributes[snapshotHead]}",
        "List[List[RuleDelayed[HoldPattern[snapshotHead[token, 5]], value]], value, 1, List[HoldAll]]",
    );
}

#[test]
fn adding_hold_attributes_after_an_existing_definition_keeps_old_and_new_identities_live() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[xAttributeInvalidation, fAttributeInvalidation]; \
         xAttributeInvalidation = 5; \
         fAttributeInvalidation[xAttributeInvalidation] = old; \
         SetAttributes[fAttributeInvalidation, HoldFirst]; \
         fAttributeInvalidation[xAttributeInvalidation] = new; \
         {fAttributeInvalidation[xAttributeInvalidation], fAttributeInvalidation[5], \
          DownValues[fAttributeInvalidation]}",
        "List[new, old, List[RuleDelayed[HoldPattern[fAttributeInvalidation[5]], old], RuleDelayed[HoldPattern[fAttributeInvalidation[xAttributeInvalidation]], new]]]",
    );
}

#[test]
fn reassigned_then_cleared_lhs_own_value_changes_only_future_definition_identities() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[kOwnInvalidation, fOwnInvalidation]; \
         kOwnInvalidation = 1; fOwnInvalidation[kOwnInvalidation] = one; \
         kOwnInvalidation = 2; fOwnInvalidation[kOwnInvalidation] = two; \
         Clear[kOwnInvalidation]; fOwnInvalidation[kOwnInvalidation] = raw; \
         {fOwnInvalidation[1], fOwnInvalidation[2], fOwnInvalidation[kOwnInvalidation], \
          DownValues[fOwnInvalidation]}",
        "List[one, two, raw, List[RuleDelayed[HoldPattern[fOwnInvalidation[1]], one], RuleDelayed[HoldPattern[fOwnInvalidation[2]], two], RuleDelayed[HoldPattern[fOwnInvalidation[kOwnInvalidation]], raw]]]",
    );
}

#[test]
fn clear_then_reinstall_has_no_stale_prepared_identity_or_matcher() {
    let mut session = definition_lhs_session();

    assert_value(
        &mut session,
        "ClearAll[kClearInvalidation, fClearInvalidation]; \
         kClearInvalidation = 1; fClearInvalidation[kClearInvalidation] = old; \
         Clear[fClearInvalidation]; \
         kClearInvalidation = 2; fClearInvalidation[kClearInvalidation] = new; \
         {fClearInvalidation[1], fClearInvalidation[2], DownValues[fClearInvalidation]}",
        "List[fClearInvalidation[1], new, List[RuleDelayed[HoldPattern[fClearInvalidation[2]], new]]]",
    );
}
