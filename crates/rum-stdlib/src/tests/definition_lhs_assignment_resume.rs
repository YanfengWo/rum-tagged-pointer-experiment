use crate::install_core;
use rum_runtime::{Session, SessionRequestError};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn unset_lhs_preparation_resumes_exactly_once_without_replay() {
    let mut session = core_session();
    session
        .eval_text(
            "ClearAll[unsetResumeHead, unsetResumeArg, unsetResumeCalls]; \
             unsetResumeCalls = 0; unsetResumeHead[\"drop\"] = old; \
             unsetResumeArg[] := (unsetResumeCalls = unsetResumeCalls + 1; \
                                  InputString[\"unset lhs: \"])",
        )
        .unwrap();

    let failure = session
        .eval_request("Unset[unsetResumeHead[unsetResumeArg[]]]")
        .expect_err("Unset lhs preparation should request input");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "unset lhs: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("drop")
        .expect("the resumed Unset lhs should remove the prepared identity");
    assert_eq!(response.value.to_string(), "Null");
    assert!(response.messages.is_empty());
    assert_eq!(
        session
            .eval_text("{unsetResumeCalls, unsetResumeHead[\"drop\"], DownValues[unsetResumeHead]}")
            .unwrap()
            .value
            .to_string(),
        "List[1, unsetResumeHead[\"drop\"], List[]]"
    );
}

#[test]
fn value_table_lhs_preparation_resumes_multiple_entries_without_replay() {
    let mut session = core_session();
    session
        .eval_text(
            "ClearAll[tableResumeHead, tableResumeFirst, tableResumeSecond, \
                      tableResumeFirstCalls, tableResumeSecondCalls, tableResumeOrder]; \
             tableResumeHead[0] := old; tableResumeFirstCalls = 0; \
             tableResumeSecondCalls = 0; tableResumeOrder = 0; \
             tableResumeFirst[] := (tableResumeFirstCalls = tableResumeFirstCalls + 1; \
                                    tableResumeOrder = tableResumeOrder * 10 + 1; \
                                    InputString[\"table first: \"]); \
             tableResumeSecond[] := (tableResumeSecondCalls = tableResumeSecondCalls + 1; \
                                     tableResumeOrder = tableResumeOrder * 10 + 2; \
                                     InputString[\"table second: \"])",
        )
        .unwrap();

    let first = session
        .eval_request(
            "DownValues[tableResumeHead] = \
             {HoldPattern[tableResumeHead[tableResumeFirst[]]] :> first, \
              HoldPattern[tableResumeHead[tableResumeSecond[]]] :> second}",
        )
        .expect_err("the first value-table lhs should request input first");
    match &first.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "table first: ");
        }
        other => panic!("expected first input-required failure, got {other:?}"),
    }

    let second = session
        .resume_eval_request_with_input("one")
        .expect_err("resuming the first table lhs should advance to the second");
    match &second.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "table second: ");
        }
        other => panic!("expected second input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("two")
        .expect("the second response should atomically install the prepared table");
    assert!(response.messages.is_empty());
    assert_eq!(
        session
            .eval_text(
                "{tableResumeFirstCalls, tableResumeSecondCalls, tableResumeOrder, \
                  tableResumeHead[\"one\"], tableResumeHead[\"two\"], \
                  tableResumeHead[0], DownValues[tableResumeHead]}"
            )
            .unwrap()
            .value
            .to_string(),
        "List[1, 1, 12, first, second, tableResumeHead[0], List[RuleDelayed[HoldPattern[tableResumeHead[\"one\"]], first], RuleDelayed[HoldPattern[tableResumeHead[\"two\"]], second]]]"
    );
}

#[test]
fn value_table_failure_after_suspension_keeps_the_old_table_intact() {
    let mut session = core_session();
    session
        .eval_text(
            "ClearAll[transactionTable, invalidTableHead, invalidTableHeadCalls]; \
             transactionTable[0] := old; invalidTableHeadCalls = 0; \
             invalidTableHead[] := (invalidTableHeadCalls = invalidTableHeadCalls + 1; \
                                    Input[\"table invalid head: \"])",
        )
        .unwrap();
    let before = session
        .eval_text("DownValues[transactionTable]")
        .unwrap()
        .value;

    let failure = session
        .eval_request(
            "DownValues[transactionTable] = \
             {HoldPattern[transactionTable[1]] :> first, \
              HoldPattern[invalidTableHead[][2]] :> invalid}",
        )
        .expect_err("the late value-table lhs should suspend before validation");
    match &failure.error {
        SessionRequestError::InputRequired { request } => {
            assert_eq!(request.prompt, "table invalid head: ");
        }
        other => panic!("expected input-required failure, got {other:?}"),
    }

    let response = session
        .resume_eval_request_with_input("otherTableHead")
        .expect("invalid prepared ownership should fail closed as a normal response");
    assert!(response.messages.iter().any(|message| {
        message.symbol == "DownValues"
            && message.tag == "setval"
            && message.text.contains("does not define the target")
    }));
    assert_eq!(
        session
            .eval_text("{invalidTableHeadCalls, transactionTable[0], transactionTable[1]}")
            .unwrap()
            .value
            .to_string(),
        "List[1, old, transactionTable[1]]"
    );
    assert_eq!(
        session
            .eval_text("DownValues[transactionTable]")
            .unwrap()
            .value,
        before
    );
}
