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
fn random_integer_default_seed_sequence_is_pinned() {
    let mut session = core_session();
    let mut zero_seed_session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "{RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100]}"
        ),
        "List[52, 42, 96, 44, 89]"
    );
    assert_eq!(
        eval_to_string(
            &mut zero_seed_session,
            "SeedRandom[0]; {RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100]}"
        ),
        "List[52, 42, 96, 44, 89]"
    );
}

#[test]
fn seed_random_integer_sequence_is_pinned() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[42]; {RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100], RandomInteger[100]}"
        ),
        "List[39, 10, 46, 9, 40]"
    );
}

#[test]
fn random_integer_respects_large_exact_ranges() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[7]; r = RandomInteger[{2^80, 2^80 + 1000}]; {r >= 2^80, r <= 2^80 + 1000}"
        ),
        "List[True, True]"
    );
}

#[test]
fn random_choice_and_sample_cover_current_list_contracts() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[42]; {RandomChoice[{a,b,c}, 5], RandomSample[Range[10]]}"
        ),
        "List[List[c, c, c, b, a], List[3, 4, 1, 8, 5, 7, 9, 10, 2, 6]]"
    );
    assert_eq!(
        eval_to_string(&mut session, "Sort[RandomSample[Range[10]]]"),
        "List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"
    );

    let empty_choice = session.eval_text("RandomChoice[{}]").unwrap();
    assert_eq!(empty_choice.value.to_string(), "RandomChoice[List[]]");
    assert!(
        empty_choice
            .messages
            .iter()
            .any(|message| message.symbol == "RandomChoice" && message.tag == "empty")
    );
}

#[test]
fn block_random_restores_the_prng_stream() {
    let mut with_block = core_session();
    let mut without_block = core_session();

    assert_eq!(
        eval_to_string(
            &mut with_block,
            "SeedRandom[123]; {RandomInteger[1000], BlockRandom[RandomInteger[1000]], RandomInteger[1000]}"
        ),
        "List[997, 474, 474]"
    );
    assert_eq!(
        eval_to_string(
            &mut without_block,
            "SeedRandom[123]; {RandomInteger[1000], RandomInteger[1000]}"
        ),
        "List[997, 474]"
    );
}

#[test]
fn random_uuid_and_uuid_q_cover_format_boundaries() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[99]; {RandomUuid[], UuidQ[RandomUuid[]], UuidQ[\"00000000-0000-4000-8000-000000000000\"], UuidQ[\"00000000-0000-5000-8000-000000000000\"], UuidQ[\"00000000-0000-4000-7000-000000000000\"], UuidQ[\"not-a-uuid\"]}"
        ),
        "List[\"cb520df9-5923-46de-ba5d-3c9f7a9cb464\", True, True, False, False, False]"
    );
}
