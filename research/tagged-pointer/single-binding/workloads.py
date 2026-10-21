"""Rum-owned value/alias checks and bounded synthetic rewrite workloads.

Only operation evaluation is timed. Expected full values and alias checks are
outside each timing window; retained repetitions are not independent blocks.
Resource-observing ByteCount/MemoryInUse equality is deliberately not asserted.
"""

REPETITIONS = 11
FOCUS_SIZES = (1024, 16000)
FAMILIES = ("increment", "increment-reversed", "identity", "miss", "opaque-body", "flat-nested")
FOCUS_CASES = [f"focus-{family}-{size}" for family in FAMILIES for size in FOCUS_SIZES]
PIPELINE_DEFINITIONS = [
    (f"synthetic-{family}-{size}", size, passes, writes)
    for family, passes, writes in (("rewrite-heavy-hit", 8, 1), ("mixed-hit", 1, 64), ("mixed-control", 0, 64))
    for size in (4000, 16000)
]
PIPELINE_CASES = [case[0] for case in PIPELINE_DEFINITIONS]


def wrapper(timed):
    """A warm full-value check, then retained checks, then result alias mutation."""
    retained = f'''Do[
    seconds = First[AbsoluteTiming[result = expression]];
    Print[StringJoin["ISO_TIME\\t", label, "\\t", ToString[i], "\\t", ToString[seconds]]];
    If[SameQ[result, expected], Null, Exit[2]],
    {{i, 1, {REPETITIONS}}}
  ];''' if timed else ""
    return f'''SetAttributes[flatMeasure, HoldAll];
flatMeasure[label_String, expression_, expected_] := Module[{{i, result, resultAlias, seconds}},
  result = expression;
  If[SameQ[result, expected], Null, Exit[2]];
  {retained}
  resultAlias = result;
  result[[1]] = flatResultMutation;
  If[SameQ[resultAlias, expected], Null, Exit[2]];
];
'''


def focus_source(timed=True):
    source = wrapper(timed) + "SetAttributes[focusHead, Flat];\n"
    for family in FAMILIES:
        for size in FOCUS_SIZES:
            rule, expected = {
                "increment": ("focusX_Integer :> focusX + 1", f"Range[2,{size + 1}]"),
                "increment-reversed": ("focusX_Integer :> 1 + focusX", f"Range[2,{size + 1}]"),
                "identity": ("focusX_Integer :> focusX", f"Range[{size}]"),
                "miss": ("focusMissing -> 0", f"Range[{size}]"),
                "opaque-body": ("focusX_Integer :> focusOpaque[focusX]", f"Table[focusOpaque[i],{{i,1,{size}}}]"),
                "flat-nested": ("focusX_Integer :> focusHead[focusHead[focusX],1]", f"Table[focusHead[i,1],{{i,1,{size}}}]"),
            }[family]
            source += f'''focusInput = Range[{size}]; focusAlias = focusInput;
focusExpected = {expected};
flatMeasure["focus-{family}-{size}", Replace[focusInput, {rule}, {{1}}], focusExpected];
If[SameQ[focusInput, Range[{size}]] && SameQ[focusAlias, Range[{size}]], Null, Exit[2]];
'''
    return source + 'Print["ISO_FOCUS_PASS"];\n'


def pipeline_source(timed=True):
    source = wrapper(timed)
    for label, size, passes, writes in PIPELINE_DEFINITIONS:
        rewrite = (f"Do[current = Replace[current, x_Integer :> x + 1, {{1}}], {{step,1,{passes}}}];"
                   if passes else "(* No rewrite in the matched control. *)")
        operation = f'''Module[{{source, sourceAlias, current, sorted, sortedAlias, i, step, x}},
  source = Reverse[Range[{size}]]; sourceAlias = source; current = source;
  {rewrite}
  sorted = Sort[current]; sortedAlias = sorted;
  Do[sorted[[i]] = -i, {{i,1,{writes}}}];
  {{sorted, sortedAlias, source, sourceAlias}}
]'''
        expected = (f"{{Join[Table[-i,{{i,1,{writes}}}],Range[{writes + passes + 1},{size + passes}]],"
                    f"Range[{passes + 1},{size + passes}],Reverse[Range[{size}]],Reverse[Range[{size}]]}}")
        source += f'pipelineExpected = {expected};\nflatMeasure["{label}", {operation}, pipelineExpected];\n'
    return source + 'Print["ISO_PIPELINE_PASS"];\n'


def semantic_source():
    """Explicit small values cover RHS order, nested Flat and COW behavior."""
    return '''(* No timing or resource-equivalence assertion in this preflight. *)
flatCheck[value_] := If[value, Null, Exit[2]];
flatInput = {3,1,2}; flatInputAlias = flatInput; flatLog = {};
flatResult = Replace[flatInput, flatX_Integer :> (AppendTo[flatLog,flatX]; flatX + 1), {1}];
flatCheck[SameQ[flatResult,{4,2,3}]];
flatCheck[SameQ[flatLog,{3,1,2}]];
flatCheck[SameQ[flatInput,{3,1,2}] && SameQ[flatInputAlias,{3,1,2}]];
flatResultAlias = flatResult; flatResult[[2]] = 99;
flatCheck[SameQ[flatResult,{4,99,3}] && SameQ[flatResultAlias,{4,2,3}]];
flatCheck[SameQ[Replace[flatInput,flatX_Integer :> 1 + flatX,{1}],{4,2,3}]];
flatCheck[SameQ[Replace[flatInput,flatX_Integer :> flatX,{1}],{3,1,2}]];
flatCheck[SameQ[Replace[flatInput,flatAbsent -> 0,{1}],{3,1,2}]];
SetAttributes[flatSemanticHead,Flat];
flatCheck[SameQ[Replace[flatInput,flatX_Integer :> flatSemanticHead[flatSemanticHead[flatX],1],{1}],
  {flatSemanticHead[3,1],flatSemanticHead[1,1],flatSemanticHead[2,1]}]];
SetAttributes[flatOrderHead,{Flat,Orderless}]; flatLog = {};
flatValue = flatOrderHead[(AppendTo[flatLog,3];3),(AppendTo[flatLog,1];flatOrderHead[1,2])];
flatCheck[SameQ[flatValue,flatOrderHead[1,2,3]] && SameQ[flatLog,{3,1}]];
flatNothing = Replace[{1,2,3},flatX_Integer :> If[SameQ[flatX,2],Nothing,flatX],{1}];
flatCheck[SameQ[flatNothing,{1,3}]];
flatAssociation = Association[1 -> g[2], 3 -> 4]; flatAssociationAlias = flatAssociation;
flatCheck[SameQ[Replace[flatAssociation,_Integer -> 9,{1,Infinity}],Association[1 -> g[9],3 -> 9]]];
flatCheck[SameQ[Replace[flatAssociation,_Integer -> 9,{-1}],Association[1 -> g[9],3 -> 9]]];
flatCheck[SameQ[flatAssociation,flatAssociationAlias] && SameQ[flatAssociation,Association[1 -> g[2],3 -> 4]]];
Print["ISO_SEMANTICS_PASS"];
'''


def sources(timed=True):
    return {
        "focus": (focus_source(timed), FOCUS_CASES, "ISO_FOCUS_PASS"),
        "pipeline": (pipeline_source(timed), PIPELINE_CASES, "ISO_PIPELINE_PASS"),
    }
