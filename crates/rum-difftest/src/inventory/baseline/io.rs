use std::fs;
use std::path::{Path, PathBuf};

use super::{
    InventoryBaseline, InventoryBaselineError, format_inventory_baseline, parse_inventory_baseline,
};
use crate::fixtures::{crate_root, load_text};

pub fn default_inventory_baseline_path() -> PathBuf {
    crate_root().join("coverage/inventory-baseline.txt")
}

pub fn load_inventory_baseline(
    path: impl AsRef<Path>,
) -> Result<InventoryBaseline, InventoryBaselineError> {
    let path = path.as_ref();
    let text = load_text(path).map_err(|source| InventoryBaselineError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_inventory_baseline(&text)
}

pub fn write_inventory_baseline(
    path: impl AsRef<Path>,
    baseline: &InventoryBaseline,
) -> Result<(), InventoryBaselineError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| InventoryBaselineError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
    }
    fs::write(path, format_inventory_baseline(baseline)).map_err(|source| {
        InventoryBaselineError::Io {
            path: path.to_path_buf(),
            source,
        }
    })
}
