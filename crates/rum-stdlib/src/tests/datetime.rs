use crate::install_core;
use rum_core::Expr;
use rum_runtime::{RuntimeMessage, Session};

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn eval_value_and_messages(session: &mut Session, source: &str) -> (Expr, Vec<RuntimeMessage>) {
    let outcome = session.eval_text(source).unwrap();
    (outcome.value, outcome.messages)
}

fn assert_failed_with_message(session: &mut Session, source: &str, symbol: &str) {
    let (value, messages) = eval_value_and_messages(session, source);
    assert_eq!(value.to_string(), "$Failed", "source {source}");
    assert!(
        messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == "date"),
        "source {source} messages {messages:?}"
    );
}

#[test]
fn absolute_time_and_date_list_epoch_pins_are_utc() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "AbsoluteTime[{1970,1,1,0,0,0}]"),
        "0"
    );
    assert_eq!(
        eval_to_string(&mut session, "DateList[0]"),
        "List[1970, 1, 1, 0, 0, 0]"
    );
    assert_eq!(
        eval_to_string(&mut session, "DateList[86400]"),
        "List[1970, 1, 2, 0, 0, 0]"
    );
    assert_eq!(
        eval_to_string(&mut session, "AbsoluteTime[{2000,2,29,0,0,0}]"),
        "951782400"
    );
    assert_eq!(
        eval_to_string(&mut session, "DateList[-1]"),
        "List[1969, 12, 31, 23, 59, 59]"
    );
}

#[test]
fn date_list_round_trips_exact_rational_timestamps() {
    let mut session = core_session();

    for timestamp in ["0", "1/3", "-1/3", "86400*20000 + 1/3", "-86400*10 + 1/3"] {
        assert_eq!(
            eval_to_string(
                &mut session,
                &format!("AbsoluteTime[DateList[{timestamp}]] == {timestamp}")
            ),
            "True",
            "timestamp {timestamp}"
        );
    }

    assert_eq!(
        eval_to_string(&mut session, "DateList[AbsoluteTime[{2026,7,7,1,2,1/3}]]"),
        "List[2026, 7, 7, 1, 2, 1/3]"
    );
}

#[test]
fn leap_year_and_month_length_edges_are_validated() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{LeapYearQ[1900], LeapYearQ[2000], DaysInMonth[1900,2], DaysInMonth[2000,2]}"
        ),
        "List[False, True, 28, 29]"
    );
    assert_ne!(
        eval_to_string(&mut session, "AbsoluteTime[{2000,2,29}]"),
        "$Failed"
    );
    assert_failed_with_message(&mut session, "AbsoluteTime[{1900,2,29}]", "AbsoluteTime");
    assert_failed_with_message(&mut session, "AbsoluteTime[{2026,1,32}]", "AbsoluteTime");
    assert_failed_with_message(
        &mut session,
        "AbsoluteTime[{2026,1,1,24,0,0}]",
        "AbsoluteTime",
    );
}

#[test]
fn from_date_string_accepts_documented_iso_subset_and_offsets() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{FromDateString[\"1970-01-01\"], FromDateString[\"1970-01-01T00:00:00\"], FromDateString[\"1970-01-01T00:00:00Z\"], FromDateString[\"1970-01-01T00:00:00.5Z\"]}"
        ),
        "List[0, 0, 0, 1/2]"
    );
    assert_eq!(
        eval_to_string(
            &mut session,
            "{FromDateString[\"1970-01-01T01:00:00+01:00\"], FromDateString[\"1969-12-31T19:00:00-05:00\"]}"
        ),
        "List[0, 0]"
    );
}

#[test]
fn from_date_string_fails_closed_on_invalid_shapes() {
    let mut session = core_session();

    for source in [
        "FromDateString[\"2026-13-01\"]",
        "FromDateString[\"2026-01-01x\"]",
        "FromDateString[\"2026-07\"]",
        "FromDateString[\"2026-01-01T24:00:00\"]",
        "FromDateString[\"2026-01-01T12:00\"]",
    ] {
        assert_failed_with_message(&mut session, source, "FromDateString");
    }
}

#[test]
fn date_string_renders_utc_and_rejects_non_terminating_fractional_seconds() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{DateString[0], DateString[{2026,7,7,12,34,56}], DateString[{1970,1,1,0,0,1/2}]}"
        ),
        "List[\"1970-01-01T00:00:00Z\", \"2026-07-07T12:34:56Z\", \"1970-01-01T00:00:00.5Z\"]"
    );
    assert_failed_with_message(&mut session, "DateString[1/3]", "DateString");
}

#[test]
fn date_plus_and_date_difference_use_exact_fixed_duration_arithmetic() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{DatePlus[0, {36, \"Hours\"}], DatePlus[{1970,1,1}, {1/2, \"Seconds\"}], DateDifference[0, 86400*7, \"Weeks\"], DateDifference[0, 1, \"Minutes\"]}"
        ),
        "List[129600, 1/2, 1, 1/60]"
    );
    assert_failed_with_message(&mut session, "DatePlus[0, {1, \"Months\"}]", "DatePlus");
}

#[test]
fn day_of_week_returns_iso_weekday_symbols() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "{DayOfWeek[0], DayOfWeek[{2000,2,29}]}"),
        "List[Thursday, Tuesday]"
    );
}
