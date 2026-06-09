use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "cli/check.rs"]
mod check;
#[path = "cli/help.rs"]
mod help;
#[path = "cli/inventory.rs"]
mod inventory;
#[path = "cli/lane_c.rs"]
mod lane_c;
#[path = "cli/lane_d.rs"]
mod lane_d;
#[path = "cli/oracle_lanes.rs"]
mod oracle_lanes;
#[path = "cli/summary_contract.rs"]
mod summary_contract;

fn run_difftest(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rum-difftest"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn parse_summary_json(summary: &str) -> serde_json::Value {
    let value: serde_json::Value = serde_json::from_str(summary)
        .unwrap_or_else(|err| panic!("invalid summary JSON: {err}\n{summary}"));
    assert_eq!(
        value.get("schema_version").and_then(|value| value.as_u64()),
        Some(1)
    );
    value
}

fn json_str<'a>(value: &'a serde_json::Value, pointer: &str) -> &'a str {
    value
        .pointer(pointer)
        .and_then(|value| value.as_str())
        .unwrap_or_else(|| panic!("expected JSON string at {pointer}: {value}"))
}

fn json_u64(value: &serde_json::Value, pointer: &str) -> u64 {
    value
        .pointer(pointer)
        .and_then(|value| value.as_u64())
        .unwrap_or_else(|| panic!("expected JSON integer at {pointer}: {value}"))
}

fn json_bool(value: &serde_json::Value, pointer: &str) -> bool {
    value
        .pointer(pointer)
        .and_then(|value| value.as_bool())
        .unwrap_or_else(|| panic!("expected JSON bool at {pointer}: {value}"))
}

fn json_array<'a>(value: &'a serde_json::Value, pointer: &str) -> &'a [serde_json::Value] {
    value
        .pointer(pointer)
        .and_then(|value| value.as_array())
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("expected JSON array at {pointer}: {value}"))
}

fn assert_json_null(value: &serde_json::Value, pointer: &str) {
    assert!(
        value
            .pointer(pointer)
            .is_some_and(serde_json::Value::is_null),
        "expected JSON null at {pointer}: {value}"
    );
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
