Get[FileNameJoin[{DirectoryName[DirectoryName[$InputFileName]], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=wk-conversion-wrapper-profile"];
Print["RUM_PAIR_PROBE engine=wolfram-only"];

rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumWKProfileN = 200;
rumWKPairRepeats = 2;
rumWKTarget = rumPairProbeXV;
rumWKRules = rumPairProbeRules["basic"];
rumWKPreparedRules = HOL`Auto`Simp`simpPrepareRule /@ rumWKRules;
rumWKProductiveRules =
  Select[rumWKPreparedRules, HOL`Auto`Simp`Private`productiveSimpRule];
rumWKProver =
  HOL`Auto`Simp`Private`makeProver[
    rumWKRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit];
rumWKRuleConvs =
  (HOL`Auto`Simp`Private`simpRuleConv[#, rumWKProver] &) /@
    rumWKProductiveRules;
rumWKBaseConv =
  HOL`Auto`Simp`Private`combineSimpConvs[rumWKRuleConvs];
rumWKRepeatC = HOL`Drule`REPEATC[rumWKBaseConv];
rumWKTryRepeatC = HOL`Drule`TRYCONV[rumWKRepeatC];

Print["RUM_PAIR_PROBE wk-profile-repeat-count=", rumWKProfileN];
Print["RUM_PAIR_PROBE wk-profile-pair-repeats=", rumWKPairRepeats];
Print["RUM_PAIR_PROBE wk-profile-rule-count=", Length[rumWKRules]];
Print["RUM_PAIR_PROBE wk-profile-productive-count=", Length[rumWKProductiveRules]];
Print["RUM_PAIR_PROBE wk-profile-baseConv-failed=",
  HOL`Drule`Private`tryConv[rumWKBaseConv[rumWKTarget]] ===
    HOL`Drule`Private`$convFailed];
Print["RUM_PAIR_PROBE wk-profile-try-repeatc-is-thm=",
  HOL`Kernel`isThm[rumWKTryRepeatC[rumWKTarget]]];

SetAttributes[rumWKRepeatMeasure, HoldAll];
rumWKRepeatMeasure[label_, n_, expr_, resultFn_] :=
  rumPairProbeMeasure[
    label,
    Module[{i = 0},
      While[i < n,
        expr;
        i = i + 1];
      expr],
    resultFn];

SetAttributes[rumWKRepeat, HoldAll];
rumWKRepeat[label_, n_, expr_] :=
  rumWKRepeatMeasure[label, n, expr, Function[{value}, Null]];

rumWKRepeat[
  "wk-profile-baseline-baseConv-tryConv-depth19",
  rumWKProfileN,
  HOL`Drule`Private`tryConv[rumWKBaseConv[rumWKTarget]]];

rumWKRepeatMeasure[
  "wk-profile-baseline-TRYCONV-REPEATC-depth19",
  rumWKProfileN,
  rumWKTryRepeatC[rumWKTarget],
  rumPairProbePrintThmResult[
    "wk-profile-baseline-TRYCONV-REPEATC-depth19",
    rumWKTarget]];

rumWKCloneSymbol[from_Symbol, to_Symbol] := (
  ClearAll[to];
  Attributes[to] = Attributes[from];
  DownValues[to] = DownValues[from] /. from -> to;
  OwnValues[to] = OwnValues[from] /. from -> to;
  SubValues[to] = SubValues[from] /. from -> to;
);

rumWKStats = <||>;

rumWKResetStats[] := (rumWKStats = <||>;);

rumWKRecord[label_String, seconds_Real, outcome_String] := Module[{},
  If[!KeyExistsQ[rumWKStats, label],
    rumWKStats[label] =
      <|"count" -> 0, "seconds" -> 0., "ok" -> 0, "fail" -> 0,
        "fallback" -> 0, "first" -> 0, "failToRefl" -> 0,
        "stable" -> 0, "recurse" -> 0, "other" -> 0|>];
  rumWKStats[label, "count"] = rumWKStats[label, "count"] + 1;
  rumWKStats[label, "seconds"] = rumWKStats[label, "seconds"] + seconds;
  If[KeyExistsQ[rumWKStats[label], outcome],
    rumWKStats[label, outcome] = rumWKStats[label, outcome] + 1,
    rumWKStats[label, "other"] = rumWKStats[label, "other"] + 1]];

rumWKPrintStats[tag_String] := Module[{keys, entry, count},
  keys = Sort[Keys[rumWKStats]];
  Do[
    entry = rumWKStats[key];
    count = entry["count"];
    Print[
      "RUM_PAIR_PROBE wkstat tag=", tag,
      " label=", key,
      " count=", count,
      " seconds=", N[entry["seconds"], 9],
      " avg_us=", N[1000000. entry["seconds"] / Max[count, 1], 6],
      " ok=", entry["ok"],
      " fail=", entry["fail"],
      " fallback=", entry["fallback"],
      " first=", entry["first"],
      " failToRefl=", entry["failToRefl"],
      " stable=", entry["stable"],
      " recurse=", entry["recurse"],
      " other=", entry["other"]];
    , {key, keys}]
];

rumWKCloneSymbol[HOL`Drule`REWRCONV, rumWKOrigREWRCONV];
rumWKCloneSymbol[HOL`Drule`Private`tryTermMatch, rumWKOrigTryTermMatch];
rumWKCloneSymbol[HOL`Auto`Simp`Private`simpRuleConv, rumWKOrigSimpRuleConv];

ClearAll[HOL`Drule`Private`tryConv];
SetAttributes[HOL`Drule`Private`tryConv, HoldFirst];
HOL`Drule`Private`tryConv[body_] :=
  Module[{start, res, seconds},
    start = AbsoluteTime[];
    res = Catch[body, HOL`Error`holErrorTag];
    If[MatchQ[res, _Failure],
      res = HOL`Drule`Private`$convFailed];
    seconds = AbsoluteTime[] - start;
    rumWKRecord["tryConv", seconds,
      If[res === HOL`Drule`Private`$convFailed, "fail", "ok"]];
    res
  ];

ClearAll[HOL`Drule`ORELSEC];
HOL`Drule`ORELSEC[c1_, c2_][t_] :=
  Module[{start, res, out, seconds, outcome},
    start = AbsoluteTime[];
    res = HOL`Drule`Private`tryConv[c1[t]];
    If[res === HOL`Drule`Private`$convFailed,
      out = c2[t]; outcome = "fallback",
      out = res; outcome = "first"];
    seconds = AbsoluteTime[] - start;
    rumWKRecord["ORELSEC", seconds, outcome];
    out
  ];

ClearAll[HOL`Drule`TRYCONV];
HOL`Drule`TRYCONV[c_][t_] :=
  Module[{start, res, out, seconds, outcome},
    start = AbsoluteTime[];
    res = HOL`Drule`Private`tryConv[c[t]];
    If[res === HOL`Drule`Private`$convFailed,
      out = HOL`Kernel`REFL[t]; outcome = "failToRefl",
      out = res; outcome = "ok"];
    seconds = AbsoluteTime[] - start;
    rumWKRecord["TRYCONV", seconds, outcome];
    out
  ];

