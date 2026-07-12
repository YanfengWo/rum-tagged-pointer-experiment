use super::super::*;

// Moved from crates/rum-runtime/src/runtime/tests/options_and_protection.rs.

#[test]
fn delayed_system_default_options_feed_current_builtin_consumers() {
    let mut session = core_session();
    session
        .eval_text("Unprotect[StringContainsQ, Depth]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("SetOptions[StringContainsQ, IgnoreCase :> True]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[IgnoreCase, True]]"
    );
    assert_eq!(
        session
            .eval_text("StringContainsQ[\"Ab\", \"a\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );

    assert_eq!(
        session
            .eval_text("SetOptions[Depth, Heads :> True]")
            .unwrap()
            .value
            .to_string(),
        "List[RuleDelayed[Heads, True]]"
    );
    assert_eq!(
        session
            .eval_text("Depth[f[g][x]]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/options_and_protection.rs.

#[test]
fn setoptions_updates_current_system_default_subset_behavior() {
    let mut session = core_session();
    session
        .eval_text("Unprotect[StringContainsQ, StringCount]")
        .unwrap();

    assert_eq!(
        session
            .eval_text("SetOptions[StringContainsQ, IgnoreCase -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, True]]"
    );
    assert_eq!(
        session
            .eval_text("StringContainsQ[\"Ab\", \"a\"]")
            .unwrap()
            .value
            .to_string(),
        "True"
    );
    assert_eq!(
        session
            .eval_text("StringContainsQ[\"Ab\", \"a\", IgnoreCase -> False]")
            .unwrap()
            .value
            .to_string(),
        "False"
    );

    assert_eq!(
        session
            .eval_text("SetOptions[StringCount, Overlaps -> True]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[Overlaps, True]]"
    );
    assert_eq!(
        session
            .eval_text("StringCount[\"aaaa\", \"aa\"]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}

// Moved from crates/rum-runtime/src/runtime/tests/options_and_protection.rs.

#[test]
fn system_default_options_cover_current_first_pass_subset() {
    let mut session = core_session();

    assert_eq!(
        session
            .eval_text("Options[Cases]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[FreeQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, True]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Position]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, True]]"
    );
    assert_eq!(
        session
            .eval_text(
                "Options /@ {Map, MapIndexed, MapAll, Scan, Apply, MapApply, Comap, ComapApply}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[List[Rule[Heads, False]], List[Rule[Heads, False]], List[Rule[Heads, False]], List[Rule[Heads, False]], List[Rule[Heads, False]], List[Rule[Heads, False]], List[Rule[Heads, False]], List[Rule[Heads, False]]]"
    );
    assert_eq!(
        session
            .eval_text("Options[StringCount]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[Overlaps, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[StringCases]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[Overlaps, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[StringPosition]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[Overlaps, True]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Nearest]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[DistanceFunction, Automatic]]"
    );
    assert_eq!(
        session
            .eval_text("Options[CheckAbort]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[PropagateAborts, Automatic]]"
    );
    assert_eq!(
        session
            .eval_text("Options[InputString]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[FieldMasked, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Input]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[FieldMasked, False]]"
    );
    assert_eq!(
        session
            .eval_text("Options[ArrayFilter]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Padding, \"Fixed\"]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Save]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[ExcludedContexts, Automatic], Rule[IncludedContexts, All]]"
    );
    assert_eq!(
        session
            .eval_text("Options[DumpSave]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[CompressionLevel, 1], Rule[SymbolAttributes, True]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Once]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[ExpirationDate, None], Rule[PersistenceTime, None]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Names]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[IgnoreCase, False], Rule[ResolveContextAliases, True], Rule[SpellingCorrection, False]]"
    );
}
