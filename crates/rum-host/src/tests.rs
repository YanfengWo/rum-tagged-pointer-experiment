use super::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LONG_RUNNING_EVAL: &str = "Do[x = 1, 50000000]";
const TWO_TO_200: &str = "1606938044258990275541962092341162602522202993782792835301376";

fn host_session_with_large_iteration_budget() -> HostSession {
    let mut limits = HostResourceLimits::default();
    limits.iteration_work_limit = 50_000_000;
    HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build()
}

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("rum-host-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, source: &str) -> PathBuf {
        let path = self.path.join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn successful_eval(outcome: HostEvalOutcome) -> HostEvalSuccess {
    match outcome {
        HostEvalOutcome::Success(success) => success,
        other => panic!("expected success, got {other:?}"),
    }
}

fn rum_path(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
        .replace('\\', "/")
}

fn wl_string(value: &str) -> String {
    format!("{value:?}")
}

fn byte_array_payload(text: &str) -> Option<&str> {
    text.strip_prefix("ByteArray[\"")?.strip_suffix("\"]")
}

fn string_payload(text: &str) -> Option<&str> {
    text.strip_prefix('"')?.strip_suffix('"')
}

fn assert_sandbox_denied(session: &mut HostSession, source: &str) {
    let success = successful_eval(session.eval(source));
    assert_eq!(success.value.text, "$Failed", "source {source}");
    assert!(
        success
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox"),
        "source {source} messages {:?}",
        success.messages
    );
}

fn assert_arity_not_sandbox_denial(
    session: &mut HostSession,
    source: &str,
    symbol: &str,
    tag: &str,
) {
    let success = successful_eval(session.eval(source));
    assert_eq!(success.value.text, source, "source {source}");
    assert!(
        success
            .messages
            .iter()
            .any(|message| message.symbol == symbol && message.tag == tag),
        "source {source} messages {:?}",
        success.messages
    );
    assert!(
        !success
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox"),
        "source {source} messages {:?}",
        success.messages
    );
}

#[test]
fn host_session_and_cancel_handle_are_send() {
    fn is_send<T: Send>() {}

    is_send::<rum_runtime::Session>();
    is_send::<HostSession>();
    is_send::<HostCancelHandle>();
    // HostSession is intentionally not asserted Sync: it is owned, mutable
    // session state, with Cell/FnMut internals that should not be shared.
}

#[test]
fn parallel_host_sessions_move_to_threads_and_isolate_definitions() {
    const THREADS: usize = 8;

    let handles = (0..THREADS)
        .map(|i| {
            let session = HostSession::new();
            thread::spawn(move || {
                let mut session = session;
                let source = format!("x = {i}; x * x");
                match session.eval(source) {
                    HostEvalOutcome::Success(success) => (i, success.value.text),
                    other => panic!("thread {i} expected success, got {other:?}"),
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        let (i, text) = handle.join().expect("host session worker should not panic");
        assert_eq!(text, (i * i).to_string(), "thread {i}");
        eprintln!("parallel_host_sessions thread={i} result={text}");
    }
}

#[test]
fn host_eval_reports_value_messages_prints_and_structured_outputs() {
    let mut session = HostSession::new();

    let outcome = session.eval("Print[1]; Message[General::stop]; 2");

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "2");
            assert_eq!(success.prints, vec!["1"]);
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "stop")
            );
            assert!(
                success
                    .outputs
                    .iter()
                    .any(|output| output.kind == HostStructuredOutputKind::Result)
            );
        }
        other => panic!("expected success, got {other:?}"),
    }
}