ClearAll[HOL`Drule`REPEATC];
HOL`Drule`REPEATC[c_][t_] :=
  Module[{start, res, rhs, out, seconds, outcome},
    start = AbsoluteTime[];
    res = HOL`Drule`Private`tryConv[c[t]];
    If[res === HOL`Drule`Private`$convFailed,
      out = HOL`Kernel`REFL[t]; outcome = "failToRefl",
      rhs = HOL`Kernel`concl[res][[2]];
      If[HOL`Terms`aconv[rhs, t],
        out = res; outcome = "stable",
        out = HOL`Kernel`TRANS[res, HOL`Drule`REPEATC[c][rhs]];
        outcome = "recurse"]];
    seconds = AbsoluteTime[] - start;
    rumWKRecord["REPEATC", seconds, outcome];
    out
  ];

ClearAll[HOL`Drule`REWRCONV];
HOL`Drule`REWRCONV[eqTh_][t_] :=
  Module[{start, res, seconds},
    start = AbsoluteTime[];
    res = Catch[rumWKOrigREWRCONV[eqTh][t], HOL`Error`holErrorTag];
    seconds = AbsoluteTime[] - start;
    If[MatchQ[res, _Failure],
      rumWKRecord["REWRCONV", seconds, "fail"];
      Throw[res, HOL`Error`holErrorTag],
      rumWKRecord["REWRCONV", seconds, "ok"];
      res]
  ];

ClearAll[HOL`Drule`Private`tryTermMatch];
HOL`Drule`Private`tryTermMatch[args___] :=
  Module[{start, res, seconds},
    start = AbsoluteTime[];
    res = rumWKOrigTryTermMatch[args];
    seconds = AbsoluteTime[] - start;
    rumWKRecord["tryTermMatch", seconds,
      If[res === HOL`Drule`Private`$matchFail, "fail", "ok"]];
    res
  ];

ClearAll[HOL`Auto`Simp`Private`simpRuleConv];
HOL`Auto`Simp`Private`simpRuleConv[ruleTh_, prover_][t_] :=
  Module[{start, res, seconds},
    start = AbsoluteTime[];
    res = Catch[rumWKOrigSimpRuleConv[ruleTh, prover][t],
      HOL`Error`holErrorTag];
    seconds = AbsoluteTime[] - start;
    If[MatchQ[res, _Failure],
      rumWKRecord["simpRuleConv", seconds, "fail"];
      Throw[res, HOL`Error`holErrorTag],
      rumWKRecord["simpRuleConv", seconds, "ok"];
      res]
  ];

rumWKResetStats[];
rumWKRepeat[
  "wk-profile-instrumented-baseConv-tryConv-depth19",
  rumWKProfileN,
  HOL`Drule`Private`tryConv[rumWKBaseConv[rumWKTarget]]];
rumWKPrintStats["baseConv-tryConv-depth19"];

rumWKResetStats[];
rumWKRepeatMeasure[
  "wk-profile-instrumented-TRYCONV-REPEATC-depth19",
  rumWKProfileN,
  rumWKTryRepeatC[rumWKTarget],
  rumPairProbePrintThmResult[
    "wk-profile-instrumented-TRYCONV-REPEATC-depth19",
    rumWKTarget]];
rumWKPrintStats["TRYCONV-REPEATC-depth19"];

rumWKResetStats[];
rumWKRepeatMeasure[
  "wk-profile-instrumented-pair4-stock-repeat2",
  rumWKPairRepeats,
  HOL`Auto`Simp`Private`simpFixpointConv[
    rumPairProbeBasicRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4Same],
  Function[{value},
    Print["RUM_PAIR_PROBE wk-profile-pair4-stock-repeat2 repeats=",
      rumWKPairRepeats];
    Print["RUM_PAIR_PROBE wk-profile-pair4-stock-repeat2 is_thm=",
      HOL`Kernel`isThm[value]];
    Print["RUM_PAIR_PROBE wk-profile-pair4-stock-repeat2 changed=",
      HOL`Kernel`concl[value][[2]] =!= rumPairProbePair4Same];
    Null]];
rumWKPrintStats["pair4-stock-repeat2"];

Null
