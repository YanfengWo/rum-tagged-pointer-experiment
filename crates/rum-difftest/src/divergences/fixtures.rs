use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::fixtures::{GoldenEntry, crate_root, golden_dir, load_text, parse_golden_file};

use super::{KnownDifferenceLedgerError, KnownDifferenceReference};

pub(super) fn collect_fixture_known_differences()
-> Result<Vec<KnownDifferenceReference>, KnownDifferenceLedgerError> {
    let mut paths = Vec::new();
    collect_txt_files(&golden_dir(), &mut paths).map_err(|source| {
        KnownDifferenceLedgerError::Io {
            path: golden_dir(),
            source,
        }
    })?;
    collect_txt_files(&crate_root().join("cases"), &mut paths).map_err(|source| {
        KnownDifferenceLedgerError::Io {
            path: crate_root().join("cases"),
            source,
        }
    })?;
    paths.sort();

    let mut references = Vec::new();
    let mut seen_ids = BTreeMap::<String, PathBuf>::new();
    for path in paths {
        let text = load_text(&path).map_err(|source| KnownDifferenceLedgerError::Io {
            path: path.clone(),
            source,
        })?;
        let entries = parse_golden_file(&text).map_err(|source| {
            KnownDifferenceLedgerError::FixtureParse {
                path: path.clone(),
                source,
            }
        })?;
        for entry in entries {
            let GoldenEntry::KnownDifference { metadata, .. } = entry else {
                continue;
            };
            let id = metadata
                .get("id")
                .expect("fixture parser validates known-difference ids")
                .clone();
            let fixture = relative_to_crate_root(&path);
            if let Some(first_fixture) = seen_ids.insert(id.clone(), fixture.clone()) {
                return Err(KnownDifferenceLedgerError::DuplicateFixtureId {
                    id,
                    first_fixture,
                    second_fixture: fixture,
                });
            }
            references.push(KnownDifferenceReference {
                id,
                fixture,
                metadata,
            });
        }
    }

    Ok(references)
}

fn collect_txt_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_txt_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("txt") {
            out.push(path);
        }
    }
    Ok(())
}

fn relative_to_crate_root(path: &Path) -> PathBuf {
    path.strip_prefix(crate_root())
        .unwrap_or(path)
        .to_path_buf()
}
