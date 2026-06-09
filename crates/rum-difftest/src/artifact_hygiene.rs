use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::fixtures::crate_root;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactHygieneIssue {
    pub path: PathBuf,
    pub kind: ArtifactHygieneIssueKind,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactHygieneIssueKind {
    OracleArtifact,
    StrayFixtureFile,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArtifactHygieneIssueCounts {
    pub oracle_artifacts: usize,
    pub stray_fixture_files: usize,
}

impl ArtifactHygieneIssueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ArtifactHygieneIssueKind::OracleArtifact => "oracle_artifact",
            ArtifactHygieneIssueKind::StrayFixtureFile => "stray_fixture_file",
        }
    }
}

pub fn count_artifact_hygiene_issues(
    issues: &[ArtifactHygieneIssue],
) -> ArtifactHygieneIssueCounts {
    let mut counts = ArtifactHygieneIssueCounts::default();
    for issue in issues {
        match issue.kind {
            ArtifactHygieneIssueKind::OracleArtifact => counts.oracle_artifacts += 1,
            ArtifactHygieneIssueKind::StrayFixtureFile => counts.stray_fixture_files += 1,
        }
    }
    counts
}

pub fn validate_artifact_hygiene() -> Result<Vec<ArtifactHygieneIssue>, std::io::Error> {
    validate_artifact_hygiene_in(&crate_root())
}

pub fn validate_artifact_hygiene_in(
    root: &Path,
) -> Result<Vec<ArtifactHygieneIssue>, std::io::Error> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;

    let mut issues = files
        .into_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            artifact_hygiene_issue(&relative)
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

fn artifact_hygiene_issue(path: &Path) -> Option<ArtifactHygieneIssue> {
    oracle_artifact_issue(path).or_else(|| fixture_tree_issue(path))
}

fn oracle_artifact_issue(path: &Path) -> Option<ArtifactHygieneIssue> {
    if path
        .components()
        .any(|component| matches_generated_session_dir(component))
    {
        return Some(issue(
            path,
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated Lane B oracle session directory",
        ));
    }

    let file_name = path.file_name()?.to_str()?;
    if matches!(
        file_name,
        "oracle_wrapper.wls"
            | "oracle_session_wrapper.wls"
            | "oracle_session_batch_wrapper.wls"
            | "oracle_version.wls"
    ) {
        return Some(issue(
            path,
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated oracle wrapper script",
        ));
    }
    if matches_numbered_name(file_name, "case-", ".wl")
        || matches_numbered_name(file_name, "step-", ".wl")
    {
        return Some(issue(
            path,
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated oracle input file",
        ));
    }
    if matches_numbered_name(file_name, "messages-", ".txt")
        || matches_numbered_name(file_name, "prints-", ".txt")
    {
        return Some(issue(
            path,
            ArtifactHygieneIssueKind::OracleArtifact,
            "generated oracle stream capture",
        ));
    }

    None
}

fn fixture_tree_issue(path: &Path) -> Option<ArtifactHygieneIssue> {
    let components = path_components(path)?;
    if components.first().copied() != Some("cases") {
        return None;
    }

    if allowed_case_tree_file(&components) {
        return None;
    }

    Some(issue(
        path,
        ArtifactHygieneIssueKind::StrayFixtureFile,
        "unexpected fixture tree file; keep probes outside the crate or use a supported fixture/support extension",
    ))
}

fn path_components(path: &Path) -> Option<Vec<&str>> {
    path.components()
        .map(|component| {
            let Component::Normal(name) = component else {
                return None;
            };
            name.to_str()
        })
        .collect()
}

fn allowed_case_tree_file(components: &[&str]) -> bool {
    match components {
        ["cases", "README.md"]
        | ["cases", "wolfram", "README.md"]
        | ["cases", "wolfram", "core", "README.md"]
        | ["cases", "stateful", "README.md"] => true,
        ["cases", "wolfram", "core", file] | ["cases", "stateful", file] => file.ends_with(".txt"),
        ["cases", "stateful", "packages", file] => file.ends_with(".rum"),
        _ => false,
    }
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

fn issue(path: &Path, kind: ArtifactHygieneIssueKind, reason: &str) -> ArtifactHygieneIssue {
    ArtifactHygieneIssue {
        path: path.to_path_buf(),
        kind,
        reason: reason.to_string(),
    }
}

#[cfg(test)]
#[path = "artifact_hygiene/tests.rs"]
mod tests;
