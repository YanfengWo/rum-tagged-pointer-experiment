#!/usr/bin/env python3
"""Compute per-function self-time from macOS `sample` call-graph output.

`sample` prints an indented call tree where each line is
    <markers> <count> <symbol> (in <image>) + <offset> [<addr>]
Self time for a node = its count minus the sum of its direct children's
counts. Aggregating self time by symbol gives the flat profile that the
call-graph view hides for deeply recursive programs.
"""
import re
import sys
from collections import defaultdict

LINE = re.compile(r"^([ +!:|.]*?)(\d+)\s+(.*?)(?:\s+\(in [^)]*\))?(?:\s+\+\s+\d+)?\s*(?:\[0x[0-9a-f]+\])?\s*$")
HASH = re.compile(r"::h[0-9a-f]{16}$")


def clean(sym: str) -> str:
    sym = HASH.sub("", sym.strip())
    sym = sym.replace("_$LT$impl$u20$rum_runtime..runtime..model..Session$GT$", "Session")
    sym = sym.replace("$LT$", "<").replace("$GT$", ">").replace("$u20$", " ").replace("..", "::")
    return sym


def main(path: str, top: int) -> None:
    self_time = defaultdict(int)
    incl_time = defaultdict(int)  # inclusive, counted once per stack via seen-set
    stack = []  # list of (indent, count, symbol, child_sum)
    total = 0

    def pop_to(indent):
        nonlocal stack
        while stack and stack[-1][0] >= indent:
            ind, cnt, sym, child_sum = stack.pop()
            self_time[sym] += cnt - child_sum
            if stack:
                stack[-1][3] += cnt

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
        if "Thread_" in sym:
            pop_to(0)
            total += count
            continue
        pop_to(indent)
        stack.append([indent, count, sym, 0])
        # inclusive: add count once per (symbol, path) — approximate by only
        # counting when symbol not already on the parse stack (recursion-safe)
        if all(s[2] != sym for s in stack[:-1]):
            incl_time[sym] += count
    pop_to(0)

    print(f"total samples: {total}")
    print(f"\n== TOP {top} BY SELF TIME ==")
    for sym, t in sorted(self_time.items(), key=lambda kv: -kv[1])[:top]:
        print(f"{t:8d} {100.0 * t / total:5.1f}%  {sym}")
    print(f"\n== TOP {top} BY INCLUSIVE TIME (recursion-deduped) ==")
    for sym, t in sorted(incl_time.items(), key=lambda kv: -kv[1])[:top]:
        print(f"{t:8d} {100.0 * t / total:5.1f}%  {sym}")


if __name__ == "__main__":
    main(sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 40)
