use crate::install_core;
use rum_runtime::Session;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn integer_digit_builtins_integer_anchors() {
    let mut session = core_session();

    let cases = [
        ("IntegerLength[12345]", "5"),
        ("IntegerLength[0]", "0"),
        ("IntegerLength[-12]", "2"),
        ("IntegerLength[255, 16]", "2"),
        ("IntegerDigits[12345]", "List[1, 2, 3, 4, 5]"),
        ("IntegerDigits[0]", "List[0]"),
        ("IntegerDigits[-12]", "List[1, 2]"),
        ("IntegerDigits[255, 16]", "List[15, 15]"),
        ("IntegerDigits[5, 2, 8]", "List[0, 0, 0, 0, 0, 1, 0, 1]"),
        ("IntegerDigits[12345, 10, 3]", "List[3, 4, 5]"),
        ("IntegerDigits[5, 2, 0]", "List[]"),
        ("IntegerReverse[1230]", "321"),
        ("IntegerReverse[0]", "0"),
        ("IntegerReverse[100, 2]", "19"),
        ("IntegerReverse[-120]", "21"),
        ("FromDigits[{1, 2, 3}]", "123"),
        ("FromDigits[{1, 0, 1}, 2]", "5"),
        ("FromDigits[\"123\"]", "123"),
        ("FromDigits[\"ff\", 16]", "255"),
        ("FromDigits[\"FF\", 16]", "255"),
        ("FromDigits[{15, 15}, 16]", "255"),
        ("FromDigits[{2}, 2]", "2"),
        ("FromDigits[{-1, 2}, 10]", "-8"),
        ("DigitCount[121]", "List[2, 1, 0, 0, 0, 0, 0, 0, 0, 0]"),
        ("DigitCount[0]", "List[0, 0, 0, 0, 0, 0, 0, 0, 0, 1]"),
        ("DigitCount[-121]", "List[2, 1, 0, 0, 0, 0, 0, 0, 0, 0]"),
        (
            "DigitCount[255, 16]",
            "List[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0]",
        ),
        ("DigitCount[121, 10, 1]", "2"),
        ("DigitCount[121, 10, 0]", "0"),
        ("DigitCount[121, 10, {1, 2}]", "List[2, 1]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn integer_digit_builtins_thread_and_passthrough_current_scope() {
    let mut session = core_session();

    let cases = [
        ("IntegerLength[{12, 345}]", "List[2, 3]"),
        (
            "IntegerDigits[{12, 345}]",
            "List[List[1, 2], List[3, 4, 5]]",
        ),
        ("IntegerReverse[{12, 345}]", "List[21, 543]"),
        ("IntegerReverse[12, {10, 2}]", "List[21, 3]"),
        (
            "DigitCount[{12, 345}]",
            "List[List[1, 1, 0, 0, 0, 0, 0, 0, 0, 0], List[0, 0, 1, 1, 1, 0, 0, 0, 0, 0]]",
        ),
        ("DigitCount[{12, 345}, 10, 1]", "List[1, 0]"),
        ("IntegerLength[x]", "IntegerLength[x]"),
        ("IntegerLength[3/2]", "IntegerLength[3/2]"),
        ("IntegerDigits[x]", "IntegerDigits[x]"),
        ("IntegerDigits[3/2]", "IntegerDigits[3/2]"),
        ("IntegerReverse[x]", "IntegerReverse[x]"),
        ("IntegerReverse[3/2]", "IntegerReverse[3/2]"),
        ("FromDigits[x]", "FromDigits[x]"),
        ("FromDigits[{1, x}]", "FromDigits[List[1, x]]"),
        ("DigitCount[x]", "DigitCount[x]"),
        ("DigitCount[3/2]", "DigitCount[3/2]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn integer_digit_builtins_report_declared_attributes() {
    let mut session = core_session();

    let cases = [
        ("Attributes[IntegerLength]", "List[Listable, Protected]"),
        ("Attributes[IntegerDigits]", "List[Listable, Protected]"),
        (
            "Attributes[IntegerReverse]",
            "List[Protected, ReadProtected]",
        ),
        ("Attributes[FromDigits]", "List[Protected]"),
        ("Attributes[DigitCount]", "List[Protected, ReadProtected]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}
