"""Artifact-preserving semantic probes for the same-image build driver."""
import difflib
import re
import subprocess

from build_support import sha, digest
import workloads
import shape_workloads


def source_record(original, common, out):
    changed = sorted(n for n, d in common.items() if original.get(n) != d)
    expected = {"crates/rum-runtime/src/runtime/" + n for n in (
        "definitions/application/replace_all.rs", "kernel_data/rewrite_data/replace/once.rs",
        "mod.rs", "rewrite_shortcuts_switch.rs", "session/session_state.rs")}
    assert set(changed) == expected and set(original) <= set(common)
    assert not any(b"rewrite_work_ledger" in d for n, d in common.items() if n.endswith(".rs"))
    diff = "".join("".join(difflib.unified_diff(
        original.get(n, b"").decode().splitlines(keepends=True),
        common[n].decode().splitlines(keepends=True),
        fromfile="a/" + n if n in original else "/dev/null", tofile="b/" + n)) for n in changed)
    (out / "common-source-diff.patch").write_text(diff, encoding="utf-8")
    return {"common_source_diff_sha256": digest(diff.encode()),
            "common_patched_files": {n: {"source_sha256": digest(common[n]),
                "base_sha256": digest(original[n]) if n in original else None,
                "bytes": len(common[n])} for n in changed},
            "common_source_hashes": {n: digest(d) for n, d in common.items()}}


def probe(binary, out, mode, label, path, marker, bounded, env, source):
    result = subprocess.run(bounded + ["--timeout", "120", "--", str(binary),
                            "--file", str(path)], env=env, cwd=source,
                            stdin=subprocess.DEVNULL, capture_output=True, timeout=135)
    (out / f"{mode}-{label}.stdout").write_bytes(result.stdout)
    (out / f"{mode}-{label}.stderr").write_bytes(result.stderr)
    assert result.returncode == 0 and marker.encode() in result.stdout and not result.stderr.strip()
    assert b"rum-work\t" not in result.stdout + result.stderr
    return {"source": str(path), "source_sha256": sha(path), "exit_code": result.returncode}


def workspace(path, primary, expected):
    log = path.read_text(encoding="utf-8")
    groups = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", log)
    result = {"passed": sum(int(r[0]) for r in groups),
              "ignored": sum(int(r[2]) for r in groups), "groups": len(groups)}
    assert result == expected and all(int(r[1]) == 0 for r in groups), result
    for name in ("matcher", "depth"):
        for test in re.findall(r"#\[test\]\s*fn (\w+)\(", (primary / f"{name}-tests.rs").read_text()):
            assert f"rewrite_shortcuts_{name}::{test} ... ok" in log
    return result


def preflight_sources(out):
    corpora = dict(workloads.sources(False),
                   shape=(shape_workloads.source(False), shape_workloads.LABELS, "ISO_SHAPE_PASS"),
                   semantic=(workloads.semantic_source(), [], "ISO_SEMANTICS_PASS"))
    for label, (text, _, _) in corpora.items():
        assert "AbsoluteTiming" not in text
        (out / f"{label}-preflight.rum").write_text(text, encoding="utf-8")
    return corpora
