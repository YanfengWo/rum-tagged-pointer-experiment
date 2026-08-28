use super::*;

fn eval_value(session: &mut Session, source: &str) -> String {
    let outcome = session.eval_text(source).unwrap();
    assert!(
        outcome.messages.is_empty(),
        "{source} emitted unexpected messages: {:?}",
        outcome.messages
    );
    outcome.value.to_string()
}

#[test]
fn all_tagged_install_entry_points_store_the_prepared_identity_by_owner() {
    let mut session = session_with_test_control_builtins();

    // Entry points: /: with = and :=, classified as DownValues after active
    // argument preparation.
    assert_eq!(
        eval_value(
            &mut session,
            "tg = 5; h /: h[tag1, tg] = 7; \
             tgD = 5; hd /: hd[tagd, tgD] := 17; \
             {h[tag1, 5], hd[tagd, 5]}"
        ),
        "List[7, 17]"
    );
    assert_eq!(
        eval_value(&mut session, "DownValues[h]"),
        "List[RuleDelayed[HoldPattern[h[tag1, 5]], 7]]"
    );
    assert_eq!(
        eval_value(&mut session, "DownValues[hd]"),
        "List[RuleDelayed[HoldPattern[hd[tagd, 5]], 17]]"
    );

    // Entry points: /: with = and :=, classified as UpValues owned by the
    // explicit argument tag after preparation.
    assert_eq!(
        eval_value(
            &mut session,
            "vv = 5; g /: k[g, vv] = 11; \
             vvD = 5; gD /: kD[gD, vvD] := 21; \
             {k[g, 5], kD[gD, 5]}"
        ),
        "List[11, 21]"
    );
    assert_eq!(
        eval_value(&mut session, "UpValues[g]"),
        "List[RuleDelayed[HoldPattern[k[g, 5]], 11]]"
    );
    assert_eq!(
        eval_value(&mut session, "UpValues[gD]"),
        "List[RuleDelayed[HoldPattern[kD[gD, 5]], 21]]"
    );

    // Explicit TagSet/TagSetDelayed entry points classified as SubValues.
    assert_eq!(
        eval_value(
            &mut session,
            "subI = 5; TagSet[sub, sub[subI][q_], 31]; \
             subDI = 6; TagSetDelayed[subD, subD[subDI][q_], delayed[q]]; \
             {sub[5][9], subD[6][10]}"
        ),
        "List[31, delayed[10]]"
    );
    assert_eq!(
        eval_value(&mut session, "SubValues[sub]"),
        "List[RuleDelayed[HoldPattern[sub[5][Pattern[q, Blank[]]]], 31]]"
    );
    assert_eq!(
        eval_value(&mut session, "SubValues[subD]"),
        "List[RuleDelayed[HoldPattern[subD[6][Pattern[q, Blank[]]]], delayed[q]]]"
    );

    // Entry points: ^= and ^:=. A surviving symbol in the canonical lhs owns
    // each inferred UpValue.
    assert_eq!(
        eval_value(
            &mut session,
            "upI = 5; upHead[upTag, upI] ^= 22; \
             upDI = 6; upHeadD[upTagD, upDI] ^:= delayedUp; \
             {upHead[upTag, 5], upHeadD[upTagD, 6]}"
        ),
        "List[22, delayedUp]"
    );
    assert_eq!(
        eval_value(&mut session, "UpValues[upTag]"),
        "List[RuleDelayed[HoldPattern[upHead[upTag, 5]], 22]]"
    );
    assert_eq!(
        eval_value(&mut session, "UpValues[upTagD]"),
        "List[RuleDelayed[HoldPattern[upHeadD[upTagD, 6]], delayedUp]]"
    );
}

