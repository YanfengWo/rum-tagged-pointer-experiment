Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=basic-chain-orelsec-split"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumBasicChainN = 200;
rumBasicChainTarget = rumPairProbeXV;
rumBasicChainRules = rumPairProbeRules["basic"];
rumBasicChainPreparedRules =
  HOL`Auto`Simp`simpPrepareRule /@ rumBasicChainRules;
rumBasicChainProductiveRules =
  Select[
    rumBasicChainPreparedRules,
    HOL`Auto`Simp`Private`productiveSimpRule];
rumBasicChainProver =
  HOL`Auto`Simp`Private`makeProver[
    rumBasicChainRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit];
rumBasicChainRuleConvs =
  (HOL`Auto`Simp`Private`simpRuleConv[#, rumBasicChainProver] &) /@
    rumBasicChainProductiveRules;

rumBasicChain1 =
  HOL`Auto`Simp`Private`combineSimpConvs[
    Take[rumBasicChainRuleConvs, 1]];
rumBasicChain5 =
  HOL`Auto`Simp`Private`combineSimpConvs[
    Take[rumBasicChainRuleConvs, 5]];
rumBasicChain10 =
  HOL`Auto`Simp`Private`combineSimpConvs[
    Take[rumBasicChainRuleConvs, 10]];
rumBasicChain19 =
  HOL`Auto`Simp`Private`combineSimpConvs[
    Take[rumBasicChainRuleConvs, 19]];

rumBasicChainRepeat19 = HOL`Drule`REPEATC[rumBasicChain19];
rumBasicChainTryRepeat19 = HOL`Drule`TRYCONV[rumBasicChainRepeat19];
rumBasicChainTryBase19 = HOL`Drule`TRYCONV[rumBasicChain19];

Print["RUM_PAIR_PROBE basic-chain-repeat-count=", rumBasicChainN];
Print["RUM_PAIR_PROBE basic-chain-rule-count=", Length[rumBasicChainRules]];
Print["RUM_PAIR_PROBE basic-chain-productive-count=",
  Length[rumBasicChainProductiveRules]];
Print["RUM_PAIR_PROBE basic-chain-target-head=", Head[rumBasicChainTarget]];
Print["RUM_PAIR_PROBE basic-chain-count-check=",
  Module[{i = 0, count = 0},
    While[i < rumBasicChainN,
      count = count + 1;
      i = i + 1];
    count]];
Print["RUM_PAIR_PROBE basic-chain-raw-failed=",
  HOL`Drule`Private`tryConv[
    rumBasicChain19[rumBasicChainTarget]] ===
    HOL`Drule`Private`$convFailed];
Print["RUM_PAIR_PROBE basic-chain-full-is-thm=",
  HOL`Kernel`isThm[rumBasicChainTryRepeat19[rumBasicChainTarget]]];

SetAttributes[rumBasicChainRepeatMeasure, HoldAll];
rumBasicChainRepeatMeasure[label_, expr_, resultFn_] :=
  rumPairProbeMeasure[
    label,
    Module[{i = 0},
      While[i < rumBasicChainN,
        expr;
        i = i + 1];
      expr],
    resultFn];

SetAttributes[rumBasicChainRepeat, HoldAll];
rumBasicChainRepeat[label_, expr_] :=
  rumBasicChainRepeatMeasure[label, expr, Function[{value}, Null]];

rumBasicChainManualLoop[includeBeta_] :=
  Module[{j = 1, res = HOL`Drule`Private`$convFailed,
          failures = 0},
    While[j <= Length[rumBasicChainRuleConvs] &&
        res === HOL`Drule`Private`$convFailed,
      res = HOL`Drule`Private`tryConv[
        rumBasicChainRuleConvs[[j]][rumBasicChainTarget]];
      If[res === HOL`Drule`Private`$convFailed,
        failures = failures + 1];
      j = j + 1];
    If[res === HOL`Drule`Private`$convFailed && includeBeta,
      res = HOL`Drule`Private`tryConv[
        HOL`Equal`BETACONV[rumBasicChainTarget]];
      If[res === HOL`Drule`Private`$convFailed,
        failures = failures + 1]];
    {res, failures}
  ];

