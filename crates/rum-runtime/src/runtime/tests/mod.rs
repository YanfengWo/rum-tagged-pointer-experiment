use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("rum-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, source: &str) -> PathBuf {
        let path = self.path.join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn rum_path(path: &Path) -> String {
    normalize_found_path(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

mod arithmetic;
mod associations;
mod associations_mutation;
mod associations_part2;
mod calculus;
mod context_and_loader;
mod debug_tools;
mod definitions_and_rules;
mod definitions_and_rules_part2;
mod definitions_and_rules_part3;
mod definitions_and_rules_part4;
mod definitions_updates;
mod functional_ops;
mod holding_and_control;
mod holding_and_control_part2;
mod holding_and_control_part3;
mod holding_and_control_part4;
mod holding_and_control_part5;
mod holding_and_control_part6;
mod holding_and_control_part7;
mod holding_and_control_part8;
mod logic;
mod logic_part2;
mod lowering_and_context;
mod lowering_golden;
mod messages;
mod messages_part2;
mod messages_part3;
mod names_and_syntax_information;
mod options_and_protection;
mod package_tools;
mod polynomials;
mod predicates;
mod predicates_part2;
mod scoping_unique;
mod session_config;
mod session_requests;
mod strings;
mod structure_ops;
mod thread_and_dispatch;
mod thread_and_dispatch_part2;
mod traversal;
mod traversal_part2;
