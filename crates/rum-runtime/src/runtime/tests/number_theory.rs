use super::*;

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[test]
fn factorization_builtins_match_wolfram_kernel_integer_anchors() {
    let mut session = Session::new();

    let cases = [
        ("Divisors[12]", "List[1, 2, 3, 4, 6, 12]"),
        ("Divisors[1]", "List[1]"),
        ("Divisors[7]", "List[1, 7]"),
        ("Divisors[-12]", "List[1, 2, 3, 4, 6, 12]"),
        ("Divisors[8]", "List[1, 2, 4, 8]"),
        ("Divisors[0]", "Divisors[0]"),
        ("EulerPhi[12]", "4"),
        ("EulerPhi[1]", "1"),
        ("EulerPhi[7]", "6"),
        ("EulerPhi[8]", "4"),
        ("EulerPhi[-12]", "4"),
        ("EulerPhi[0]", "0"),
        ("MoebiusMu[1]", "1"),
        ("MoebiusMu[12]", "0"),
        ("MoebiusMu[30]", "-1"),
        ("MoebiusMu[6]", "1"),
        ("MoebiusMu[8]", "0"),
        ("MoebiusMu[0]", "0"),
        ("DivisorSigma[1, 12]", "28"),
        ("DivisorSigma[0, 12]", "6"),
        ("DivisorSigma[2, 10]", "130"),
        ("DivisorSigma[1, 8]", "15"),
        ("DivisorSigma[0, 8]", "4"),
        ("DivisorSigma[2, 8]", "85"),
        ("DivisorSigma[-1, 12]", "7/3"),
        ("DivisorSigma[-2, 12]", "35/24"),
        ("DivisorSigma[1, -12]", "28"),
        ("DivisorSigma[1, 0]", "DivisorSigma[1, 0]"),
        ("PrimeNu[12]", "2"),
        ("PrimeNu[1]", "0"),
        ("PrimeNu[8]", "1"),
        ("PrimeNu[30]", "3"),
        ("PrimeNu[-12]", "2"),
        ("PrimeNu[0]", "PrimeNu[0]"),
        ("PrimeOmega[12]", "3"),
        ("PrimeOmega[1]", "0"),
        ("PrimeOmega[8]", "3"),
        ("PrimeOmega[30]", "3"),
        ("PrimeOmega[-12]", "3"),
        ("PrimeOmega[0]", "PrimeOmega[0]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn next_prime_matches_wolfram_kernel_signed_prime_order() {
    let mut session = Session::new();

    let cases = [
        ("NextPrime[10]", "11"),
        ("NextPrime[10, 3]", "17"),
        ("NextPrime[-1]", "2"),
        ("NextPrime[-10]", "-7"),
        ("NextPrime[-10, 2]", "-5"),
        ("NextPrime[-10, -1]", "-11"),
        ("NextPrime[10, -1]", "7"),
        ("NextPrime[10, -2]", "5"),
        ("NextPrime[3, -1]", "2"),
        ("NextPrime[2, -1]", "-2"),
        ("NextPrime[1, -1]", "-2"),
        ("NextPrime[0, -1]", "-2"),
        ("NextPrime[-3, -1]", "-5"),
        ("NextPrime[10, 0]", "NextPrime[10, 0]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn factorization_builtins_thread_and_passthrough_like_current_scope() {
    let mut session = Session::new();

    let cases = [
        (
            "Divisors[{6, 7, x}]",
            "List[List[1, 2, 3, 6], List[1, 7], Divisors[x]]",
        ),
        ("EulerPhi[{12, 7, x}]", "List[4, 6, EulerPhi[x]]"),
        ("MoebiusMu[{12, 30, x}]", "List[0, -1, MoebiusMu[x]]"),
        (
            "DivisorSigma[1, {6, 7, x}]",
            "List[12, 8, DivisorSigma[1, x]]",
        ),
        ("DivisorSigma[{1, 2}, 6]", "List[12, 50]"),
        ("PrimeNu[{12, 1, x}]", "List[2, 0, PrimeNu[x]]"),
        ("PrimeOmega[{12, 1, x}]", "List[3, 0, PrimeOmega[x]]"),
        ("NextPrime[{10, 11, x}]", "List[11, 13, NextPrime[x]]"),
        ("NextPrime[{10, 11}, 2]", "List[13, 17]"),
        ("NextPrime[10, {1, 2}]", "List[11, 13]"),
        ("Divisors[3/2]", "Divisors[3/2]"),
        ("EulerPhi[x]", "EulerPhi[x]"),
        ("MoebiusMu[x]", "MoebiusMu[x]"),
        ("DivisorSigma[x, 12]", "DivisorSigma[x, 12]"),
        ("DivisorSigma[1, x]", "DivisorSigma[1, x]"),
        ("PrimeNu[3/2]", "PrimeNu[3/2]"),
        ("PrimeOmega[x]", "PrimeOmega[x]"),
        ("NextPrime[x]", "NextPrime[x]"),
        ("NextPrime[10, x]", "NextPrime[10, x]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn factorization_builtins_report_wolfram_kernel_attributes() {
    let mut session = Session::new();

    let cases = [
        ("Attributes[Divisors]", "List[Listable, Protected]"),
        ("Attributes[EulerPhi]", "List[Listable, Protected]"),
        ("Attributes[MoebiusMu]", "List[Listable, Protected]"),
        (
            "Attributes[DivisorSigma]",
            "List[Listable, NHoldAll, Protected]",
        ),
        ("Attributes[PrimeNu]", "List[Protected, ReadProtected]"),
        ("Attributes[PrimeOmega]", "List[Protected, ReadProtected]"),
        ("Attributes[NextPrime]", "List[Protected, ReadProtected]"),
    ];

    for (source, expected) in cases {
        assert_eq!(eval_to_string(&mut session, source), expected, "{source}");
    }
}

#[test]
fn large_semiprime_factorization_uses_pollard_rho_path() {
    let mut session = Session::new();
    let semiprime = "1000003 * 1000033";

    assert_eq!(
        eval_to_string(&mut session, &format!("PrimeNu[{semiprime}]")),
        "2"
    );
    assert_eq!(
        eval_to_string(&mut session, &format!("PrimeOmega[{semiprime}]")),
        "2"
    );
    assert_eq!(
        eval_to_string(&mut session, &format!("Divisors[{semiprime}]")),
        "List[1, 1000003, 1000033, 1000036000099]"
    );
}

#[test]
fn factorization_respects_integer_work_limit() {
    let mut session = Session::new();
    session.set_integer_work_limit(1);

    let limited = session.eval_text("Divisors[12]").unwrap();
    assert_eq!(
        limited.value.to_string(),
        "TerminatedEvaluation[\"IntegerWorkLimit\"]"
    );
    assert!(
        limited
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "reslim")
    );
}
