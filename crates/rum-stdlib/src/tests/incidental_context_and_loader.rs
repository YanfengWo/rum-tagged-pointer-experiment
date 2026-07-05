use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::install_core;
use crate::tests::install_test_file_io;
use rum_core::Expr;
use rum_runtime::Session;

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

fn normalize_found_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    })
}

fn rum_path(path: &Path) -> String {
    normalize_found_path(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn core_session() -> Session {
    let mut session = Session::new();
    install_test_file_io(&mut session);
    install_core(&mut session);
    session
}

#[test]
fn path_variables_and_file_name_helpers_cover_current_script_environment_subset() {
    let root = TempDirGuard::new("file-name-helpers");
    let package = root.write("Pkg.rum", "BeginPackage[\"Pkg`\"]\nvalue = 7\nEndPackage[]");
    let nested = root.write(
        "nested/script.rum",
        "{StringQ[$InputFileName], FileNameTake[$InputFileName], DirectoryName[$InputFileName]}",
    );

    let mut session = core_session();
    assert_eq!(
        session
            .eval_text("$InputFileName")
            .unwrap()
            .value
            .to_string(),
        "\"\""
    );

    assert_eq!(
        session
            .eval_text("FileNameJoin[{\"alpha\", \"beta\", \"gamma.wl\"}]")
            .unwrap()
            .value
            .to_string(),
        "\"alpha/beta/gamma.wl\""
    );
    assert_eq!(
        session
            .eval_text("FileNameJoin[{}]")
            .unwrap()
            .value
            .to_string(),
        "\"\""
    );
    assert_eq!(
        session
            .eval_text(&format!("FileNameTake[\"{}\"]", rum_path(&package)))
            .unwrap()
            .value
            .to_string(),
        "\"Pkg.rum\""
    );

    let nested_dir = nested.parent().unwrap();
    let nested_dir_rendered = format!("{}/", rum_path(nested_dir));
    let loaded = session
        .eval_text(&format!("Get[\"{}\"]", rum_path(&nested)))
        .unwrap();
    assert_eq!(
        loaded.value.to_string(),
        format!("List[True, \"script.rum\", \"{nested_dir_rendered}\"]")
    );

    let parent = session
        .eval_text(&format!("ParentDirectory[\"{}\"]", rum_path(nested_dir)))
        .unwrap();
    assert_eq!(
        parent.value.to_string(),
        format!("\"{}/\"", rum_path(root.path()))
    );

    assert_eq!(
        session
            .eval_text(&format!(
                "$Path = {{\"{}\"}}; FindFile[\"Pkg`\"]",
                rum_path(root.path())
            ))
            .unwrap()
            .value
            .to_string(),
        Expr::string(rum_path(&package)).to_string()
    );
}