#[test]
fn upset_infers_tags_only_after_preparation_and_keeps_nosym() {
    let mut session = session_with_test_control_builtins();

    let immediate = session.eval_text("ww = 5; m[ww] ^= 22").unwrap();
    assert_eq!(immediate.value.to_string(), "22");
    assert!(immediate.messages.iter().any(|message| {
        message.symbol == "UpSet" && message.tag == "nosym" && message.text.contains("m[5]")
    }));
    assert_eq!(eval_value(&mut session, "UpValues[m]"), "List[]");

    let delayed = session.eval_text("wwD = 5; mD[wwD] ^:= 32").unwrap();
    assert_eq!(delayed.value.to_string(), "Null");
    assert!(delayed.messages.iter().any(|message| {
        message.symbol == "UpSetDelayed" && message.tag == "nosym" && message.text.contains("mD[5]")
    }));
    assert_eq!(eval_value(&mut session, "UpValues[mD]"), "List[]");
}

#[test]
fn tagged_preparation_preserves_patterns_all_holds_and_callable_heads() {
    let mut session = session_with_test_control_builtins();

    // Pattern binders and typed blanks stay pattern syntax while active ground
    // arguments are prepared.
    assert_eq!(
        eval_value(
            &mut session,
            "ground = 5; cc /: cc[ctag, qq_] = 88; \
             typedTag /: typedHead[typedTag, q_Integer, ground] := typed[q]; \
             {cc[ctag, 9], typedHead[typedTag, 7, 5], \
              typedHead[typedTag, other, 5]}"
        ),
        "List[88, typed[7], typedHead[typedTag, other, 5]]"
    );

    // HoldFirst and HoldRest exercise both sides of a partially-held call;
    // HoldAll and HoldAllComplete retain the complete raw argument list.
    assert_eq!(
        eval_value(
            &mut session,
            "xHeld = 5; \
             SetAttributes[hFirst, HoldFirst]; \
             SetAttributes[hRest, HoldRest]; \
             SetAttributes[hAll, HoldAll]; \
             SetAttributes[hComplete, HoldAllComplete]; \
             hFirst /: hFirst[xHeld, firstMark] = first; \
             hRest /: hRest[restMark, xHeld] = rest; \
             hAll /: hAll[xHeld, allMark] = all; \
             hComplete /: hComplete[xHeld, completeMark] = complete; \
             {hFirst[xHeld, firstMark], hRest[restMark, xHeld], \
              hAll[xHeld, allMark], hComplete[xHeld, completeMark]}"
        ),
        "List[first, rest, all, complete]"
    );

    // SequenceHold is the fifth hold-family attribute: it retains Sequence as
    // one identity argument on both store and lookup.
    assert_eq!(
        eval_value(
            &mut session,
            "SetAttributes[hSequence, SequenceHold]; \
             hSequence /: hSequence[Sequence[a, b], sequenceMark] = sequence; \
             hSequence[Sequence[a, b], sequenceMark]"
        ),
        "sequence"
    );

    // A nested active argument completes evaluation, while a callable lhs
    // head is evaluated first and its HoldFirst metadata governs preparation.
    assert_eq!(
        eval_value(
            &mut session,
            "nestedValue = 5; \
             nestedTag /: nestedOuter[nestedInner[nestedValue], nestedTag] = nested; \
             callableFactory[] := callableHead; \
             SetAttributes[callableHead, HoldFirst]; callableValue = 9; \
             callableTag /: callableFactory[][callableValue, callableTag] = callable; \
             {nestedOuter[nestedInner[5], nestedTag], \
              callableHead[callableValue, callableTag]}"
        ),
        "List[nested, callable]"
    );
}

