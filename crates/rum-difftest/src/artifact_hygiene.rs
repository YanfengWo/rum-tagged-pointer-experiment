use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::fixtures::crate_root;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleArtifactIssue {
    pub path: PathBuf,
    pub reason: String,
}

pub fn validate_oracle_artifact_hygiene() -> Result<Vec<OracleArtifactIssue>, std::io::Error> {
    validate_oracle_artifact_hygiene_in(&crate_root())
}

pub fn validate_oracle_artifact_hygiene_in(
    root: &Path,
) -> Result<Vec<OracleArtifactIssue>, std::io::Error> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;

    let mut issues = files
        .into_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            oracle_artifact_issue(&relative)
        })
        .collect::<Vec<_>>();
    issues.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(issues)
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn oracle_artifact_issue(path: &Path) -> Option<OracleArtifactIssue> {
    if path
        .components()
        .any(|component| matches_generated_session_dir(component))
    {
        return Some(issue(path, "generated Lane B oracle session directory"));
    }

    let file_name = path.file_name()?.to_str()?;
    if matches!(
        file_name,
        "oracle_wrapper.wls"
            | "oracle_session_wrapper.wls"
            | "oracle_session_batch_wrapper.wls"
            | "oracle_version.wls"
    ) {
        return Some(issue(path, "generated oracle wrapper script"));
    }
    if matches_numbered_name(file_name, "case-", ".wl")
        || matches_numbered_name(file_name, "step-", ".wl")
    {
        return Some(issue(path, "generated oracle input file"));
    }
    if matches_numbered_name(file_name, "messages-", ".txt")
        || matches_numbered_name(file_name, "prints-", ".txt")
    {
        return Some(issue(path, "generated oracle stream capture"));
    }

    None
}

fn matches_generated_session_dir(component: Component<'_>) -> bool {
    let Component::Normal(name) = component else {
        return false;
    };
    let Some(name) = name.to_str() else {
        return false;
    };
    matches_numbered_name(name, "session-", "")
        || matches_numbered_name(name, "session-batches-", "")
}

fn matches_numbered_name(name: &str, prefix: &str, suffix: &str) -> bool {
    let Some(number) = name
        .strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(suffix))
    else {
        return false;
    };
    number.len() == 4 && number.bytes().all(|byte| byte.is_ascii_digit())
}

fn issue(path: &Path, reason: &str) -> OracleArtifactIssue {
    OracleArtifactIssue {
        path: path.to_path_buf(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn current_tree_has_no_oracle_artifacts() {
        let issues = validate_oracle_artifact_hygiene().unwrap();
        assert!(
            issues.is_empty(),
            "oracle artifacts should stay transient and out of the crate tree: {issues:?}"
        );
    }

    #[test]
    fn detects_oracle_wrapper_artifacts() {
        assert_eq!(
            oracle_artifact_issue(Path::new("oracle_wrapper.wls")),
            Some(issue(
                Path::new("oracle_wrapper.wls"),
                "generated oracle wrapper script"
            ))
        );
        assert_eq!(
            oracle_artifact_issue(Path::new("oracle_version.wls")),
            Some(issue(
                Path::new("oracle_version.wls"),
                "generated oracle wrapper script"
            ))
        );
        assert_eq!(
            oracle_artifact_issue(Path::new("oracle_session_batch_wrapper.wls")),
            Some(issue(
                Path::new("oracle_session_batch_wrapper.wls"),
                "generated oracle wrapper script"
            ))
        );
    }

    #[test]
    fn detects_oracle_case_and_stream_artifacts() {
        assert_eq!(
            oracle_artifact_issue(Path::new("case-0001.wl")),
            Some(issue(
                Path::new("case-0001.wl"),
                "generated oracle input file"
            ))
        );
        assert_eq!(
            oracle_artifact_issue(Path::new("session-0002/messages-0001.txt")),
            Some(issue(
                Path::new("session-0002/messages-0001.txt"),
                "generated Lane B oracle session directory"
            ))
        );
        assert_eq!(
            oracle_artifact_issue(Path::new("session-batches-0002/batch-0001/manifest.txt")),
            Some(issue(
                Path::new("session-batches-0002/batch-0001/manifest.txt"),
                "generated Lane B oracle session directory"
            ))
        );
        assert!(oracle_artifact_issue(Path::new("cases/stateful/package_defaults.txt")).is_none());
    }

    #[test]
    fn validates_artifacts_under_an_isolated_root() {
        let root = unique_temp_root("rum-difftest-artifact-hygiene");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::create_dir_all(root.join("session-batches-0002/batch-0001")).unwrap();
        fs::create_dir_all(root.join("cases/stateful")).unwrap();
        fs::write(root.join("oracle_wrapper.wls"), "").unwrap();
        fs::write(root.join("nested/case-0001.wl"), "").unwrap();
        fs::write(
            root.join("session-batches-0002/batch-0001/manifest.txt"),
            "",
        )
        .unwrap();
        fs::write(root.join("cases/stateful/package_defaults.txt"), "").unwrap();

        let issues = validate_oracle_artifact_hygiene_in(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();

        assert_eq!(
            issues,
            vec![
                issue(
                    Path::new("nested/case-0001.wl"),
                    "generated oracle input file"
                ),
                issue(
                    Path::new("oracle_wrapper.wls"),
                    "generated oracle wrapper script"
                ),
                issue(
                    Path::new("session-batches-0002/batch-0001/manifest.txt"),
                    "generated Lane B oracle session directory"
                ),
            ]
        );
    }

    fn unique_temp_root(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
    }
}
