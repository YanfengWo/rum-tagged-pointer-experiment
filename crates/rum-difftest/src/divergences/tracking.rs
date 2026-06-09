use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::fixtures::crate_root;

use super::{KnownDifferenceRecord, KnownDifferenceTrackingTargetIssue};

pub(super) fn collect_tracking_target_issues(
    records: &[KnownDifferenceRecord],
) -> Vec<KnownDifferenceTrackingTargetIssue> {
    records
        .iter()
        .filter_map(|record| {
            let target = record
                .fields
                .get("tracked_by")
                .expect("required tracked_by field was already validated");
            validate_tracking_target(target).err().map(|message| {
                KnownDifferenceTrackingTargetIssue {
                    id: record.id.clone(),
                    target: target.clone(),
                    message,
                }
            })
        })
        .collect()
}

fn validate_tracking_target(target: &str) -> Result<(), String> {
    let Some((path_text, needle)) = target.split_once(':') else {
        return Err("expected `path:needle`".to_string());
    };
    if needle.trim().is_empty() {
        return Err("tracking target needle is empty".to_string());
    }

    let relative_path = Path::new(path_text.trim());
    if relative_path.as_os_str().is_empty() {
        return Err("tracking target path is empty".to_string());
    }
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("tracking target path must stay inside the repository".to_string());
    }

    let Some(path) = resolve_tracking_target_path(relative_path) else {
        return Err(format!(
            "tracking target file `{}` does not exist",
            relative_path.display()
        ));
    };
    let text = fs::read_to_string(&path)
        .map_err(|source| format!("failed to read `{}`: {source}", path.display()))?;
    if !text.contains(needle.trim()) {
        return Err(format!(
            "tracking target `{}` does not contain `{}`",
            path.display(),
            needle.trim()
        ));
    }

    Ok(())
}

fn resolve_tracking_target_path(relative_path: &Path) -> Option<PathBuf> {
    let repo_path = repo_root().join(relative_path);
    if repo_path.is_file() {
        return Some(repo_path);
    }

    let crate_path = crate_root().join(relative_path);
    crate_path.is_file().then_some(crate_path)
}

fn repo_root() -> PathBuf {
    let crate_path = crate_root();
    crate_path
        .parent()
        .and_then(Path::parent)
        .unwrap_or(crate_path.as_path())
        .to_path_buf()
}
