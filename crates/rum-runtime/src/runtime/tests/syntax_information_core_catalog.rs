use super::*;

#[test]
fn syntax_information_includes_current_system_default_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("SyntaxInformation[Names]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Optional[Blank[]], OptionsPattern]], Rule[\"OptionNames\", List[\"IgnoreCase\", \"ResolveContextAliases\", \"SpellingCorrection\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Options]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[SetOptions]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], BlankNullSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[ArgumentsOptions]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], Optional[Blank[Association]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Protect]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[BlankNullSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Unprotect]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[BlankSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[DeclarePackage]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], List[BlankSequence[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Begin]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[BeginPackage]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], BlankNullSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[End]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[EndPackage]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Needs]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[MessageName]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Message]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[BlankSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Module]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[List[BlankSequence[]], Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Block]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[List[BlankSequence[]], Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Function]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Table]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], BlankSequence[]]], Rule[\"LocalVariables\", List[\"Table\", List[2, Infinity]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Do]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], BlankSequence[]]], Rule[\"LocalVariables\", List[\"Table\", List[2, Infinity]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Save]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]], Rule[\"OptionNames\", List[\"ExcludedContexts\", \"IncludedContexts\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[DumpSave]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]], Rule[\"OptionNames\", List[\"CompressionLevel\", \"SymbolAttributes\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Once]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], OptionsPattern]], Rule[\"OptionNames\", List[\"ExpirationDate\", \"PersistenceTime\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringCount]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringContainsQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringPosition]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringCases]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringDelete]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[StringReplace]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Cases]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Count]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[MemberQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[FreeQ]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Position]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Level]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], Optional[Blank[]], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Depth]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[CheckAbort]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[ArrayFilter]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], Blank[], OptionsPattern]]]"
    );
}
