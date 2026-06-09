use super::{json_array, json_str};

#[path = "check/artifact_hygiene.rs"]
mod artifact_hygiene;
#[path = "check/basic.rs"]
mod basic;
#[path = "check/inventory.rs"]
mod inventory;
#[path = "check/issue_samples.rs"]
mod issue_samples;
#[path = "check/lane_c.rs"]
mod lane_c;
#[path = "check/lane_d.rs"]
mod lane_d;
#[path = "check/summary_json.rs"]
mod summary_json;

fn check_entry<'a>(summary: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    json_array(summary, "/checks")
        .iter()
        .find(|check| json_str(check, "/name") == name)
        .unwrap_or_else(|| panic!("missing check summary `{name}`: {summary}"))
}

fn skipped_check_entry<'a>(summary: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    json_array(summary, "/skipped_checks")
        .iter()
        .find(|check| json_str(check, "/name") == name)
        .unwrap_or_else(|| panic!("missing skipped check summary `{name}`: {summary}"))
}

fn issue_check_count_entry<'a>(
    summary: &'a serde_json::Value,
    check_name: &str,
) -> &'a serde_json::Value {
    json_array(summary, "/issue_check_counts")
        .iter()
        .find(|entry| json_str(entry, "/check") == check_name)
        .unwrap_or_else(|| panic!("missing issue check count `{check_name}`: {summary}"))
}

fn issue_kind_count_entry<'a>(
    summary: &'a serde_json::Value,
    check_name: &str,
    kind: &str,
) -> &'a serde_json::Value {
    json_array(summary, "/issue_kind_counts")
        .iter()
        .find(|entry| json_str(entry, "/check") == check_name && json_str(entry, "/kind") == kind)
        .unwrap_or_else(|| panic!("missing issue kind count `{check_name}`/`{kind}`: {summary}"))
}
