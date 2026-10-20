"""Standalone neighbors and the Stage27 target, all with retained samples."""
CASES = ["control-set-part-400", "control-set-part-1600", "control-sort-8000",
         "control-sort-16000", "control-rewrite-miss-16000", "control-rewrite-hit-16000"]


def make_source():
    text = '''SetAttributes[isoMeasure, HoldAll];
isoMeasure[label_String, expression_, expected_] := Module[{i, result, seconds},
  result = expression;
  If[SameQ[result, expected], Null, Exit[2]];
  Do[
    seconds = First[AbsoluteTiming[result = expression]];
    Print[StringJoin["ISO_TIME\\t", label, "\\t", ToString[i], "\\t", ToString[seconds]]],
    {i, 1, 11}
  ];
  If[SameQ[result, expected], Null, Exit[2]];
];
'''
    for size in [400, 1600]:
        text += f'''isoPart = Table[0, {{{size}}}];
isoMeasure["control-set-part-{size}",
  (Do[isoPart[[i]] = i, {{i, 1, {size}}}]; isoPart), Range[{size}]];
'''
    for size in [8000, 16000]:
        text += f'''isoInput = Reverse[Range[{size}]]; isoAlias = isoInput;
isoMeasure["control-sort-{size}", Sort[isoInput], Range[{size}]];
If[SameQ[isoInput, Reverse[Range[{size}]]] && SameQ[isoInput, isoAlias], Null, Exit[2]];
'''
    text += '''isoInput = Range[16000]; isoAlias = isoInput;
isoMeasure["control-rewrite-miss-16000", Replace[isoInput, isoMissing -> 0, {1}], Range[16000]];
isoMeasure["control-rewrite-hit-16000", Replace[isoInput, isoX_Integer :> isoX + 1, {1}], Range[2, 16001]];
If[SameQ[isoInput, Range[16000]] && SameQ[isoInput, isoAlias], Null, Exit[2]];
Print["ISO_CONTROL_PASS"];
'''
    return text
