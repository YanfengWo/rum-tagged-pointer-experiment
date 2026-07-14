use super::*;

#[test]
fn kernel_crates_do_not_use_std_fs_outside_tests() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should live under crates/rum-runtime");
    let roots = [
        workspace.join("crates/rum-core/src"),
        workspace.join("crates/rum-runtime/src"),
        workspace.join("crates/rum-stdlib/src"),
    ];
    let mut violations = Vec::new();

    for root in roots {
        collect_std_fs_violations(&root, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "std::fs is forbidden in rum-core, rum-runtime, and rum-stdlib outside /tests/ paths; use host-injected FileIo for kernel I/O. Violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn kernel_crates_do_not_write_stdout_outside_tests_and_bins() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("runtime crate should live under crates/rum-runtime");
    let roots = [
        workspace.join("crates/rum-core/src"),
        workspace.join("crates/rum-runtime/src"),
        workspace.join("crates/rum-stdlib/src"),
    ];
    let mut violations = Vec::new();

    for root in roots {
        collect_stdout_violations(&root, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "stdout writes are forbidden in rum-core, rum-runtime, and rum-stdlib outside /tests/ and /bin/ paths; /bin/ is excluded because codegen tools own their stdout. Violations:\n{}",
        violations.join("\n")
    );
}

fn collect_std_fs_violations(path: &Path, violations: &mut Vec<String>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("hygiene test should read source tree") {
            let entry = entry.expect("hygiene test should read source entry");
            collect_std_fs_violations(&entry.path(), violations);
        }
        return;
    }

    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return;
    }

    let normalized = path.display().to_string().replace('\\', "/");
    if normalized.contains("/tests/") {
        return;
    }

    let contents = fs::read_to_string(path).expect("hygiene test should read source file");
    if contents.contains("std::fs") {
        violations.push(normalized);
    }
}

fn collect_stdout_violations(path: &Path, violations: &mut Vec<String>) {
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("hygiene test should read source tree") {
            let entry = entry.expect("hygiene test should read source entry");
            collect_stdout_violations(&entry.path(), violations);
        }
        return;
    }

    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return;
    }

    let normalized = path.display().to_string().replace('\\', "/");
    if normalized.contains("/tests/") || normalized.contains("/bin/") {
        return;
    }

    let contents = fs::read_to_string(path).expect("hygiene test should read source file");
    let needles = ["println!", "print!", "io::stdout", "stdout()"];
    if needles
        .iter()
        .any(|needle| contains_unprefixed_needle(&contents, needle))
    {
        violations.push(normalized);
    }
}

fn contains_unprefixed_needle(contents: &str, needle: &str) -> bool {
    let contents = contents.as_bytes();
    let needle = needle.as_bytes();

    if needle.len() > contents.len() {
        return false;
    }

    (0..=contents.len() - needle.len()).any(|start| {
        contents[start..start + needle.len()] == *needle
            && (start == 0
                || !(contents[start - 1].is_ascii_alphanumeric() || contents[start - 1] == b'_'))
    })
}
