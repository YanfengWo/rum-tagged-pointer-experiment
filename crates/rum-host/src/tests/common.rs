use super::super::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const LONG_RUNNING_EVAL: &str = "Do[x = 1, 50000000]";
pub(super) const TWO_TO_200: &str = "1606938044258990275541962092341162602522202993782792835301376";

pub(super) fn host_session_with_large_iteration_budget() -> HostSession {
    let mut limits = HostResourceLimits::default();
    limits.iteration_work_limit = 50_000_000;
    HostSession::builder()
        .with_config(HostConfig::default().with_resource_limits(limits))
        .build()
}

pub(super) struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    pub(super) fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("rum-host-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn write(&self, relative: &str, source: &str) -> PathBuf {
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

pub(super) fn successful_eval(outcome: HostEvalOutcome) -> HostEvalSuccess {
    match outcome {
        HostEvalOutcome::Success(success) => success,
        other => panic!("expected success, got {other:?}"),
    }
}

pub(super) fn failed_eval(outcome: HostEvalOutcome) -> HostFailure {
    match outcome {
        HostEvalOutcome::Failure(failure) => failure,
        other => panic!("expected failure, got {other:?}"),
    }
}

pub(super) fn rum_path(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
        .replace('\\', "/")
}

pub(super) fn wl_string(value: &str) -> String {
    format!("{value:?}")
}

pub(super) fn byte_array_payload(text: &str) -> Option<&str> {
    text.strip_prefix("ByteArray[\"")?.strip_suffix("\"]")
}

pub(super) fn string_payload(text: &str) -> Option<&str> {
    text.strip_prefix('"')?.strip_suffix('"')
}

pub(super) fn assert_sandbox_denied(session: &mut HostSession, source: &str) {
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

pub(super) fn assert_arity_not_sandbox_denial(
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

pub(super) const ALL_CAPABILITIES: [Capability; 8] = [
    Capability::Pure,
    Capability::FileRead,
    Capability::FileWrite,
    Capability::Clock,
    Capability::Random,
    Capability::Diagnostic,
    Capability::Process,
    Capability::Interactive,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExpectedProfileSurface {
    PureOnly,
    Unrestricted,
}

pub(super) fn profile_allows(surface: ExpectedProfileSurface, capability: Capability) -> bool {
    match capability {
        Capability::Pure => true,
        Capability::FileRead
        | Capability::FileWrite
        | Capability::Clock
        | Capability::Random
        | Capability::Diagnostic
        | Capability::Process
        | Capability::Interactive => surface == ExpectedProfileSurface::Unrestricted,
    }
}

pub(super) fn capability_probe_source(
    capability: Capability,
    input_path: &Path,
    save_path: &Path,
) -> String {
    match capability {
        Capability::Pure => "1 + 1".to_string(),
        Capability::FileRead => format!("Get[{}]", wl_string(&rum_path(input_path))),
        Capability::FileWrite => format!(
            "profileValue = 242; Save[{}, profileValue]",
            wl_string(&rum_path(save_path))
        ),
        Capability::Clock => "AbsoluteTime[]".to_string(),
        Capability::Random => "ReseedRandom[]".to_string(),
        Capability::Diagnostic => "RumProfileSnapshot[]".to_string(),
        Capability::Process => "Exit[7]".to_string(),
        Capability::Interactive => "InputString[]".to_string(),
    }
}

pub(super) fn assert_profile_capability_surface(
    config: HostConfig,
    surface: ExpectedProfileSurface,
) {
    let dir = TempDirGuard::new("profile-capabilities");
    let input_path = dir.write("input.rum", "73");
    let save_path = dir.path().join("saved.rum");

    for capability in ALL_CAPABILITIES {
        let mut session = HostSession::builder().with_config(config.clone()).build();
        let source = capability_probe_source(capability, &input_path, &save_path);

        if !profile_allows(surface, capability) {
            assert_sandbox_denied(&mut session, &source);
            continue;
        }

        let outcome = session.eval(source.as_str());
        match capability {
            Capability::Pure => {
                let success = successful_eval(outcome);
                assert_eq!(success.value.text, "2");
                assert!(success.messages.is_empty(), "{:?}", success.messages);
            }
            Capability::FileRead => {
                let success = successful_eval(outcome);
                assert_eq!(success.value.text, "73");
                assert!(success.messages.is_empty(), "{:?}", success.messages);
            }
            Capability::FileWrite => {
                let success = successful_eval(outcome);
                assert_eq!(success.value.text, "Null");
                assert!(success.messages.is_empty(), "{:?}", success.messages);
                assert!(save_path.exists());
            }
            Capability::Clock => {
                let success = successful_eval(outcome);
                assert_ne!(success.value.text, "$Failed");
                assert!(success.messages.is_empty(), "{:?}", success.messages);
            }
            Capability::Random | Capability::Diagnostic => {
                let success = successful_eval(outcome);
                assert_eq!(success.value.text, "Null");
                assert!(success.messages.is_empty(), "{:?}", success.messages);
            }
            Capability::Process => match outcome {
                HostEvalOutcome::Failure(failure) => {
                    assert_eq!(failure.code, HostFailureCode::Exit);
                    assert_eq!(failure.exit_status, Some(7));
                }
                other => panic!("expected process-exit outcome, got {other:?}"),
            },
            Capability::Interactive => match outcome {
                HostEvalOutcome::InputRequired(suspension) => {
                    assert_eq!(suspension.request.kind, HostInputKind::InputString);
                }
                other => panic!("expected interactive suspension, got {other:?}"),
            },
        }
    }
}

pub(super) fn assert_profile_providers(config: HostConfig, expected: bool) {
    assert_eq!(config.file_io.is_some(), expected);
    assert_eq!(config.entropy_source.is_some(), expected);

    let dir = TempDirGuard::new("profile-providers");
    let input_path = dir.write("input.rum", "91");
    let mut file_session = HostSession::builder().with_config(config.clone()).build();
    match file_session.eval_file(&input_path) {
        HostEvalOutcome::Success(success) if expected => assert_eq!(success.value.text, "91"),
        HostEvalOutcome::Failure(failure) if !expected => {
            assert_eq!(failure.code, HostFailureCode::Other);
            assert_eq!(failure.message, "no file provider installed");
        }
        other => panic!("unexpected file-provider probe outcome: {other:?}"),
    }

    let mut entropy_session = HostSession::builder()
        .with_config(config.with_capability_policy(CapabilityPolicy::unrestricted()))
        .build();
    let entropy = successful_eval(entropy_session.eval("RandomBytes[8]"));
    if expected {
        assert!(byte_array_payload(&entropy.value.text).is_some());
        assert!(entropy.messages.is_empty(), "{:?}", entropy.messages);
    } else {
        assert_eq!(entropy.value.text, "$Failed");
        assert_eq!(entropy.messages.len(), 1);
        assert_eq!(entropy.messages[0].symbol, "General");
        assert_eq!(entropy.messages[0].tag, "noentropy");
        assert_eq!(entropy.messages[0].text, "no entropy provider installed");
    }
}
