use super::*;

#[test]
fn syntax_information_covers_current_cl09_debugging_surface_subset() {
    let mut session = Session::new();

    assert_eq!(
        session
            .eval_text("SyntaxInformation[Trace]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]]]], Rule[\"OptionNames\", List[\"Heads\"]]]"
    );
    assert_eq!(
        session
            .eval_text("Options[Trace]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, False]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[TraceDetail]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]]]], Rule[\"OptionNames\", List[\"Heads\"]]]"
    );
    assert_eq!(
        session
            .eval_text("Options[TraceDetail]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, False]]"
    );
    assert_eq!(
        session
            .eval_text("SyntaxInformation[TracePrint]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Blank[], Optional[Blank[]]]], Rule[\"OptionNames\", List[\"Heads\"]]]"
    );
    assert_eq!(
        session
            .eval_text("Options[TracePrint]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[Heads, False]]"
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
            .eval_text("SyntaxInformation[Dialog]")
            .unwrap()
            .value
            .to_string(),
        "List[Rule[\"ArgumentsPattern\", List[Optional[Blank[]], OptionsPattern]], Rule[\"OptionNames\", List[\"DialogProlog\", \"DialogSymbols\"]]]"
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