#[test]
fn sandboxed_host_config_blocks_filesystem_builtins_at_dispatch() {
    let dir = TempDirGuard::new("sandbox-blocks-fs");
    let package = dir.write("Pkg.rum", "BeginPackage[\"Pkg`\"]\nvalue = 7\nEndPackage[]");
    let input = dir.write("input.wl", "77");
    let save_path = dir.path().join("save.wl");
    let dump_path = dir.path().join("dump.mx");

    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed().with_package_search_paths([dir.path().to_path_buf()]))
        .build();

    assert_sandbox_denied(
        &mut session,
        &format!("Get[{}]", wl_string(&rum_path(&input))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("Needs[\"Pkg`\", {}]", wl_string(&rum_path(&package))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("FindFile[{}]", wl_string(&rum_path(&input))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("FileNames[\"*\", {{{}}}]", wl_string(&rum_path(dir.path()))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("x = 1; Save[{}, x]", wl_string(&rum_path(&save_path))),
    );
    assert_sandbox_denied(
        &mut session,
        &format!("x = 1; DumpSave[{}, x]", wl_string(&rum_path(&dump_path))),
    );
}

#[test]
fn trusted_host_file_names_uses_the_std_file_io_provider() {
    let dir = TempDirGuard::new("trusted-file-names");
    let b = dir.write("b.wl", "b");
    let a = dir.write("a.wl", "a");
    dir.write("c.txt", "c");
    let mut session = HostSession::new();

    let success = successful_eval(session.eval(format!(
        "FileNames[\"*.wl\", {{{}}}]",
        wl_string(&rum_path(dir.path()))
    )));

    assert_eq!(
        success.value.text,
        format!(
            "List[{}, {}]",
            wl_string(&rum_path(&a)),
            wl_string(&rum_path(&b))
        )
    );
    assert!(success.messages.is_empty(), "{:?}", success.messages);
}

#[test]
fn sandboxed_host_config_blocks_interactive_builtins_at_dispatch() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut session, "Input[]");
    assert_sandbox_denied(&mut session, "InputString[]");
    assert_sandbox_denied(&mut session, "Dialog[1]");
}

#[test]
fn sandboxed_host_config_with_file_provider_blocks_internal_loader_calls() {
    let dir = TempDirGuard::new("sandbox-blocks-internal-loads");
    dir.write(
        "DepPkg.rum",
        "BeginPackage[\"DepPkg`\"]\ndependencyValue = 33\nEndPackage[]",
    );
    dir.write(
        "StubPkg.rum",
        "BeginPackage[\"StubPkg`\"]\nautoloadValue = 44\nEndPackage[]",
    );

    let mut session = HostSession::builder()
        .with_config(
            HostConfig::sandboxed()
                .with_file_io(StdFileIo::provider())
                .with_package_search_paths([dir.path().to_path_buf()]),
        )
        .build();

    let begin_package = successful_eval(session.eval("BeginPackage[\"MainPkg`\", {\"DepPkg`\"}]"));
    assert_eq!(begin_package.value.text, "$Failed");
    assert!(begin_package.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "sandbox"
            && message.text == "Needs is not permitted in this sandbox."
    }));
    let context = successful_eval(session.eval("$Context")).value.text;
    let context_path = successful_eval(session.eval("$ContextPath")).value.text;
    let packages = successful_eval(session.eval("$Packages")).value.text;
    eprintln!(
        "restricted+provider BeginPackage value={} messages={:?} context={} context_path={} packages={}",
        begin_package.value.text, begin_package.messages, context, context_path, packages
    );
    assert_eq!(context, "\"Global`\"");
    assert_eq!(context_path, "List[\"System`\", \"Global`\"]");
    assert_eq!(packages, "List[]");

    successful_eval(session.eval("DeclarePackage[\"StubPkg`\", {\"autoloadValue\"}]"));
    let declared_stub = successful_eval(session.eval("StubPkg`autoloadValue"));
    assert_eq!(declared_stub.value.text, "$Failed");
    assert!(declared_stub.messages.iter().any(|message| {
        message.symbol == "General"
            && message.tag == "sandbox"
            && message.text == "Needs is not permitted in this sandbox."
    }));
    let packages = successful_eval(session.eval("$Packages")).value.text;
    eprintln!(
        "restricted+provider declared-stub value={} messages={:?} packages={}",
        declared_stub.value.text, declared_stub.messages, packages
    );
    assert_eq!(packages, "List[]");
}

#[test]
fn sandboxed_host_config_blocks_runtime_constructed_filesystem_calls() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut session, "Symbol[\"Get\"][\"/etc/passwd\"]");

    let runtime_constructed = format!(
        "ToExpression[{}]",
        wl_string(&format!("Get[{}]", wl_string("/etc/passwd")))
    );
    assert_sandbox_denied(&mut session, &runtime_constructed);
}

#[test]
fn sandboxed_host_config_blocks_diagnostic_builtins_at_dispatch() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut session, "RumProfileSnapshot[]");
}

#[test]
fn sandboxed_host_config_blocks_clock_builtins_at_dispatch() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    let success = successful_eval(session.eval("AbsoluteTime[]"));
    eprintln!(
        "sandboxed AbsoluteTime[] value={} messages={:?}",
        success.value.text, success.messages
    );
    assert_eq!(success.value.text, "$Failed");
    assert!(
        success
            .messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox")
    );
}

#[test]
fn sandboxed_host_config_blocks_exit_at_dispatch_but_trusted_allows_it() {
    let mut sandboxed = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut sandboxed, "Exit[0]");

    let mut trusted = HostSession::new();
    match trusted.eval("Exit[7]") {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Exit);
            assert_eq!(failure.message, "Exit[7]");
        }
        other => panic!("expected trusted Exit failure, got {other:?}"),
    }
}