#[test]
fn tagged_snapshot_identity_tracks_attribute_rebind_clear_and_owner_invalidation() {
    let mut session = session_with_test_control_builtins();

    // Invalidation: SetAttributes runs after the first definition. The old
    // canonical identity stays f[5,...], while the new held definition stores
    // the raw symbol identity.
    eval_value(
        &mut session,
        "attrX = 5; attrHead /: attrHead[attrX, attrMark] = beforeHold; \
         SetAttributes[attrHead, HoldFirst]",
    );
    assert_eq!(
        eval_value(
            &mut session,
            "{attrHead[attrX, attrMark], attrHead[5, attrMark]}"
        ),
        "List[attrHead[attrX, attrMark], beforeHold]"
    );
    assert_eq!(
        eval_value(
            &mut session,
            "attrHead /: attrHead[attrX, attrMark] = afterHold; \
             {attrHead[attrX, attrMark], attrHead[5, attrMark]}"
        ),
        "List[afterHold, beforeHold]"
    );

    // TagUnset entry point under unchanged state removes the prepared rule.
    assert_eq!(
        eval_value(
            &mut session,
            "iTag = 3; tu /: tu[tuMark, iTag] = 77; \
             tu /: tu[tuMark, iTag] =.; {tu[tuMark, 3], DownValues[tu]}"
        ),
        "List[tu[tuMark, 3], List[]]"
    );

    // The existing explicit Unset[TagSet[tag, lhs]] spelling reaches the
    // same canonical removal path; its embedded active argument is prepared.
    assert_eq!(
        eval_value(
            &mut session,
            "explicitIndex = 8; \
             TagSet[explicitTag, explicitHead[explicitTag, explicitIndex], explicit]; \
             Unset[TagSet[explicitTag, explicitHead[explicitTag, explicitIndex]]]; \
             {explicitHead[explicitTag, 8], UpValues[explicitTag]}"
        ),
        "List[explicitHead[explicitTag, 8], List[]]"
    );

    // Invalidation: rebinding iTag makes TagUnset address [4], so the stored
    // [3] snapshot remains. Clearing iTag then makes a new raw identity
    // addressable and removable without touching [3].
    assert_eq!(
        eval_value(
            &mut session,
            "iTag = 3; tu /: tu[tuMark, iTag] = canonicalThree; \
             iTag = 4; TagUnset[tu, tu[tuMark, iTag]]; \
             afterRebind = tu[tuMark, 3]; \
             Clear[iTag]; tu /: tu[tuMark, iTag] = rawIdentity; \
             TagUnset[tu, tu[tuMark, iTag]]; \
             {afterRebind, tu[tuMark, iTag], tu[tuMark, 3], DownValues[tu]}"
        ),
        "List[canonicalThree, tu[tuMark, iTag], canonicalThree, List[RuleDelayed[HoldPattern[tu[tuMark, 3]], canonicalThree]]]"
    );

    // Invalidation: Clear on the owning head remains the owner-wide removal.
    assert_eq!(
        eval_value(&mut session, "Clear[tu]; {tu[tuMark, 3], DownValues[tu]}"),
        "List[tu[tuMark, 3], List[]]"
    );

    // Invalidation: Clear on an explicit UpValue tag removes the tagged table.
    assert_eq!(
        eval_value(
            &mut session,
            "clearIndex = 5; clearTag /: clearHead[clearTag, clearIndex] = tagged; \
             Clear[clearTag]; {clearHead[clearTag, 5], UpValues[clearTag]}"
        ),
        "List[clearHead[clearTag, 5], List[]]"
    );
}

#[test]
fn upset_multiple_owner_protection_is_preserved_after_canonicalization() {
    let mut session = session_with_test_control_builtins();
    session.eval_text("Protect[protectedUpTag]").unwrap();

    let outcome = session
        .eval_text("UpSetDelayed[multiUpHead[protectedUpTag[x_], openUpTag[y_]], pair[x, y]]")
        .unwrap();
    assert_eq!(outcome.value.to_string(), "Null");
    assert!(
        outcome
            .messages
            .iter()
            .any(|message| message.symbol == "UpSetDelayed" && message.tag == "wrsym")
    );

    assert_eq!(
        eval_value(&mut session, "UpValues[protectedUpTag]"),
        "List[]"
    );
    assert_eq!(
        eval_value(
            &mut session,
            "{multiUpHead[protectedUpTag[1], openUpTag[2]], UpValues[openUpTag]}"
        ),
        "List[pair[1, 2], List[RuleDelayed[HoldPattern[multiUpHead[protectedUpTag[Pattern[x, Blank[]]], openUpTag[Pattern[y, Blank[]]]]], pair[x, y]]]]"
    );
}
