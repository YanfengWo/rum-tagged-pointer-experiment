Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=collect-app-spine-split"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumCollectN = 500;
rumCollectRules = rumPairProbeRules["basic"];
rumCollectPreparedRules = HOL`Auto`Simp`simpPrepareRule /@ rumCollectRules;
rumCollectProductiveRules =
  Select[
    rumCollectPreparedRules,
    HOL`Auto`Simp`Private`productiveSimpRule];
rumCollectRule = First[rumCollectProductiveRules];
rumCollectConcl = HOL`Kernel`concl[rumCollectRule];
rumCollectLhs = rumCollectConcl[[1, 2]];
rumCollectTarget = rumPairProbeXV;
rumCollectSpine = HOL`Drule`Private`collectAppSpine[rumCollectLhs];
rumCollectHead = rumCollectSpine[[1]];
rumCollectArgs = rumCollectSpine[[2]];
rumCollectFirstArg = rumCollectArgs[[1]];

Print["RUM_PAIR_PROBE collect-repeat-count=", rumCollectN];
Print["RUM_PAIR_PROBE collect-lhs-head=", Head[rumCollectLhs]];
Print["RUM_PAIR_PROBE collect-target-head=", Head[rumCollectTarget]];
Print["RUM_PAIR_PROBE collect-spine-arg-count=", Length[rumCollectArgs]];
Print["RUM_PAIR_PROBE collect-spine-head-head=", Head[rumCollectHead]];

SetAttributes[rumCollectRepeat, HoldAll];
rumCollectRepeat[label_, expr_] :=
  rumPairProbeMeasure[
    label,
    Module[{i = 0},
      While[i < rumCollectN,
        expr;
        i = i + 1]],
    Function[{value}, Null]];

rumCollectManual[] :=
  Module[{cur = rumCollectLhs, args = {}},
    While[MatchQ[cur, HOL`Terms`comb[_, _]],
      args = Prepend[args, cur[[2]]];
      cur = cur[[1]]];
    {cur, args}
  ];

rumCollectManualCountOnly[] :=
  Module[{cur = rumCollectLhs, count = 0},
    While[MatchQ[cur, HOL`Terms`comb[_, _]],
      count = count + 1;
      cur = cur[[1]]];
    count
  ];

rumCollectManualPartOnly[] :=
  Module[{cur = rumCollectLhs, left, right, count = 0},
    While[MatchQ[cur, HOL`Terms`comb[_, _]],
      right = cur[[2]];
      left = cur[[1]];
      count = count + 1;
      cur = left];
    {left, right, count}
  ];

rumCollectRepeat[
  "collect-full",
  HOL`Drule`Private`collectAppSpine[rumCollectLhs]];

rumCollectRepeat[
  "collect-manual",
  rumCollectManual[]];

rumCollectRepeat[
  "collect-manual-count-only",
  rumCollectManualCountOnly[]];

rumCollectRepeat[
  "collect-manual-part-only",
  rumCollectManualPartOnly[]];

rumCollectRepeat[
  "collect-MatchQ-comb-hit",
  MatchQ[rumCollectLhs, HOL`Terms`comb[_, _]]];

rumCollectRepeat[
  "collect-MatchQ-comb-miss-var",
  MatchQ[rumCollectTarget, HOL`Terms`comb[_, _]]];

rumCollectRepeat[
  "collect-Part-left",
  rumCollectLhs[[1]]];

rumCollectRepeat[
  "collect-Part-right",
  rumCollectLhs[[2]]];

rumCollectRepeat[
  "collect-Prepend-empty",
  Prepend[{}, rumCollectFirstArg]];

rumCollectRepeat[
  "collect-Prepend-one",
  Prepend[{rumCollectFirstArg}, rumCollectFirstArg]];

rumCollectRepeat[
  "collect-list-pair",
  {rumCollectHead, rumCollectArgs}];

rumCollectRepeat[
  "collect-loop-empty",
  Null];

Null
