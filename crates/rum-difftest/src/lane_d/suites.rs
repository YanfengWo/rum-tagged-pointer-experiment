use std::path::{Path, PathBuf};

use crate::fixtures::{crate_root, seed_root};

use super::LaneDSuite;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LaneDInputSuite {
    pub(super) name: &'static str,
    pub(super) path: PathBuf,
}

pub(super) fn suites_for(suite: LaneDSuite) -> Vec<LaneDInputSuite> {
    match suite {
        LaneDSuite::All => vec![timing_suite(), memory_suite()],
        LaneDSuite::Timing => vec![timing_suite()],
        LaneDSuite::Memory => vec![memory_suite()],
        LaneDSuite::CoreLanguage => vec![core_language_suite()],
    }
}

fn timing_suite() -> LaneDInputSuite {
    LaneDInputSuite {
        name: "timing",
        path: seed_root().join("bench/timing_suite.txt"),
    }
}

fn memory_suite() -> LaneDInputSuite {
    LaneDInputSuite {
        name: "memory",
        path: seed_root().join("mem/memory_suite.txt"),
    }
}

fn core_language_suite() -> LaneDInputSuite {
    LaneDInputSuite {
        name: "core-language",
        path: crate_root().join("perf/core-language.txt"),
    }
}

pub fn display_path(path: &Path) -> String {
    path.strip_prefix(seed_root().parent().unwrap_or_else(|| Path::new("")))
        .unwrap_or(path)
        .display()
        .to_string()
}
