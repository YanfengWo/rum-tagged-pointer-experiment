"""Synthetic complete programs; these cases do not represent measured user apps.

Every retained AbsoluteTiming includes input construction, optional sequential
Replace passes, Sort, List Part writes, and assembly of a result with aliases.
Expected outputs and all SameQ checks are outside that timing window. Retained
per-process samples are repetitions, not independent experiment blocks.
"""

SIZES = (4000, 16000)
REPETITIONS = 11
MARKER = "ISO_TIME"
PASS_MARKER = "ISO_PIPELINE_PASS"

CASE_DEFINITIONS = [
    {
        "label": f"synthetic-{family}-{size}",
        "family": family,
        "size": size,
        "rewrite_passes": passes,
        "part_writes": writes,
    }
    for family, passes, writes in (
        ("rewrite-heavy-hit", 8, 1),
        ("mixed-hit", 1, 64),
        ("mixed-control", 0, 64),
    )
    for size in SIZES
]
CASES = [case["label"] for case in CASE_DEFINITIONS]


def _operation(case):
    size = case["size"]
    passes = case["rewrite_passes"]
    writes = case["part_writes"]
    rewrite = (
        f"Do[current = Replace[current, x_Integer :> x + 1, {{1}}], "
        f"{{step, 1, {passes}}}];"
        if passes
        else "(* No Replace call in this control. *)"
    )
    return f"""Module[{{source, sourceAlias, current, sorted, sortedAlias, i, step, x}},
  source = Reverse[Range[{size}]];
  sourceAlias = source;
  current = source;
  {rewrite}
  sorted = Sort[current];
  sortedAlias = sorted;
  Do[sorted[[i]] = -i, {{i, 1, {writes}}}];
  {{sorted, sortedAlias, source, sourceAlias}}
]"""


def _expected(case):
    size = case["size"]
    passes = case["rewrite_passes"]
    writes = case["part_writes"]
    # Position k of the sorted, unmodified list is k + passes. Replacing the
    # prefix with -k leaves an independently specified arithmetic suffix.
    return (
        "{Join[Table[-i, {i, 1, " + str(writes) + "}], "
        f"Range[{writes + passes + 1}, {size + passes}]], "
        f"Range[{passes + 1}, {size + passes}], "
        f"Reverse[Range[{size}]], Reverse[Range[{size}]]}}"
    )


def make_source(cases=None):
    """Return a .rum corpus with a warm run and eleven retained runs per case.

    Each result carries the mutated sorted list, the pre-mutation sorted alias,
    and both original-input aliases. Thus successful full-result comparison
    checks the rewrite arithmetic, ordering, Part effect, and alias preservation.
    The no-Replace control has its own expected values: its elapsed time is not
    subtracted from another case or treated as an application speed estimate.
    """
    source = f'''(* Synthetic pipeline programs; no external-engine oracle. *)
SetAttributes[pipelineMeasure, HoldAll];
pipelineMeasure[label_String, expression_, expected_] := Module[{{i, result, seconds}},
  result = expression;
  If[SameQ[result, expected], Null, Exit[2]];
  Do[
    seconds = First[AbsoluteTiming[result = expression]];
    Print[StringJoin["{MARKER}\\t", label, "\\t", ToString[i], "\\t", ToString[seconds]]];
    If[SameQ[result, expected], Null, Exit[2]],
    {{i, 1, {REPETITIONS}}}
  ];
];
'''
    for case in CASE_DEFINITIONS if cases is None else cases:
        # Set expected once before invoking the HoldAll measurement wrapper;
        # neither generating expected lists nor validating them is timed.
        source += f"pipelineExpected = {_expected(case)};\n"
        source += (
            f'pipelineMeasure["{case["label"]}", '
            f"{_operation(case)}, pipelineExpected];\n"
        )
    return source + f'Print["{PASS_MARKER}"];\n'
