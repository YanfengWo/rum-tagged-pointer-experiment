#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
#[path = "oracle_lanes/failure_gates.rs"]
mod failure_gates;
#[path = "oracle_lanes/filters.rs"]
mod filters;
#[path = "oracle_lanes/help.rs"]
mod help;
#[path = "oracle_lanes/issue_samples.rs"]
mod issue_samples;
#[path = "oracle_lanes/summary_json.rs"]
mod summary_json;

#[cfg(unix)]
fn write_fixed_value_batch_oracle(path: &std::path::Path) {
    fs::write(
        path,
        r#"#!/bin/sh
wrapper=$3
manifest=$4
case "$wrapper" in
  *oracle_version*)
    printf '%s\n' '__RUM_DIFFTEST_VERSION__=fake Wolfram oracle'
    exit 0
    ;;
esac

batch_index=1
while IFS= read -r batch_manifest || [ -n "$batch_manifest" ]; do
  [ -z "$batch_manifest" ] && continue
  session_dir=$(dirname "$batch_manifest")
  step_index=1
  while IFS= read -r input_path || [ -n "$input_path" ]; do
    [ -z "$input_path" ] && continue
    suffix=$(printf '%04d-%04d' "$batch_index" "$step_index")
    : > "$session_dir/messages-$suffix.txt"
    : > "$session_dir/prints-$suffix.txt"
    printf '__RUM_DIFFTEST_SESSION_BATCH_RESULT__=%d\t%d\t2\n' "$batch_index" "$step_index"
    step_index=$((step_index + 1))
  done < "$batch_manifest"
  batch_index=$((batch_index + 1))
done < "$manifest"
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}
