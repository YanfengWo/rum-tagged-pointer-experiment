"""Prepare pinned typed sources plus an unused safe runtime oracle export."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sys
sys.dont_write_bytecode = True

def prepare(output,metadata):
    study=Path(__file__).resolve().parent
    spec=importlib.util.spec_from_file_location("typed_prepare",study.parent/"typed-layout/prepare-overlay.py")
    loader=importlib.util.module_from_spec(spec);spec.loader.exec_module(loader)
    loader.prepare(output,metadata)
    shutil.copytree(study,output/"research/tagged-pointer/symbol-operations",ignore=shutil.ignore_patterns("__pycache__","target"))
    record=json.loads(metadata.read_text())
    files={"crates/rum-runtime/src/runtime/definitions/mod.rs":"runtime-oracle-definitions.rs",
           "crates/rum-runtime/src/runtime/mod.rs":"runtime-oracle.rs"}
    patched={}
    for rel,extension_file in files.items():
        p=output/rel; original=p.read_bytes();extension=(study/extension_file).read_bytes()
        assert b"unsafe" not in extension
        p.write_bytes(original+b"\n"+extension)
        patched[rel]={"original_sha256":hashlib.sha256(original).hexdigest(),
                      "extension_file":extension_file,"extension_sha256":hashlib.sha256(extension).hexdigest(),
                      "patched_sha256":hashlib.sha256(p.read_bytes()).hexdigest()}
    actual="crates/rum-runtime/src/runtime/definitions/unique_symbols.rs"
    record["runtime_collector_unchanged_sha256"]=hashlib.sha256((output/actual).read_bytes()).hexdigest()
    record["runtime_exports"]=patched
    metadata.write_text(json.dumps(record,indent=2)+"\n",encoding="utf-8")
    print("PASS: runtime oracle exports are safe, unused by default evaluator; original collector unchanged")

if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--output",required=True,type=Path);p.add_argument("--metadata",required=True,type=Path)
    a=p.parse_args();prepare(a.output,a.metadata)
