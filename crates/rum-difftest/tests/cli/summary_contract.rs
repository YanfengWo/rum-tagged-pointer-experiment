use std::fs;

use super::{json_bool, json_str, parse_summary_json, run_difftest, stderr_text, unique_temp_dir};

#[test]
fn commands_emit_schema_versioned_summary_json() {
    let temp_dir = unique_temp_dir("summary-contract");
    fs::create_dir_all(&temp_dir).unwrap();
    let cases: &[(&str, &[&str], &str, &str, bool)] = &[
        (
            "check",
            &["check", "--summary-json"],
            "/command",
            "check",
            true,
        ),
        (
            "lane-a",
            &[
                "lane-a",
                "--oracle-command",
                "/bin/true",
                "--limit",
                "1",
                "--summary-json",
            ],
            "/lane",
            "lane-a",
            false,
        ),
        (
            "lane-b",
            &[
                "lane-b",
                "--oracle-command",
                "/bin/true",
                "--limit",
                "1",
                "--summary-json",
            ],
            "/lane",
            "lane-b",
            false,
        ),
        (
            "inventory",
            &["inventory", "--summary-json"],
            "/lane",
            "inventory",
            true,
        ),
        (
            "lane-c",
            &["lane-c", "--summary-json"],
            "/lane",
            "lane-c",
            true,
        ),
        (
            "lane-d",
            &[
                "lane-d",
                "--suite",
                "timing",
                "--limit",
                "1",
                "--summary-json",
            ],
            "/lane",
            "lane-d",
            true,
        ),
    ];

    for (name, args_prefix, identity_pointer, identity_value, expected_clean) in cases {
        let summary_path = temp_dir.join(format!("{name}.json"));
        let mut args = args_prefix.to_vec();
        args.push(summary_path.to_str().unwrap());
        let output = run_difftest(&args);

        assert!(
            output.status.success(),
            "{name} summary command failed: {}",
            stderr_text(&output)
        );
        let summary = fs::read_to_string(&summary_path).unwrap();
        let summary_json = parse_summary_json(&summary);
        assert_eq!(
            json_str(&summary_json, identity_pointer),
            *identity_value,
            "{name} summary did not expose the expected command/lane identity"
        );
        assert_eq!(
            json_bool(&summary_json, "/clean"),
            *expected_clean,
            "{name} summary did not expose the expected top-level clean status"
        );
        assert!(
            summary_path.is_file(),
            "{name} summary file should be created"
        );
    }

    fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn failed_aggregate_check_still_emits_schema_versioned_summary_json() {
    let temp_dir = unique_temp_dir("failed-check-summary-contract");
    let baseline_path = temp_dir.join("inventory-baseline.txt");
    let summary_path = temp_dir.join("summary.json");
    fs::create_dir_all(&temp_dir).unwrap();
    fs::write(
        &baseline_path,
        "lane=lane-a source=curated-core min_fixtures=999 min_entries=999 min_normal_entries=999 max_known_difference_entries=0\n",
    )
    .unwrap();

    let output = run_difftest(&[
        "check",
        "--inventory-baseline",
        baseline_path.to_str().unwrap(),
        "--summary-json",
        summary_path.to_str().unwrap(),
    ]);

    assert!(!output.status.success());
    let summary = fs::read_to_string(&summary_path).unwrap();
    let summary_json = parse_summary_json(&summary);
    assert_eq!(json_str(&summary_json, "/command"), "check");
    assert!(!json_bool(&summary_json, "/clean"));

    fs::remove_dir_all(temp_dir).unwrap();
}
