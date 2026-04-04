use super::*;

#[test]
fn names_supports_current_first_pass_string_patterns() {
    let mut session = Session::new();

    session.eval_text("pkg`alpha = 1").unwrap();
    session.eval_text("pkg`beta = 2").unwrap();
    session.eval_text("pkg`local = 4").unwrap();
    session.eval_text("pkg`llocal = 5").unwrap();
    session.eval_text("pkg`lllocal = 6").unwrap();
    session.eval_text("local = 3").unwrap();

    assert_eq!(
        session
            .eval_text("Names[\"local\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"local\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[\"pkg`a*\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[StringExpression[\"pkg`\", Alternatives[\"a*\", \"b*\"]]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[{\"pkg`a*\", \"pkg`b*\"}]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^pkg`[ab].*$\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"pkg`alpha|pkg`beta\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^pkg`(alpha|beta)$\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^pkg`(?:alpha|beta)$\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`alpha\", \"pkg`beta\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"^pkg`l{1,2}ocal$\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"pkg`llocal\", \"pkg`local\"]"
    );
}

#[test]
fn names_without_explicit_context_only_sees_visible_contexts() {
    let mut session = Session::new();

    session.eval_text("Hidden`local = 1").unwrap();

    assert_eq!(
        session
            .eval_text("Names[\"local\"]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
    assert_eq!(
        session
            .eval_text("Names[\"Hidden`local\"]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Hidden`local\"]"
    );
    assert_eq!(
        session
            .eval_text("Names[RegularExpression[\"Hidden`l.*\"]]")
            .unwrap()
            .value
            .to_string(),
        "List[\"Hidden`local\"]"
    );
}

#[test]
fn syntax_information_defaults_assignment_and_unset_cover_current_first_pass_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );

    let assigned = session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_, OptionsPattern[]}}")
        .unwrap();
    assert_eq!(
        assigned.value.to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], OptionsPattern[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], OptionsPattern[]]]]"
    );

    session.eval_text("Unset[SyntaxInformation[f]]").unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn syntax_information_participates_in_block_and_clearall() {
    let mut session = Session::new();

    session
        .eval_text("SyntaxInformation[f] = {\"LocalVariables\" -> {x}}")
        .unwrap();

    assert_eq!(
        session
            .eval_text("Block[{f}, SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_}}; SyntaxInformation[f]]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"LocalVariables\", List[x]]]"
    );

    session.eval_text("Clear[f]").unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"LocalVariables\", List[x]]]"
    );

    session.eval_text("ClearAll[f]").unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[]"
    );
}

#[test]
fn protect_blocks_direct_syntax_information_assignment_and_unset() {
    let mut session = Session::new();

    session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_}}")
        .unwrap();
    session.eval_text("Protect[f]").unwrap();

    let assigned = session
        .eval_text("SyntaxInformation[f] = {\"ArgumentsPattern\" -> {_, _}}")
        .unwrap();
    assert!(
        assigned
            .messages
            .iter()
            .any(|message| message.symbol == "Set" && message.tag == "wrsym")
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );

    let unset = session.eval_text("Unset[SyntaxInformation[f]]").unwrap();
    assert!(
        unset
            .messages
            .iter()
            .any(|message| message.symbol == "Unset" && message.tag == "wrsym")
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[f]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
}

#[test]
fn syntax_information_includes_current_system_default_subset() {
    let mut session = Session::new();

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

#[test]
fn unsetting_system_syntax_information_reverts_to_default_rules() {
    let mut session = Session::new();

    session
        .eval_text("SyntaxInformation[Options] = {\"LocalVariables\" -> {x}}")
        .unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Options]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"LocalVariables\", List[x]]]"
    );

    session
        .eval_text("Unset[SyntaxInformation[Options]]")
        .unwrap();
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Options]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
}

#[test]
fn syntax_information_covers_current_cl09_debugging_surface_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("SyntaxInformation[Trace]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[RepeatedTiming]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Short]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[On]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[BlankSequence[]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Reap]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[TimeConstrained]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[Monitor]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Blank[], OptionsPattern]], Rule[\"OptionNames\", List[\"EvaluationMonitor\", \"StepMonitor\"]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[EvaluationData]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]]]]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[TestReport]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]], OptionsPattern]], Rule[\"OptionNames\", List[\"SameTest\", \"TimeConstraint\", \"MemoryConstraint\"]]]"
    );
}