#[test]
fn sandboxed_host_config_still_allows_pure_work_and_path_string_helpers() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_eq!(successful_eval(session.eval("2 + 3")).value.text, "5");
    assert_eq!(
        successful_eval(session.eval("Reverse[{1, 2, 3}]"))
            .value
            .text,
        "List[3, 2, 1]"
    );
    assert_eq!(
        successful_eval(session.eval("FileNameJoin[{\"alpha\", \"beta.wl\"}]"))
            .value
            .text,
        "\"alpha/beta.wl\""
    );
    assert_eq!(
        successful_eval(session.eval("DirectoryName[\"/tmp/a/b.txt\"]"))
            .value
            .text,
        "\"/tmp/a/\""
    );
    assert_eq!(
        successful_eval(session.eval("ParentDirectory[\"/tmp/a/b/\"]"))
            .value
            .text,
        "\"/tmp/a/\""
    );
    assert_eq!(
        successful_eval(session.eval("FileNameTake[\"/tmp/a/b.txt\"]"))
            .value
            .text,
        "\"b.txt\""
    );
}

#[test]
fn capability_policy_can_allow_reads_without_writes() {
    let dir = TempDirGuard::new("sandbox-read-only");
    let input = dir.write("input.wl", "loaded = 42\nloaded");
    let save_path = dir.path().join("save.wl");
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::FileRead]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    assert_eq!(
        successful_eval(session.eval(format!("Get[{}]", wl_string(&rum_path(&input)))))
            .value
            .text,
        "42"
    );
    assert_sandbox_denied(
        &mut session,
        &format!("Save[{}, loaded]", wl_string(&rum_path(&save_path))),
    );
    assert_sandbox_denied(&mut session, "RumProfileSnapshot[]");
}

#[test]
fn capability_policy_can_allow_clock_without_filesystem() {
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Clock]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    let absolute_time = successful_eval(session.eval("AbsoluteTime[]"));
    eprintln!(
        "clock policy AbsoluteTime[] value={}",
        absolute_time.value.text
    );
    assert_ne!(absolute_time.value.text, "$Failed");

    let get = successful_eval(session.eval("Get[\"/etc/passwd\"]"));
    eprintln!(
        "clock policy Get[] value={} messages={:?}",
        get.value.text, get.messages
    );
    assert_eq!(get.value.text, "$Failed");
    assert!(
        get.messages
            .iter()
            .any(|message| message.symbol == "General" && message.tag == "sandbox")
    );
}

#[test]
fn sandboxed_host_config_allows_pure_datetime_forms() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_arity_not_sandbox_denial(&mut session, "DateList[]", "DateList", "argx");
    assert_arity_not_sandbox_denial(&mut session, "DateString[]", "DateString", "argx");

    let date_list = successful_eval(session.eval("DateList[0]"));
    assert_eq!(date_list.value.text, "List[1970, 1, 1, 0, 0, 0]");
    assert!(date_list.messages.is_empty(), "{:?}", date_list.messages);

    let date_string = successful_eval(session.eval("DateString[0]"));
    assert_eq!(date_string.value.text, "\"1970-01-01T00:00:00Z\"");
    assert!(
        date_string.messages.is_empty(),
        "{:?}",
        date_string.messages
    );
}

#[test]
fn clock_policy_allows_composed_datetime_current_readers() {
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Clock]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    let date_list = successful_eval(session.eval("DateList[AbsoluteTime[]]"));
    assert_ne!(date_list.value.text, "$Failed");
    assert!(date_list.messages.is_empty(), "{:?}", date_list.messages);

    let date_string = successful_eval(session.eval("DateString[AbsoluteTime[]]"));
    assert_ne!(date_string.value.text, "$Failed");
    assert!(
        date_string.messages.is_empty(),
        "{:?}",
        date_string.messages
    );
}

#[test]
fn permissive_policy_without_file_provider_returns_noio_failed() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().without_file_io())
        .build();

    let success = successful_eval(session.eval("Get[\"/nonexistent\"]"));

    assert_eq!(success.value.text, "$Failed");
    assert_eq!(success.messages.len(), 1);
    assert_eq!(success.messages[0].symbol, "General");
    assert_eq!(success.messages[0].tag, "noio");
    assert_eq!(success.messages[0].text, "no file provider installed");
    eprintln!(
        "no_provider_get value={} messages={:?}",
        success.value.text, success.messages
    );

    let file_names = successful_eval(session.eval("FileNames[]"));
    assert_eq!(file_names.value.text, "$Failed");
    assert_eq!(file_names.messages.len(), 1);
    assert_eq!(file_names.messages[0].symbol, "General");
    assert_eq!(file_names.messages[0].tag, "noio");
    assert_eq!(file_names.messages[0].text, "no file provider installed");
}

