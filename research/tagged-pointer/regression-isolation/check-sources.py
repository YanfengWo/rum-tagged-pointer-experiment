"""Check source syntax and unchanged primary corpus generation."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
import control_cases
import pipeline_cases


def sha(value):
    return hashlib.sha256(value).hexdigest()


def main(out):
    study = Path(__file__).resolve().parent
    sources = {}
    for path in sorted(study.glob("*.py")):
        data = path.read_bytes()
        ast.parse(data.decode("utf-8"), filename=path.name)
        assert len(data) < 10000, (path, len(data))
        sources[path.name] = {"bytes": len(data), "sha256": sha(data)}
    assert control_cases.make_source() == (out / "control-corpus.rum").read_text(encoding="utf-8")
    assert pipeline_cases.make_source() == (out / "pipeline-corpus.rum").read_text(encoding="utf-8")
    plan = json.loads((out / "context-plan.json").read_text(encoding="utf-8"))
    for case in pipeline_cases.CASE_DEFINITIONS:
        if case["label"] in plan["cases"]:
            data = pipeline_cases.make_source([case]).encode()
            assert sha(data) == plan["cases"][case["label"]]["sha256"]
    result = {"status": "PASS", "python_sources": sources, "primary_corpus_generation_unchanged": True,
              "selected_pipeline_generation_matches_fresh_case_evidence": True,
              "scope": "AST and source/corpus provenance checks; not runtime semantic tests or formal proofs."}
    (out / "source-check.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print("PASS: source AST, module-size rule, unchanged primary and selected pipeline generation")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
