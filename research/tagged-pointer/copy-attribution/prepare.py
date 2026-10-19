"""Prepare pinned original/diagnostic copies; default checkouts stay unchanged."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile

PIN = "77029938f999b14dbe5b0755bebdf98e51517ae6"
def sha(data):
    return hashlib.sha256(data).hexdigest()

def prepare(output, metadata, mode):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    assert not output.exists() or not any(output.iterdir())
    assert not metadata.exists()
    archive = subprocess.run(["git", "archive", "--format=tar", PIN], cwd=repo, capture_output=True, check=True, timeout=180).stdout
    output.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as source:
        for item in source:
            path = (output / item.name).resolve()
            assert output.resolve() in path.parents and (item.isfile() or item.isdir())
        source.extractall(output)
    patched = {}
    def patch(name, transform):
        path = output / name
        before = path.read_bytes()
        path.write_text(transform(before.decode("utf-8")), encoding="utf-8", newline="\n")
        patched[name] = {"base_sha256": sha(before)}
    if mode in {"candidate", "candidate-trace"}:
        name = "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs"
        before = (output / name).read_bytes()
        patch_text = (study / "candidate.patch").read_text(encoding="utf-8")
        subprocess.run(["git", "apply", "--check", "-"], input=patch_text, text=True, cwd=output, check=True)
        subprocess.run(["git", "apply", "-"], input=patch_text, text=True, cwd=output, check=True)
        patched[name] = {"base_sha256": sha(before)}
    test_name = "crates/rum-runtime/src/runtime/tests/copy_attribution_regression.rs"
    shutil.copyfile(study / "regression-tests.rs", output / test_name)
    patched[test_name] = {"base_sha256": None}
    patch("crates/rum-runtime/src/runtime/tests/mod.rs", lambda value: value + "\nmod copy_attribution_regression;\n")
    if mode in {"trace", "candidate-trace"}:
        def modify(value):
            for signature in ["    pub fn into_owned(self) -> Expr {", "    pub fn into_vec(self) -> Vec<Expr> {", "    pub fn make_mut(&mut self) -> &mut Vec<Expr> {"]:
                assert value.count(signature) == 1
                value = value.replace(signature, "    #[track_caller]\n" + signature)
            before = "                record_into_vec(true);\n                args.as_ref().clone()"
            after = '                crate::copy_attribution::record("into_vec", args.len(), Arc::strong_count(&args), Arc::weak_count(&args));\n' + before
            assert value.count(before) == 1
            value = value.replace(before, after)
            before = "                record_into_owned(true);\n                expr.as_ref().clone()"
            after = '                crate::copy_attribution::record("into_owned", 1, Arc::strong_count(&expr), Arc::weak_count(&expr));\n' + before
            assert value.count(before) == 1
            value = value.replace(before, after)
            before = "        let original = Arc::as_ptr(&self.0);\n        let args = Arc::make_mut(&mut self.0);"
            after = "        let strong = Arc::strong_count(&self.0);\n        let weak = Arc::weak_count(&self.0);\n" + before
            assert value.count(before) == 1
            value = value.replace(before, after)
            before = "        if !std::ptr::eq(original, args) {\n            record_shared_args_alloc();"
            after = '        if !std::ptr::eq(original, args) {\n            crate::copy_attribution::record(if strong > 1 { "make_mut_clone" } else { "make_mut_dissociate" }, args.len(), strong, weak);\n            record_shared_args_alloc();'
            assert value.count(before) == 1
            return value.replace(before, after)
        patch("crates/rum-core/src/expr.rs", modify)
        patch("crates/rum-core/src/lib.rs", lambda value: value + "\nmod copy_attribution;\n")
        name = "crates/rum-core/src/copy_attribution.rs"
        shutil.copyfile(study / "diagnostic.rs", output / name)
        patched[name] = {"base_sha256": None}
    rust = [str(output / name) for name in patched if name.endswith(".rs")]
    if rust:
        subprocess.run(["rustup", "run", "1.85.0", "rustfmt", "--edition", "2024", *rust], check=True, timeout=180)
    for name, entry in patched.items():
        entry["patched_sha256"] = sha((output / name).read_bytes())
    metadata.parent.mkdir(parents=True, exist_ok=True)
    metadata.write_text(json.dumps({"upstream_commit": PIN, "mode": mode, "overlay_path": str(output), "patched_files": patched, "default_source_changed": False}, indent=2) + "\n", encoding="utf-8")
    print("Prepared", mode)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--mode", choices=["base", "trace", "candidate", "candidate-trace"], required=True)
    args = parser.parse_args()
    prepare(args.output.resolve(), args.metadata.resolve(), args.mode)