#[test]
fn sandboxed_host_config_blocks_entropy_builtins_but_allows_prng_builtins() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::sandboxed())
        .build();

    assert_sandbox_denied(&mut session, "ReseedRandom[]");
    assert_sandbox_denied(&mut session, "RandomBytes[8]");
    assert_sandbox_denied(&mut session, "RandomPassword[8]");
    assert_arity_not_sandbox_denial(&mut session, "SeedRandom[]", "SeedRandom", "argx");

    let pure = successful_eval(session.eval(
        "SeedRandom[42]; {RandomInteger[10], RandomChoice[{a,b}], RandomSample[{a,b}], RandomString[8], BlockRandom[RandomInteger[10]], UuidQ[RandomUuid[]]}",
    ));
    assert_ne!(pure.value.text, "$Failed");
    assert!(pure.messages.is_empty(), "messages={:?}", pure.messages);
}

#[test]
fn permissive_policy_without_entropy_provider_returns_noentropy_failed() {
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().without_entropy_source())
        .build();

    for source in ["RandomBytes[8]", "RandomPassword[8]", "ReseedRandom[]"] {
        let success = successful_eval(session.eval(source));

        assert_eq!(success.value.text, "$Failed", "source {source}");
        assert_eq!(success.messages.len(), 1, "source {source}");
        assert_eq!(success.messages[0].symbol, "General");
        assert_eq!(success.messages[0].tag, "noentropy");
        assert_eq!(success.messages[0].text, "no entropy provider installed");
    }
}

#[test]
fn random_policy_allows_entropy_reseed() {
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Random]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    let success = successful_eval(session.eval("ReseedRandom[]"));
    assert_eq!(success.value.text, "Null");
    assert!(
        success.messages.is_empty(),
        "messages={:?}",
        success.messages
    );
}

#[test]
fn default_host_config_installs_entropy_provider_for_random_bytes() {
    let mut session = HostSession::new();

    let first = successful_eval(session.eval("RandomBytes[8]"));
    let second = successful_eval(session.eval("RandomBytes[8]"));

    let first_payload =
        byte_array_payload(&first.value.text).expect("RandomBytes should return ByteArray text");
    let second_payload =
        byte_array_payload(&second.value.text).expect("RandomBytes should return ByteArray text");
    assert_eq!(first_payload.len(), 12);
    assert_eq!(second_payload.len(), 12);
    assert_ne!(first.value.text, second.value.text);
}

#[test]
fn default_host_config_installs_entropy_provider_for_random_password() {
    let mut session = HostSession::new();
    let default_alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let custom_alphabet = "abcdefghjkmnpqrstuvwxyz23456789";

    let first = successful_eval(session.eval("RandomPassword[16]"));
    let second = successful_eval(session.eval("RandomPassword[16]"));
    let custom_source = format!("RandomPassword[{custom_alphabet:?}, 20]");
    let custom = successful_eval(session.eval(custom_source.as_str()));

    let first_text = string_payload(&first.value.text).expect("RandomPassword should return text");
    let second_text =
        string_payload(&second.value.text).expect("RandomPassword should return text");
    let custom_text =
        string_payload(&custom.value.text).expect("RandomPassword should return text");

    assert_eq!(first_text.len(), 16);
    assert_eq!(second_text.len(), 16);
    assert!(first_text.chars().all(|ch| default_alphabet.contains(ch)));
    assert!(second_text.chars().all(|ch| default_alphabet.contains(ch)));
    assert_ne!(first.value.text, second.value.text);

    assert_eq!(custom_text.len(), 20);
    assert!(custom_text.chars().all(|ch| custom_alphabet.contains(ch)));
    assert!(first.messages.is_empty(), "messages={:?}", first.messages);
    assert!(second.messages.is_empty(), "messages={:?}", second.messages);
    assert!(custom.messages.is_empty(), "messages={:?}", custom.messages);
}

#[test]
fn default_host_config_keeps_filesystem_catalog_unrestricted() {
    let dir = TempDirGuard::new("default-unrestricted");
    let input = dir.write("input.wl", "loaded = 42\nloaded");
    let save_path = dir.path().join("save.wl");
    let mut session = HostSession::new();

    assert_eq!(
        successful_eval(session.eval(format!("Get[{}]", wl_string(&rum_path(&input)))))
            .value
            .text,
        "42"
    );
    assert_eq!(
        successful_eval(session.eval(format!(
            "Save[{}, loaded]",
            wl_string(&rum_path(&save_path))
        )))
        .value
        .text,
        "Null"
    );
    assert!(save_path.exists());
    assert!(!fs::read_to_string(save_path).unwrap().is_empty());
}

