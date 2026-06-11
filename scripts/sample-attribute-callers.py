#!/usr/bin/env python3
"""Attribute a target leaf symbol's samples to its nearest interesting callers."""
import re
import sys
from collections import defaultdict

LINE = re.compile(r"^([ +!:|.]*?)(\d+)\s+(.*?)(?:\s+\(in [^)]*\))?(?:\s+\+\s+\d+)?\s*(?:\[0x[0-9a-f]+\])?\s*$")
HASH = re.compile(r"::h[0-9a-f]{16}$")

SKELETON = (
    "resume_call_with_dispatch_plan", "run_call_dispatch_stage",
    "evaluate_once_unstable", "evaluate_inner_unstable",
    "Session::evaluate", "drive_internal_value",
    "resume_compound_expression", "eval_compound_expression",
    "resume_prepared_call_arguments", "core::fmt", "Display>::fmt",
    "memcmp", "malloc", "free", "memmove", "deduplicated",
    "Clone>::clone", "drop_in_place", "Arc<", "RawVec", "hashbrown",
    "finish_grow", "grow_one", "bzero", "DYLD-STUB",
)


def clean(sym: str) -> str:
    sym = HASH.sub("", sym.strip())
    sym = sym.replace("_$LT$impl$u20$rum_runtime..runtime..model..Session$GT$", "Session")
    sym = sym.replace("$LT$", "<").replace("$GT$", ">").replace("$u20$", " ").replace("..", "::")
    return sym


def main(path: str, target: str, top: int) -> None:
    by_caller = defaultdict(int)
    stack = []  # (indent, symbol)
    in_graph = False
    for line in open(path, errors="replace"):
        if line.startswith("Call graph"):
            in_graph = True
            continue
        if in_graph and line.startswith("Total number in stack"):
            break
        if not in_graph:
            continue
        m = LINE.match(line.rstrip("\n"))
        if not m:
            continue
        markers, count, sym = m.group(1), int(m.group(2)), clean(m.group(3))
        indent = len(markers)
        while stack and stack[-1][0] >= indent:
            stack.pop()
        if target in sym and all(target not in s for _, s in stack):
            caller = "?"
            for _, s in reversed(stack):
                if not any(k in s for k in SKELETON):
                    caller = s
                    break
            by_caller[caller] += count
        stack.append((indent, sym))

    total = sum(by_caller.values())
    print(f"target '{target}': {total} samples, by nearest non-skeleton caller:")
    for sym, t in sorted(by_caller.items(), key=lambda kv: -kv[1])[:top]:
        print(f"{t:8d} {100.0 * t / max(total, 1):5.1f}%  {sym}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], int(sys.argv[3]) if len(sys.argv) > 3 else 25)
