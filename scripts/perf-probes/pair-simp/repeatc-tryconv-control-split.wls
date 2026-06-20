Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=repeatc-tryconv-control-split"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumControlN = 200;
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
rumControlRefl = HOL`Kernel`REFL[rumControlTarget];
rumControlFailed = HOL`Drule`Private`$convFailed;

Print["RUM_PAIR_PROBE control-repeat-count=", rumControlN];
Print["RUM_PAIR_PROBE control-rule-count=", Length[rumControlRules]];
Print["RUM_PAIR_PROBE control-productive-count=",
  Length[rumControlProductiveRules]];
Print["RUM_PAIR_PROBE control-count-check=",
  Module[{i = 0, count = 0},
    While[i < rumControlN,
      count = count + 1;
      i = i + 1];
    count]];
Print["RUM_PAIR_PROBE control-baseConv-failed=",
  HOL`Drule`Private`tryConv[
    rumControlBaseConv[rumControlTarget]] === rumControlFailed];
Print["RUM_PAIR_PROBE control-repeatc-is-thm=",
  HOL`Kernel`isThm[rumControlRepeatC[rumControlTarget]]];
Print["RUM_PAIR_PROBE control-try-repeatc-is-thm=",
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

SetAttributes[rumControlRepeat, HoldAll];
rumControlRepeat[label_, expr_] :=
  rumControlRepeatMeasure[label, expr, Function[{value}, Null]];

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

rumControlManualTryPrebuiltSuccessInline[] :=
  Module[{res},
    res = Catch[rumControlRefl, HOL`Error`holErrorTag];
    If[MatchQ[res, _Failure],
      HOL`Kernel`REFL[rumControlTarget],
      res]
  ];

rumControlRepeat[
  "control-loop-empty",
  Null];

rumControlRepeat[
  "control-baseConv-tryConv-fail",
  HOL`Drule`Private`tryConv[
    rumControlBaseConv[rumControlTarget]]];

rumControlRepeatMeasure[
  "control-TRYCONV-baseConv-fail-to-REFL",
  rumControlTryBase[rumControlTarget],
  rumPairProbePrintThmResult[
    "control-TRYCONV-baseConv-fail-to-REFL",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-REPEATC-baseConv-fail-to-REFL",
  rumControlRepeatC[rumControlTarget],
  rumPairProbePrintThmResult[
    "control-REPEATC-baseConv-fail-to-REFL",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-manual-REPEATC-return",
  rumControlManualRepeatReturn[],
  rumPairProbePrintThmResult[
    "control-manual-REPEATC-return",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-manual-REPEATC-no-return",
  rumControlManualRepeatNoReturn[],
  rumPairProbePrintThmResult[
    "control-manual-REPEATC-no-return",
    rumControlTarget]];

rumControlRepeat[
  "control-failed-sameq-only",
  rumControlFailed === rumControlFailed];

rumControlRepeatMeasure[
  "control-REFL-only",
  HOL`Kernel`REFL[rumControlTarget],
  rumPairProbePrintThmResult[
    "control-REFL-only",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-prebuilt-REFL-return",
  rumControlRefl,
  rumPairProbePrintThmResult[
    "control-prebuilt-REFL-return",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-TRYCONV-REPEATC-success",
  rumControlTryRepeatC[rumControlTarget],
  rumPairProbePrintThmResult[
    "control-TRYCONV-REPEATC-success",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-manual-TRYCONV-REPEATC-tryConv",
  rumControlManualTryRepeat[],
  rumPairProbePrintThmResult[
    "control-manual-TRYCONV-REPEATC-tryConv",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-manual-TRYCONV-REPEATC-inline",
  rumControlManualTryRepeatInline[],
  rumPairProbePrintThmResult[
    "control-manual-TRYCONV-REPEATC-inline",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-catch-REPEATC-success-only",
  Catch[rumControlRepeatC[rumControlTarget], HOL`Error`holErrorTag],
  rumPairProbePrintThmResult[
    "control-catch-REPEATC-success-only",
    rumControlTarget]];

rumControlRepeat[
  "control-MatchQ-success-thm",
  MatchQ[rumControlRefl, _Failure]];

rumControlRepeatMeasure[
  "control-If-MatchQ-success-thm",
  If[MatchQ[rumControlRefl, _Failure],
    HOL`Kernel`REFL[rumControlTarget],
    rumControlRefl],
  rumPairProbePrintThmResult[
    "control-If-MatchQ-success-thm",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-catch-prebuilt-success-thm",
  Catch[rumControlRefl, HOL`Error`holErrorTag],
  rumPairProbePrintThmResult[
    "control-catch-prebuilt-success-thm",
    rumControlTarget]];

rumControlRepeatMeasure[
  "control-manual-TRYCONV-prebuilt-success-inline",
  rumControlManualTryPrebuiltSuccessInline[],
  rumPairProbePrintThmResult[
    "control-manual-TRYCONV-prebuilt-success-inline",
    rumControlTarget]];

Null