#[test]
fn default_host_config_keeps_diagnostic_catalog_unrestricted() {
    let mut session = HostSession::new();

    assert_eq!(
        successful_eval(session.eval("RumProfileSnapshot[]"))
            .value
            .text,
        "Null"
    );
    assert_ne!(
        successful_eval(session.eval("MemoryInUse[]")).value.text,
        "$Failed"
    );
}

#[test]
fn capability_policy_can_explicitly_allow_diagnostics() {
    let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Diagnostic]);
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_capability_policy(policy))
        .build();

    assert_eq!(
        successful_eval(session.eval("RumProfileSnapshot[]"))
            .value
            .text,
        "Null"
    );
    assert_ne!(
        successful_eval(session.eval("MaxMemoryUsed[]")).value.text,
        "$Failed"
    );
    assert_sandbox_denied(&mut session, "Get[\"/etc/passwd\"]");
}

#[test]
fn host_eval_exposes_typed_value_variants() {
    let mut session = HostSession::new();
    let cases = vec![
        ("2 + 3", "5", HostTyped::Integer(5)),
        (
            "2^200",
            TWO_TO_200,
            HostTyped::BigInteger(TWO_TO_200.to_string()),
        ),
        (
            "1/3",
            "1/3",
            HostTyped::Rational {
                numerator: "1".to_string(),
                denominator: "3".to_string(),
            },
        ),
        ("True", "True", HostTyped::Symbol("True".to_string())),
        ("\"hi\"", "\"hi\"", HostTyped::Str("hi".to_string())),
        (
            "{1, 2, 3}",
            "List[1, 2, 3]",
            HostTyped::List(vec![
                HostTyped::Integer(1),
                HostTyped::Integer(2),
                HostTyped::Integer(3),
            ]),
        ),
        (
            "f[1]",
            "f[1]",
            HostTyped::Other {
                head: "f".to_string(),
                text: "f[1]".to_string(),
            },
        ),
    ];

    for (input, expected_text, expected_typed) in cases {
        match session.eval(input) {
            HostEvalOutcome::Success(success) => {
                assert_eq!(success.value.text, expected_text, "input {input}");
                assert_eq!(success.value.typed, expected_typed, "input {input}");
                eprintln!(
                    "typed_value input={input:?} text={:?} typed={:?}",
                    success.value.text, success.value.typed
                );
            }
            other => panic!("expected success for {input}, got {other:?}"),
        }
    }
}

#[test]
fn host_suspend_resume_flow_wraps_input_requests() {
    let mut session = HostSession::new();

    let suspended = session.eval("StringJoin[InputString[\"name: \"], \"!\"]");
    match suspended {
        HostEvalOutcome::InputRequired(suspension) => {
            assert_eq!(suspension.request.kind, HostInputKind::InputString);
            assert_eq!(suspension.request.prompt, "name: ");
            assert!(!suspension.request.password);
        }
        other => panic!("expected input suspension, got {other:?}"),
    }

    let resumed = session.resume_with_input("rum");
    match resumed {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "\"rum!\""),
        other => panic!("expected resumed success, got {other:?}"),
    }
}

#[test]
fn host_parse_inspect_complete_and_is_complete_cover_editor_flows() {
    let mut session = HostSession::new();

    let parsed = session.parse("1 + 2").expect("parse should succeed");
    assert_eq!(parsed.syntax, "1 + 2");

    assert!(matches!(
        session.eval("localName = 1"),
        HostEvalOutcome::Success(_)
    ));

    let inspect = session.inspect(HostInspectRequest::new("localName", 3));
    assert!(inspect.found);
    assert_eq!(
        inspect
            .info
            .as_ref()
            .map(|info| info.qualified_name.as_str()),
        Some("Global`localName")
    );

    let completion = session.complete(HostCompletionRequest::new("localName", 3));
    assert_eq!(completion.replacement_start, 0);
    assert_eq!(completion.replacement_end, "localName".len());
    assert_eq!(completion.prefix, "loc");
    assert_eq!(completion.matches, vec!["localName"]);

    let incomplete = session.is_complete("f[\n  1");
    assert_eq!(incomplete.status, HostCompletenessStatus::Incomplete);
    assert_eq!(incomplete.indent.as_deref(), Some("    "));

    let open_multiline_string = session.is_complete("x = \"\"\"\nabc");
    assert_eq!(
        open_multiline_string.status,
        HostCompletenessStatus::Incomplete
    );

    let open_raw_multiline_string = session.is_complete("x = '''\nabc");
    assert_eq!(
        open_raw_multiline_string.status,
        HostCompletenessStatus::Incomplete
    );
}

