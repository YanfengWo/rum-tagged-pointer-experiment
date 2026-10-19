"""Audit isolated source/test provenance and scoped copy-event diagnostics."""
from collections import Counter
import hashlib
import json
import re

PIN = "77029938f999b14dbe5b0755bebdf98e51517ae6"
ONCE = "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs"
TESTS = {"crates/rum-runtime/src/runtime/tests/copy_attribution_regression.rs",
         "crates/rum-runtime/src/runtime/tests/mod.rs"}
TRACE = {"crates/rum-core/src/expr.rs", "crates/rum-core/src/lib.rs",
         "crates/rum-core/src/copy_attribution.rs"}
FIELDS = ("into_vec_total", "into_vec_clone", "into_owned_total", "into_owned_clone",
          "shared_args_alloc", "shared_expr_alloc")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def read(path):
    return path.read_text(encoding="utf-8")


def fields(line):
    return {k: int(v) if v.isdigit() else v for k, v in re.findall(r"(\w+)=([^ ]+)", line)}


def parse_window(path):
    lines = read(path).splitlines()
    marks = [i for i, line in enumerate(lines) if line.startswith("rum-profile repr ")]
    assert len(marks) == 2, str(path)
    before, after = (fields(lines[i]) for i in marks)
    delta = {key: after[key] - before[key] for key in FIELDS}
    assert all(v >= 0 for v in delta.values())
    events = [fields(line) for line in lines[marks[0] + 1:marks[1]]
              if line.startswith("rum-copy event ")]
    for event in events:
        assert event["kind"] in {"into_vec", "into_owned", "make_mut_clone", "make_mut_dissociate"}
        assert event["slots"] >= 0 and event["shallow_slot_bytes"] == event["slots"] * 32
        assert event["strong"] >= 1 and event["weak"] >= 0
        assert re.fullmatch(r"crates/[A-Za-z0-9_./-]+\.rs:\d+", event["caller"])
    return {"repr_delta": delta, "events": events}


def counter(events):
    return Counter((e["kind"], e["slots"], e["shallow_slot_bytes"], e["caller"]) for e in events)


def difference(active, control):
    net = counter(active["events"])
    net.subtract(counter(control["events"]))
    return [{"kind": kind, "slots": slots, "shallow_slot_bytes": size,
             "caller": caller, "count": count}
            for (kind, slots, size, caller), count in sorted(net.items()) if count]


def audit_sources(out, study):
    repo = study.parents[2]
    base = repo.parent / "base"
    overlays = {}
    for name, mode, expected in [("base-final", "base", TESTS),
            ("candidate", "candidate", TESTS | {ONCE}),
            ("trace-final", "trace", TESTS | TRACE),
            ("candidate-trace", "candidate-trace", TESTS | TRACE | {ONCE})]:
        data = load(out / (name + "-overlay.json"))
        assert data["upstream_commit"] == PIN and data["mode"] == mode
        assert data["default_source_changed"] is False
        assert set(data["patched_files"]) == expected
        for path, entry in data["patched_files"].items():
            assert re.fullmatch(r"[0-9a-f]{64}", entry["patched_sha256"])
            if entry["base_sha256"] is None:
                assert not (base / path).exists() and not (repo / path).exists()
            else:
                assert sha(repo / path) == entry["base_sha256"]
                if base.is_dir():
                    assert sha(base / path) == entry["base_sha256"]
        overlays[name] = data
    for name in overlays:
        for path in TESTS:
            assert overlays[name]["patched_files"][path] == overlays["base-final"]["patched_files"][path]
    for path in TRACE:
        assert overlays["trace-final"]["patched_files"][path] == overlays["candidate-trace"]["patched_files"][path]
    assert overlays["candidate"]["patched_files"][ONCE] == overlays["candidate-trace"]["patched_files"][ONCE]
    matched = load(out / "matched-build.json")
    assert matched["status"] == "PASS" and matched["same_local_source_root"] is True
    assert matched["baseline_source_restored"] and matched["baseline_binary_unchanged"]
    assert matched["source_root"] == overlays["base-final"]["overlay_path"]
    assert matched["changed_file"] == ONCE
    assert matched["baseline_source_sha256"] == overlays["candidate"]["patched_files"][ONCE]["base_sha256"]
    assert matched["candidate_expected_source_sha256"] == overlays["candidate"]["patched_files"][ONCE]["patched_sha256"]
    for key, name in [("baseline_binary_sha256", "base-final"), ("candidate_binary_sha256", "matched-candidate")]:
        assert matched[key] == read(out / (name + "-binary.sha256")).split()[0]
    tests = {}
    pattern = r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;"
    for mode in ["base-final", "candidate"]:
        focused = re.findall(pattern, read(out / (mode + "-focused-tests.log")))
        assert focused == [("3", "0", "0")]
        groups = re.findall(pattern, read(out / (mode + "-workspace.log")))
        counts = [sum(int(r[i]) for r in groups) for i in range(3)]
        # Pinned original has 2238 checks; this study adds exactly three.
        # Earlier 2242 counts included four symbol-access API checks absent here.
        assert counts == [2241, 0, 7] and len(groups) == 29, (mode, counts, len(groups))
        tests[mode] = {"focused_passed": 3, "workspace_passed": 2241, "ignored": 7, "groups": 29,
                       "pinned_original_passed": 2238, "added_regressions": 3}
    for mode in ["base-final", "candidate", "matched-candidate"]:
        assert "COPY_ATTRIBUTION_ACCEPTANCE_PASS" in read(out / (mode + "-acceptance.stdout"))
        assert not read(out / (mode + "-acceptance.stderr")).strip()
    history = {"initial_failure_observed": False}
    failed = out / "candidate-workspace-initial-failed.log"
    if failed.exists():
        assert re.findall(r"test result: FAILED\. (\d+) passed; (\d+) failed;", read(failed)) == [("101", "2")]
        retry = re.findall(pattern, read(out / "candidate-jupyter-pump-retry.log"))
        assert sum(int(r[0]) for r in retry) == 5 and all(r[1:] == ("0", "0") for r in retry)
        history = {"initial_failure_observed": True, "crate": "rum-jupyter", "initial_failed": 2,
                   "isolated_retry_passed": 5, "final_workspace_passed": 2241,
                   "root_cause": "unestablished; retained initial failure and successful reruns"}
    return {"overlays": overlays, "matched_build": matched, "tests": tests,
            "test_history": history, "default_patched_files_match_upstream": True}


