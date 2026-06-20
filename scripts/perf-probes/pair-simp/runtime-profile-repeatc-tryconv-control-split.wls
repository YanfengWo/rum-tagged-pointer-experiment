Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-repeatc-tryconv-control-split"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumControlN = 80;
rumControlTarget = rumPairProbeXV;
rumControlRules = rumPairProbeRules["basic"];
rumControlPreparedRules =
  HOL`Auto`Simp`simpPrepareRule /@ rumControlRules;
rumControlProductiveRules =
  Select[
    rumControlPreparedRules,
    HOL`Auto`Simp`Private`productiveSimpRule];
rumControlProver =
  HOL`Auto`Simp`Private`makeProver[
    rumControlRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit];
rumControlRuleConvs =
  (HOL`Auto`Simp`Private`simpRuleConv[#, rumControlProver] &) /@
    rumControlProductiveRules;
rumControlBaseConv =
  HOL`Auto`Simp`Private`combineSimpConvs[rumControlRuleConvs];
rumControlRepeatC = HOL`Drule`REPEATC[rumControlBaseConv];
rumControlTryRepeatC = HOL`Drule`TRYCONV[rumControlRepeatC];
rumControlTryBase = HOL`Drule`TRYCONV[rumControlBaseConv];
rumControlFailed = HOL`Drule`Private`$convFailed;

Print["RUM_PAIR_PROBE runtime-control-repeat-count=", rumControlN];
Print["RUM_PAIR_PROBE runtime-control-rule-count=", Length[rumControlRules]];
Print["RUM_PAIR_PROBE runtime-control-productive-count=",
  Length[rumControlProductiveRules]];
Print["RUM_PAIR_PROBE runtime-control-baseConv-failed=",
  HOL`Drule`Private`tryConv[
    rumControlBaseConv[rumControlTarget]] === rumControlFailed];
Print["RUM_PAIR_PROBE runtime-control-repeatc-is-thm=",
  HOL`Kernel`isThm[rumControlRepeatC[rumControlTarget]]];
Print["RUM_PAIR_PROBE runtime-control-try-repeatc-is-thm=",
  HOL`Kernel`isThm[rumControlTryRepeatC[rumControlTarget]]];

SetAttributes[rumControlRepeatMeasure, HoldAll];
rumControlRepeatMeasure[label_, expr_, resultFn_] :=
  rumPairProbeMeasure[
    label,
    Module[{i = 0},
      While[i < rumControlN,
        expr;
        i = i + 1];
      expr],
    resultFn];

SetAttributes[rumControlProfileMeasure, HoldAll];
rumControlProfileMeasure[label_, expr_, resultFn_] := (
  RumProfileReset[];
  rumControlProfileResult =
    rumControlRepeatMeasure[label, expr, resultFn];
  RumProfileSnapshot[label];
  rumControlProfileResult
);

rumControlManualRepeatReturn[] :=
  Module[{res, rhs},
    res = HOL`Drule`Private`tryConv[
      rumControlBaseConv[rumControlTarget]];
    If[res === rumControlFailed,
      Return[HOL`Kernel`REFL[rumControlTarget]]];
    rhs = HOL`Kernel`concl[res][[2]];
    If[HOL`Terms`aconv[rhs, rumControlTarget], Return[res]];
    HOL`Kernel`TRANS[res, rumControlRepeatC[rhs]]
  ];

rumControlManualRepeatNoReturn[] :=
  Module[{res, rhs},
    res = HOL`Drule`Private`tryConv[
      rumControlBaseConv[rumControlTarget]];
    If[res === rumControlFailed,
      HOL`Kernel`REFL[rumControlTarget],
      rhs = HOL`Kernel`concl[res][[2]];
      If[HOL`Terms`aconv[rhs, rumControlTarget],
        res,
        HOL`Kernel`TRANS[res, rumControlRepeatC[rhs]]]]
  ];

rumControlManualTryRepeat[] :=
  Module[{res},
    res = HOL`Drule`Private`tryConv[
      rumControlRepeatC[rumControlTarget]];
    If[res === rumControlFailed,
      HOL`Kernel`REFL[rumControlTarget],
      res]
  ];

rumControlManualTryRepeatInline[] :=
  Module[{res},
    res = Catch[
      rumControlRepeatC[rumControlTarget],
      HOL`Error`holErrorTag];
    If[MatchQ[res, _Failure],
      HOL`Kernel`REFL[rumControlTarget],
      res]
  ];

rumControlProfileMeasure[
  "runtime-control-baseConv-tryConv-fail",
  HOL`Drule`Private`tryConv[
    rumControlBaseConv[rumControlTarget]],
  Function[{value}, Null]];

rumControlProfileMeasure[
  "runtime-control-TRYCONV-baseConv-fail-to-REFL",
  rumControlTryBase[rumControlTarget],
  rumPairProbePrintThmResult[
    "runtime-control-TRYCONV-baseConv-fail-to-REFL",
    rumControlTarget]];

rumControlProfileMeasure[
  "runtime-control-REPEATC-baseConv-fail-to-REFL",
  rumControlRepeatC[rumControlTarget],
  rumPairProbePrintThmResult[
    "runtime-control-REPEATC-baseConv-fail-to-REFL",
    rumControlTarget]];

rumControlProfileMeasure[
  "runtime-control-manual-REPEATC-return",
  rumControlManualRepeatReturn[],
  rumPairProbePrintThmResult[
    "runtime-control-manual-REPEATC-return",
    rumControlTarget]];

rumControlProfileMeasure[
  "runtime-control-manual-REPEATC-no-return",
  rumControlManualRepeatNoReturn[],
  rumPairProbePrintThmResult[
    "runtime-control-manual-REPEATC-no-return",
    rumControlTarget]];

rumControlProfileMeasure[
  "runtime-control-TRYCONV-REPEATC-success",
  rumControlTryRepeatC[rumControlTarget],
  rumPairProbePrintThmResult[
    "runtime-control-TRYCONV-REPEATC-success",
    rumControlTarget]];

rumControlProfileMeasure[
  "runtime-control-manual-TRYCONV-REPEATC-tryConv",
  rumControlManualTryRepeat[],
  rumPairProbePrintThmResult[
    "runtime-control-manual-TRYCONV-REPEATC-tryConv",
    rumControlTarget]];

rumControlProfileMeasure[
  "runtime-control-manual-TRYCONV-REPEATC-inline",
  rumControlManualTryRepeatInline[],
  rumPairProbePrintThmResult[
    "runtime-control-manual-TRYCONV-REPEATC-inline",
    rumControlTarget]];

rumControlProfileMeasure[
  "runtime-control-catch-REPEATC-success-only",
  Catch[rumControlRepeatC[rumControlTarget], HOL`Error`holErrorTag],
  rumPairProbePrintThmResult[
    "runtime-control-catch-REPEATC-success-only",
    rumControlTarget]];

Null
