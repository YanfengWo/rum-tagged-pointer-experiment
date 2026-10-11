#!/usr/bin/env python3
"""Prepare a persistent pinned source copy; never modify the default core."""
from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile

REVISION = "77029938f999b14dbe5b0755bebdf98e51517ae6"
STAGES = (
    "compact-expr", "recursive-expr", "symbol-sharing", "expression-census", "layout-design"
)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def prepare(output: Path, metadata: Path | None) -> None:
    study = Path(__file__).resolve().parent
    source = study.parent
    repo = study.parents[2]
    output = output.resolve()
    # A destination inside a copied source would recursively copy itself. Build
    # artifacts and evidence are intentionally stored outside the source tree.
    if output == source or source in output.parents:
        raise ValueError("--output must be outside research/tagged-pointer")
    if output.exists() and (not output.is_dir() or any(output.iterdir())):
        raise ValueError("--output must not exist, or must be an empty directory")
    if metadata is not None:
        metadata = metadata.resolve()
        if metadata.exists():
            raise ValueError("--metadata must not overwrite an existing file")
    extension = (study / "symbol-access.rs").read_bytes()
    if b"unsafe" in extension:
        raise ValueError("the proposed additive Symbol API must contain no unsafe")
    for stage in STAGES:
        if not (source / stage).is_dir():
            raise ValueError(f"missing research source: {stage}")

    archive = subprocess.run(
        ["git", "archive", "--format=tar", REVISION], cwd=repo,
        check=True, capture_output=True, timeout=180,
    ).stdout
    output.mkdir(parents=True, exist_ok=True)
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as tar:
        for member in tar.getmembers():
            target = (output / member.name).resolve()
            if target != output and output not in target.parents:
                raise ValueError("archive entry escapes the requested output directory")
            if not (member.isdir() or member.isfile()):
                raise ValueError("archive links and special files are not accepted")
        # All entries have been validated, including links, before extraction.
        tar.extractall(output)
    for stage in STAGES:
        shutil.copytree(
            source / stage, output / "research/tagged-pointer" / stage,
            ignore=shutil.ignore_patterns("target", "evidence", "__pycache__"),
        )
    expr = output / "crates/rum-core/src/expr.rs"
    original = expr.read_bytes()
    result = original + b"\n" + extension
    expr.write_bytes(result)
    record = {
        "upstream_commit": REVISION,
        "overlay_path": "<OUTPUT>",
        "default_core_unchanged": True,
        "isolated_additive_safe_api": True,
        "copied_research_stages": list(STAGES),
        "original_expr_sha256": sha(original),
        "extension_sha256": sha(extension),
        "overlay_expr_sha256": sha(result),
        "patched_file": "crates/rum-core/src/expr.rs",
        "extension_file": "research/tagged-pointer/layout-design/symbol-access.rs",
    }
    metadata = metadata or output / "layout-overlay.json"
    if metadata.exists():
        raise ValueError("metadata must not replace an extracted or copied source file")
    metadata.parent.mkdir(parents=True, exist_ok=True)
    metadata.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print("PASS: prepared persistent pinned source copy with the additive safe Symbol API")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path,
                        help="persistent source directory, absent or empty")
    parser.add_argument("--metadata", type=Path,
                        help="new metadata file; defaults to OUTPUT/layout-overlay.json")
    args = parser.parse_args()
    prepare(args.output, args.metadata)


if __name__ == "__main__":
    main()