#[test]
fn host_evaluates_multiline_string_literals_as_ordinary_strings() {
    let mut session = HostSession::new();

    let escaped = successful_eval(session.eval("\"\"\"\nab\"\"\" === \"ab\""));
    assert_eq!(escaped.value.text, "True");

    let raw =
        successful_eval(session.eval("'''\nC:\\path \\t raw\n''' === \"C:\\\\path \\\\t raw\\n\""));
    assert_eq!(raw.value.text, "True");
}

#[test]
fn host_processes_inline_multiline_csv_data() {
    let mut session = HostSession::new();
    let source = "data = \"\"\"\na,1\nb,2\"\"\"; {StringLines[data], FromCsv[data]}";

    let success = successful_eval(session.eval(source));

    assert_eq!(
        success.value.text,
        "List[List[\"a,1\", \"b,2\"], List[List[\"a\", \"1\"], List[\"b\", \"2\"]]]"
    );
}

#[test]
fn host_session_introspection_enumerates_user_definitions_and_contexts() {
    let mut session = HostSession::new();

    let fresh_definitions = session.definitions();
    assert!(
        fresh_definitions
            .iter()
            .all(|summary| summary.name != "System`Plus"),
        "native builtin Plus should not appear in definition summaries"
    );

    let fresh_contexts = session.contexts();
    assert_eq!(
        fresh_contexts,
        vec!["Global`".to_string(), "System`".to_string()]
    );

    successful_eval(session.eval("a = 1; f[x_] := x; f[2] := 9"));

    let definitions = session.definitions();
    assert_eq!(
        definitions,
        vec![
            HostDefinitionSummary {
                name: "Global`a".to_string(),
                has_own_value: true,
                down_value_count: 0,
                up_value_count: 0,
                sub_value_count: 0,
            },
            HostDefinitionSummary {
                name: "Global`f".to_string(),
                has_own_value: false,
                down_value_count: 2,
                up_value_count: 0,
                sub_value_count: 0,
            },
        ]
    );

    let contexts = session.contexts();
    assert_eq!(contexts, vec!["Global`".to_string(), "System`".to_string()]);
    eprintln!("introspection definitions={definitions:?} contexts={contexts:?}");
}

#[test]
fn host_session_introspection_survives_poisoned_session() {
    let mut session = HostSession::new();

    successful_eval(session.eval("p = 7"));
    match session.eval_with_test_panic() {
        HostEvalOutcome::Failure(failure) => assert_eq!(failure.code, HostFailureCode::Panic),
        other => panic!("expected panic failure, got {other:?}"),
    }
    assert!(session.poisoned_for_test());

    let definitions = session.definitions();
    assert!(
        definitions
            .iter()
            .any(|summary| summary.name == "Global`p" && summary.has_own_value)
    );

    let contexts = session.contexts();
    assert!(contexts.iter().any(|context| context == "Global`"));
    assert!(contexts.iter().any(|context| context == "System`"));
    eprintln!("poisoned_introspection definitions={definitions:?} contexts={contexts:?}");
}

#[test]
fn host_failures_use_stable_failure_codes() {
    let mut session = HostSession::new();

    // `1 2` is now valid juxtaposition (1*2) since 67a3bdc; use a still-invalid input.
    let outcome = session.eval("f[1,]");

    match outcome {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Syntax);
            assert!(!failure.message.is_empty());
        }
        other => panic!("expected syntax failure, got {other:?}"),
    }
}

#[test]
fn host_boundary_catches_panic_and_poisons_session() {
    let mut session = HostSession::new();

    let caught = session.eval_with_test_panic();

    let (caught_code, caught_message) = match caught {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Panic);
            assert!(failure.message.contains("host panic boundary test"));
            (failure.code, failure.message)
        }
        other => panic!("expected panic failure, got {other:?}"),
    };
    assert!(session.poisoned_for_test());

    let followup = session.eval("1 + 1");
    match followup {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Poisoned);
            assert!(failure.message.contains("poisoned"));
            eprintln!(
                "host_boundary_catches_panic_and_poisons_session caught_code={caught_code:?} caught_message={caught_message:?} followup_code={:?} followup_message={:?}",
                failure.code, failure.message
            );
        }
        other => panic!("expected poisoned failure, got {other:?}"),
    }
}