def audit_diagnostics(out):
    previous = out.parent / "hotspot-validation"
    old = load(previous / "summary.json")
    all_results = {}
    expected_names = {f"{mode}-{label}-{n}-{kind}" for mode in ["base", "trace"]
                      for label in ["table-pairs", "append-part", "rewrite-wide"]
                      for n in [64, 256, 1024] for kind in ["active", "control"]} | {"rewrite-stack"}
    for suffix, binary_name in [("final", "trace-final"), ("candidate", "candidate-trace")]:
        meta = load(out / ("diagnostics-" + suffix + ".json"))
        raw = out / ("diagnostic-" + suffix + "-raw")
        assert meta["status"] == "PASS" and meta["diagnostic_only"] is True
        assert meta["baseline_sha256"] == old["binary_sha256"]
        assert meta["trace_sha256"] == read(out / (binary_name + "-binary.sha256")).split()[0]
        assert len(meta["runs"]) == 37 and {r["name"] for r in meta["runs"]} == expected_names
        parsed = {}
        for run in meta["runs"]:
            name = run["name"]
            assert run["exit_code"] == 0
            trace = name.startswith("trace-") or name == "rewrite-stack"
            assert run["binary_sha256"] == meta["trace_sha256" if trace else "baseline_sha256"]
            source_name = run["source"].replace("\\", "/").rsplit("/", 1)[-1]
            source = out / source_name if name == "rewrite-stack" else previous / "inputs" / source_name
            assert sha(source) == run["source_sha256"]
            if name != "rewrite-stack":
                assert source_name == name.split("-", 1)[1] + ".rum"
                assert run["source_sha256"] == old["raw_evidence_hashes"]["inputs/" + source_name]
            stdout = read(raw / (name + ".stdout")).replace("\\t", "\t")
            probe = "rewrite-wide-8-active" if name == "rewrite-stack" else name.split("-", 1)[1]
            label, size, kind = probe.rsplit("-", 2)
            assert f"RUM_HOTSPOT_PASS\t{label}\t{size}\t{kind}" in stdout
            parsed[name] = parse_window(raw / (name + ".stderr"))
            if trace:
                for kind, key in [("into_vec", "into_vec_clone"), ("into_owned", "into_owned_clone")]:
                    assert sum(e["kind"] == kind for e in parsed[name]["events"]) == parsed[name]["repr_delta"][key]
            else:
                assert not parsed[name]["events"]
        pairs = []
        for label in ["table-pairs", "append-part", "rewrite-wide"]:
            for n in [64, 256, 1024]:
                stem = f"trace-{label}-{n}-"
                active, control = parsed[stem + "active"], parsed[stem + "control"]
                net = difference(active, control)
                counts = Counter()
                for event in net:
                    counts[event["kind"], event["slots"]] += event["count"]
                if label == "rewrite-wide":
                    expected = {("into_vec", 2): n - 1, ("into_owned", 1): 1}
                    if suffix == "final":
                        expected["into_vec", n] = 1
                    assert counts == Counter(expected), (suffix, n, counts)
                pairs.append({"label": label, "size": n, "active": active, "control": control,
                              "net_copy_events": net, "repr_delta_minus_control": {
                                  k: active["repr_delta"][k] - control["repr_delta"][k] for k in FIELDS}})
        stack = read(raw / "rewrite-stack.stderr")
        assert "replace_children_owned" in stack and "normalize_call_args_shared" in stack
        all_results[suffix] = {"runs": 37, "pairs": pairs, "stack_paths_confirmed": True,
                              "baseline_sha256": meta["baseline_sha256"], "trace_sha256": meta["trace_sha256"]}
    return all_results
