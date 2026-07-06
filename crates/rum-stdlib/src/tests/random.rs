use crate::install_core;
use rum_runtime::{
    Session,
    kernel_api::{EntropyError, EntropyProvider, EntropySource},
};
use std::collections::VecDeque;
use std::sync::Mutex;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn core_session_with_entropy(bytes: &[u8]) -> Session {
    let mut session = core_session();
    session.set_entropy_source(Some(SequentialEntropy::provider(bytes)));
    session
}

fn eval_to_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

#[derive(Debug)]
struct SequentialEntropy {
    bytes: Mutex<VecDeque<u8>>,
}

impl SequentialEntropy {
    fn provider(bytes: &[u8]) -> EntropyProvider {
        EntropyProvider::new(Self {
            bytes: Mutex::new(bytes.iter().copied().collect()),
        })
    }
}

impl EntropySource for SequentialEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyError> {
        let mut bytes = self.bytes.lock().unwrap();
        for slot in buf {
            let Some(byte) = bytes.pop_front() else {
                return Err(EntropyError::new("fixed entropy exhausted"));
            };
            *slot = byte;
        }
        Ok(())
    }
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
fn random_string_default_seed_sequence_is_pinned() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "{RandomString[8], RandomString[\"abc\", 5]}"),
        "List[\"0qgsZKYx\", \"babaa\"]"
    );
}

#[test]
fn random_string_seed_random_reproducibility_is_pinned() {
    let mut first = core_session();
    let mut second = core_session();
    let source = "SeedRandom[42]; RandomString[8]";

    assert_eq!(eval_to_string(&mut first, source), "\"7nKuuJoy\"");
    assert_eq!(
        eval_to_string(&mut first, source),
        eval_to_string(&mut second, source)
    );
}

#[test]
fn random_string_alphabet_duplicates_weight_draws() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "SeedRandom[42]; RandomString[\"aaab\", 16]"),
        "\"bbaaaaaaaaaaaaab\""
    );
}

#[test]
fn random_string_draws_from_unicode_code_points() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "SeedRandom[42]; RandomString[\"日本語\", 12]"),
        "\"語語語本日語語日日日本語\""
    );
}

#[test]
fn random_string_invalid_count_or_empty_alphabet_stays_inert() {
    let mut session = core_session();

    assert_eq!(
        eval_to_string(&mut session, "{RandomString[-1], RandomString[\"\", 3]}"),
        "List[RandomString[-1], RandomString[\"\", 3]]"
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
fn block_random_restores_the_random_string_stream() {
    let mut with_block = core_session();
    let mut without_block = core_session();

    assert_eq!(
        eval_to_string(
            &mut with_block,
            "SeedRandom[123]; {RandomString[4], BlockRandom[RandomString[4]], RandomString[4]}"
        ),
        "List[\"laD2\", \"jatK\", \"jatK\"]"
    );
    assert_eq!(
        eval_to_string(
            &mut without_block,
            "SeedRandom[123]; {RandomString[4], RandomString[4]}"
        ),
        "List[\"laD2\", \"jatK\"]"
    );
}

#[test]
fn random_password_fixed_entropy_uses_rejection_sampling() {
    let mut session = core_session_with_entropy(&[248, 0, 249, 61, 250, 62, 63]);

    assert_eq!(
        eval_to_string(&mut session, "RandomPassword[4]"),
        "\"A9AB\""
    );
}

#[test]
fn random_password_respects_custom_alphabet_membership() {
    let mut session = core_session_with_entropy(&[0, 1, 2, 3, 4, 5]);

    assert_eq!(
        eval_to_string(&mut session, "RandomPassword[\"abc\", 6]"),
        "\"abcabc\""
    );
}

#[test]
fn random_password_does_not_advance_the_prng_stream() {
    let mut session = core_session_with_entropy(&[0, 1, 2, 3, 4, 5, 6, 7]);

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[7]; first = RandomString[8]; password = RandomPassword[8]; after = RandomString[8]; SeedRandom[7]; {first, password, after, RandomString[8], RandomString[8]}"
        ),
        "List[\"XVoZyDqB\", \"ABCDEFGH\", \"t0CnBKji\", \"XVoZyDqB\", \"t0CnBKji\"]"
    );
}

#[test]
fn block_random_does_not_restore_random_password_entropy() {
    let mut session = core_session_with_entropy(&[0, 1, 2, 3, 4, 5, 6, 7]);

    assert_eq!(
        eval_to_string(
            &mut session,
            "SeedRandom[123]; {BlockRandom[RandomPassword[4]], RandomPassword[4], RandomString[4], SeedRandom[123]; RandomString[4]}"
        ),
        "List[\"ABCD\", \"EFGH\", \"laD2\", \"laD2\"]"
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
