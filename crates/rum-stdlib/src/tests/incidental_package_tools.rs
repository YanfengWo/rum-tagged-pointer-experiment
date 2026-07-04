use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::install_core;
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
        let path = std::env::temp_dir().join(format!("rum-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn rum_path(path: &Path) -> String {
    let normalized = fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    });
    normalized.display().to_string().replace('\\', "/")
}

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

#[test]
fn dump_save_restore_preserves_registered_core_builtins() {
    let dir = TempDirGuard::new("dumpsave-core-builtins");
    let file = dir.path().join("saved.mx");
    let mut session = core_session();

    assert_installed_builtins_registered(&mut session);
    session.eval_text("f[x_] := x + 1").unwrap();
    session
        .eval_text(&format!("DumpSave[File[\"{}\"], f]", rum_path(&file)))
        .unwrap();

    let mut restored = core_session();
    restored
        .eval_text(&format!("Get[File[\"{}\"]]", rum_path(&file)))
        .unwrap();

    assert_installed_builtins_registered(&mut restored);
    assert_eq!(restored.eval_text("f[1]").unwrap().value.to_string(), "2");
}

fn assert_installed_builtins_registered(session: &mut Session) {
    let length = session
        .inspect("Length")
        .expect("Length should inspect as a known system symbol");
    assert!(length.is_builtin);
    assert_eq!(
        session
            .eval_text("Length[{1, 2, 3}]")
            .unwrap()
            .value
            .to_string(),
        "3"
    );
}