rumBasicChainInlineTryConv[conv_] :=
  Module[{res},
    res = Catch[conv[rumBasicChainTarget], HOL`Error`holErrorTag];
    If[MatchQ[res, _Failure], HOL`Drule`Private`$convFailed, res]
  ];

rumBasicChainRepeat[
  "basic-chain-loop-empty",
  Null];

rumBasicChainRepeatMeasure[
  "basic-chain-fresh-TRYCONV-REPEATC-depth19",
  HOL`Drule`TRYCONV[
    HOL`Drule`REPEATC[rumBasicChain19]][rumBasicChainTarget],
  rumPairProbePrintThmResult[
    "basic-chain-fresh-TRYCONV-REPEATC-depth19",
    rumBasicChainTarget]];

rumBasicChainRepeatMeasure[
  "basic-chain-prebuilt-TRYCONV-REPEATC-depth19",
  rumBasicChainTryRepeat19[rumBasicChainTarget],
  rumPairProbePrintThmResult[
    "basic-chain-prebuilt-TRYCONV-REPEATC-depth19",
    rumBasicChainTarget]];

rumBasicChainRepeatMeasure[
  "basic-chain-prebuilt-REPEATC-depth19",
  rumBasicChainRepeat19[rumBasicChainTarget],
  rumPairProbePrintThmResult[
    "basic-chain-prebuilt-REPEATC-depth19",
    rumBasicChainTarget]];

rumBasicChainRepeatMeasure[
  "basic-chain-TRYCONV-baseConv-depth19",
  rumBasicChainTryBase19[rumBasicChainTarget],
  rumPairProbePrintThmResult[
    "basic-chain-TRYCONV-baseConv-depth19",
    rumBasicChainTarget]];

rumBasicChainRepeat[
  "basic-chain-raw-baseConv-catch-depth19",
  Catch[rumBasicChain19[rumBasicChainTarget], HOL`Error`holErrorTag]];

rumBasicChainRepeat[
  "basic-chain-raw-baseConv-tryConv-depth19",
  HOL`Drule`Private`tryConv[
    rumBasicChain19[rumBasicChainTarget]]];

rumBasicChainRepeat[
  "basic-chain-raw-baseConv-inline-tryConv-depth19",
  rumBasicChainInlineTryConv[rumBasicChain19]];

rumBasicChainRepeatMeasure[
  "basic-chain-manual-rule-loop-tryConv",
  rumBasicChainManualLoop[False],
  Function[{value},
    Print["RUM_PAIR_PROBE basic-chain-manual-loop-failures=",
      value[[2]]];
    Print["RUM_PAIR_PROBE basic-chain-manual-loop-failed=",
      value[[1]] === HOL`Drule`Private`$convFailed];
    Null]];

rumBasicChainRepeatMeasure[
  "basic-chain-manual-rule-loop-plus-beta",
  rumBasicChainManualLoop[True],
  Function[{value},
    Print["RUM_PAIR_PROBE basic-chain-manual-plus-beta-failures=",
      value[[2]]];
    Print["RUM_PAIR_PROBE basic-chain-manual-plus-beta-failed=",
      value[[1]] === HOL`Drule`Private`$convFailed];
    Null]];

rumBasicChainRepeat[
  "basic-chain-raw-tryConv-depth1",
  HOL`Drule`Private`tryConv[
    rumBasicChain1[rumBasicChainTarget]]];

rumBasicChainRepeat[
  "basic-chain-raw-tryConv-depth5",
  HOL`Drule`Private`tryConv[
    rumBasicChain5[rumBasicChainTarget]]];

rumBasicChainRepeat[
  "basic-chain-raw-tryConv-depth10",
  HOL`Drule`Private`tryConv[
    rumBasicChain10[rumBasicChainTarget]]];

rumBasicChainRepeat[
  "basic-chain-raw-tryConv-depth19",
  HOL`Drule`Private`tryConv[
    rumBasicChain19[rumBasicChainTarget]]];

rumBasicChainRepeatMeasure[
  "basic-chain-full-TRYCONV-REPEATC-depth1",
  HOL`Drule`TRYCONV[
    HOL`Drule`REPEATC[rumBasicChain1]][rumBasicChainTarget],
  rumPairProbePrintThmResult[
    "basic-chain-full-TRYCONV-REPEATC-depth1",
    rumBasicChainTarget]];

rumBasicChainRepeatMeasure[
  "basic-chain-full-TRYCONV-REPEATC-depth5",
  HOL`Drule`TRYCONV[
    HOL`Drule`REPEATC[rumBasicChain5]][rumBasicChainTarget],
  rumPairProbePrintThmResult[
    "basic-chain-full-TRYCONV-REPEATC-depth5",
    rumBasicChainTarget]];

rumBasicChainRepeatMeasure[
  "basic-chain-full-TRYCONV-REPEATC-depth10",
  HOL`Drule`TRYCONV[
    HOL`Drule`REPEATC[rumBasicChain10]][rumBasicChainTarget],
  rumPairProbePrintThmResult[
    "basic-chain-full-TRYCONV-REPEATC-depth10",
    rumBasicChainTarget]];

rumBasicChainRepeatMeasure[
  "basic-chain-full-TRYCONV-REPEATC-depth19",
  HOL`Drule`TRYCONV[
    HOL`Drule`REPEATC[rumBasicChain19]][rumBasicChainTarget],
  rumPairProbePrintThmResult[
    "basic-chain-full-TRYCONV-REPEATC-depth19",
    rumBasicChainTarget]];

Null