#[test]
fn host_boundary_normal_eval_does_not_poison_session() {
    let mut session = HostSession::new();

    match session.eval("1 + 2") {
        HostEvalOutcome::Success(success) => assert_eq!(success.value.text, "3"),
        other => panic!("expected normal success, got {other:?}"),
    }
    assert!(!session.poisoned_for_test());

    match session.eval("3 + 4") {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "7");
            eprintln!(
                "host_boundary_normal_eval_does_not_poison_session first=3 second=7 poisoned={}",
                session.poisoned_for_test()
            );
        }
        other => panic!("expected follow-up success, got {other:?}"),
    }
}

#[test]
fn host_session_reset_clears_user_state() {
    let mut session = HostSession::new();

    let assigned = successful_eval(session.eval("x = 41; x"));
    assert_eq!(assigned.value.text, "41");

    session.reset();

    let reset_symbol = successful_eval(session.eval("x"));
    assert_eq!(reset_symbol.value.text, "x");
    assert_eq!(reset_symbol.value.typed, HostTyped::Symbol("x".to_string()));

    let arithmetic = successful_eval(session.eval("2 + 2"));
    assert_eq!(arithmetic.value.text, "4");

    eprintln!(
        "host_session_reset_clears_user_state assigned=41 after_reset={:?} arithmetic=4 poisoned={}",
        reset_symbol.value.text,
        session.poisoned_for_test()
    );
}

#[test]
fn host_session_reset_recovers_poisoned_session() {
    let mut session = HostSession::new();

    let caught = session.eval_with_test_panic();
    match caught {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Panic);
            assert!(failure.message.contains("host panic boundary test"));
        }
        other => panic!("expected panic failure, got {other:?}"),
    }
    assert!(session.poisoned_for_test());

    let poisoned = session.eval("1 + 1");
    match poisoned {
        HostEvalOutcome::Failure(failure) => {
            assert_eq!(failure.code, HostFailureCode::Poisoned);
            assert!(failure.message.contains("poisoned"));
        }
        other => panic!("expected poisoned failure, got {other:?}"),
    }

    session.reset();
    assert!(!session.poisoned_for_test());

    let recovered = successful_eval(session.eval("1 + 1"));
    assert_eq!(recovered.value.text, "2");

    eprintln!(
        "host_session_reset_recovers_poisoned_session poisoned_before_reset=true recovered_value=2 poisoned_after_reset={}",
        session.poisoned_for_test()
    );
}

#[test]
fn host_session_reset_restores_original_config_not_mutated_state() {
    let mut limits = HostResourceLimits::default();
    limits.recursion_limit = 77;
    let original_config = HostConfig::default()
        .with_resource_limits(limits)
        .with_current_context("Original")
        .with_context_path(["System", "Original"]);
    let mut session = HostSession::builder().with_config(original_config).build();

    assert_eq!(session.configuration().current_context, "Original`");
    assert_eq!(
        session.configuration().context_path,
        vec!["System`".to_string(), "Original`".to_string()]
    );

    let changed = successful_eval(
        session.eval("Begin[\"Changed`\"]; $ContextPath = {\"Changed`\"}; $Context"),
    );
    assert_eq!(changed.value.text, "\"Changed`\"");
    assert_eq!(session.configuration().current_context, "Changed`");
    assert_eq!(
        session.configuration().context_path,
        vec!["Changed`".to_string()]
    );

    session.reset();

    let reset_config = session.configuration();
    assert_eq!(reset_config.current_context, "Original`");
    assert_eq!(
        reset_config.context_path,
        vec!["System`".to_string(), "Original`".to_string()]
    );
    assert_eq!(reset_config.resource_limits.recursion_limit, 77);

    let reset_context = successful_eval(session.eval("$Context"));
    assert_eq!(reset_context.value.text, "\"Original`\"");

    eprintln!(
        "host_session_reset_restores_original_config_not_mutated_state current_context={:?} context_path={:?} recursion_limit={}",
        reset_config.current_context,
        reset_config.context_path,
        reset_config.resource_limits.recursion_limit
    );
}

#[test]
fn host_session_snapshot_bytes_roundtrip_and_restore_replaces_user_state() {
    let mut a = HostSession::new();
    successful_eval(
        a.eval("a = 1; f[x_] := x; g[2] := 20; h[x_] := x; SetAttributes[h, Orderless]"),
    );
    let bytes = a.snapshot_to_bytes();

    let mut b = HostSession::new();
    successful_eval(b.eval("z = 99"));
    b.restore_from_bytes(&bytes)
        .expect("snapshot bytes should restore");

    assert_eq!(b.definitions(), a.definitions());
    assert_eq!(successful_eval(b.eval("f[7]")).value.text, "7");
    assert_eq!(
        successful_eval(b.eval("Attributes[h]")).value.text,
        "List[Orderless]"
    );
    assert_eq!(successful_eval(b.eval("z")).value.text, "z");

    eprintln!(
        "host_session_snapshot_bytes_roundtrip_and_restore_replaces_user_state definitions={:?} bytes={} f7=7 attributes_h=List[Orderless] z_after_restore=z",
        b.definitions(),
        bytes.len()
    );
}

