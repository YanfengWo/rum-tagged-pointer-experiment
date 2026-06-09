use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::oracle::{OracleError, OracleKernel};

use super::*;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rum-difftest-{prefix}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn test_kernel() -> OracleKernel {
    OracleKernel::with_limits("/bin/true", Duration::from_secs(1), 1024).unwrap()
}

#[test]
fn parses_session_batch_markers_into_nested_results() {
    let kernel = test_kernel();
    let batches = vec![
        vec!["x = 2".to_string()],
        vec!["y = 4".to_string(), "y + 1".to_string()],
    ];
    let stdout = concat!(
        "debug line\n",
        "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=1\t1\t2\n",
        "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=2\t1\t4\n",
        "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=2\t2\t5\n",
    );

    let results = parse_session_batch_results(&kernel, &batches, stdout, "").unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0][0].as_deref(), Some("2"));
    assert_eq!(results[1][0].as_deref(), Some("4"));
    assert_eq!(results[1][1].as_deref(), Some("5"));
}

#[test]
fn rejects_missing_or_malformed_session_batch_markers() {
    let kernel = test_kernel();
    let batches = vec![vec!["x = 2".to_string()], vec!["y = 4".to_string()]];
    let missing = "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=1\t1\t2\n";

    let err = parse_session_batch_results(&kernel, &batches, missing, "stderr").unwrap_err();
    assert!(matches!(
        err,
        OracleError::SessionResultCountMismatch {
            expected: 2,
            actual: 1,
            ..
        }
    ));

    let malformed = "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=3\t1\tbad\n";
    let err = parse_session_batch_results(&kernel, &batches, malformed, "stderr").unwrap_err();
    assert!(matches!(
        err,
        OracleError::InvalidSessionResultMarker { .. }
    ));
}

#[test]
fn builds_session_batch_evaluations_from_per_step_stream_files() {
    let temp = TestDir::new("session-batch-eval");
    let first_batch = temp.path().join("batch-0001");
    let second_batch = temp.path().join("batch-0002");
    fs::create_dir(&first_batch).unwrap();
    fs::create_dir(&second_batch).unwrap();
    fs::write(
        first_batch.join("messages-0001-0001.txt"),
        "first message\n",
    )
    .unwrap();
    fs::write(first_batch.join("prints-0001-0001.txt"), "first print\n").unwrap();
    fs::write(second_batch.join("messages-0002-0001.txt"), "").unwrap();
    fs::write(second_batch.join("prints-0002-0001.txt"), "").unwrap();
    let results = vec![vec![Some("2".to_string())], vec![Some("5".to_string())]];
    let stdout = concat!(
        "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=1\t1\t2\n",
        "process print\n",
        "__RUM_DIFFTEST_SESSION_BATCH_RESULT__=2\t1\t5\n",
    );

    let evaluations =
        build_session_batch_evaluations(temp.path(), results, stdout, "process stderr").unwrap();

    assert_eq!(evaluations.len(), 2);
    assert_eq!(evaluations[0][0].result, "2");
    assert_eq!(evaluations[0][0].stderr, "first message\nprocess stderr");
    assert_eq!(
        evaluations[0][0].printed_output,
        "first print\nprocess print"
    );
    assert_eq!(evaluations[1][0].result, "5");
    assert_eq!(evaluations[1][0].stderr, "process stderr");
    assert_eq!(evaluations[1][0].printed_output, "process print");
}