#[test]
fn host_eval_ordinary_value_has_no_termination_reason() {
    let mut session = HostSession::new();

    let success = successful_eval(session.eval("1 + 1"));

    assert_eq!(success.value.text, "2");
    assert_eq!(success.termination, None);
}

#[test]
fn host_eval_literal_terminated_evaluation_has_no_termination_reason() {
    let mut session = HostSession::new();

    let success = successful_eval(session.eval("TerminatedEvaluation[\"Cancelled\"]"));

    assert_eq!(success.value.text, "TerminatedEvaluation[\"Cancelled\"]");
    assert_eq!(success.termination, None);
}

#[test]
fn host_eval_recursion_limit_reports_structured_termination_reason() {
    let mut limits = HostResourceLimits::default();
    limits.recursion_limit = 32;
    let mut session = HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build();

    let success = successful_eval(session.eval("f[x_] := f[x + 1]; f[0]"));

    assert_eq!(
        success.value.text,
        "TerminatedEvaluation[\"RecursionLimit\"]"
    );
    assert_eq!(
        success.termination,
        Some(HostTerminationReason::RecursionLimit)
    );
}

#[test]
fn host_cancel_handle_interrupts_running_eval_from_another_thread() {
    let mut session = host_session_with_large_iteration_budget();
    let cancel = session.cancel_handle();
    let (cancelled_tx, cancelled_rx) = mpsc::channel();

    let canceller = {
        let cancel = cancel.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            let cancelled_at = Instant::now();
            cancel.cancel();
            cancelled_tx
                .send(cancelled_at)
                .expect("test should receive cancel timestamp");
        })
    };

    let started = Instant::now();
    let outcome = session.eval(LONG_RUNNING_EVAL);
    let completed_at = Instant::now();
    let elapsed = completed_at.duration_since(started);
    let cancelled_at = cancelled_rx
        .recv()
        .expect("canceller should report cancel timestamp");
    canceller.join().expect("canceller should not panic");
    let cancel_latency = completed_at.duration_since(cancelled_at);

    eprintln!(
        "host_cancel_handle_interrupts_running_eval_from_another_thread elapsed_ms={} cancel_latency_ms={}",
        elapsed.as_millis(),
        cancel_latency.as_millis()
    );

    assert!(
        elapsed < Duration::from_secs(2),
        "cancelled eval took {elapsed:?}"
    );
    assert!(
        cancel_latency < Duration::from_secs(1),
        "cancel observation took {cancel_latency:?}"
    );

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "TerminatedEvaluation[\"Cancelled\"]");
            assert_eq!(success.termination, Some(HostTerminationReason::Cancelled));
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "cancel")
            );
        }
        other => panic!("expected graceful cancelled value, got {other:?}"),
    }
}

#[test]
fn host_eval_with_timeout_interrupts_running_eval() {
    let mut session = host_session_with_large_iteration_budget();

    let started = Instant::now();
    let outcome = session.eval_with_timeout(LONG_RUNNING_EVAL, Duration::from_millis(200));
    let elapsed = started.elapsed();

    eprintln!(
        "host_eval_with_timeout_interrupts_running_eval elapsed_ms={}",
        elapsed.as_millis()
    );

    assert!(
        elapsed < Duration::from_secs(2),
        "timed eval took {elapsed:?}"
    );

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(
                success.value.text,
                "TerminatedEvaluation[\"TimeConstraint\"]"
            );
            assert_eq!(
                success.termination,
                Some(HostTerminationReason::TimeConstraint)
            );
            assert!(
                success
                    .messages
                    .iter()
                    .any(|message| message.symbol == "General" && message.tag == "timelim")
            );
        }
        other => panic!("expected graceful timeout value, got {other:?}"),
    }
}

#[test]
fn host_eval_with_unused_handle_and_generous_timeout_still_succeeds() {
    let mut session = HostSession::new();
    let _cancel = session.cancel_handle();

    let started = Instant::now();
    let outcome = session.eval_with_timeout("1 + 2", Duration::from_secs(5));
    let elapsed = started.elapsed();

    eprintln!(
        "host_eval_with_unused_handle_and_generous_timeout_still_succeeds elapsed_ms={}",
        elapsed.as_millis()
    );

    match outcome {
        HostEvalOutcome::Success(success) => {
            assert_eq!(success.value.text, "3");
            assert_eq!(success.termination, None);
        }
        other => panic!("expected normal success, got {other:?}"),
    }
}
