Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-stability-dirty-split"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumDirtyN = 200;
rumDirtyRules = rumPairProbeRules["basic"];
rumDirtyPreparedRules = HOL`Auto`Simp`simpPrepareRule /@ rumDirtyRules;
rumDirtyProductiveRules =
  Select[
    rumDirtyPreparedRules,
    HOL`Auto`Simp`Private`productiveSimpRule];
rumDirtyRule = First[rumDirtyProductiveRules];
rumDirtyConcl = HOL`Kernel`concl[rumDirtyRule];
rumDirtyStableTerm = rumDirtyConcl[[1, 2]];
rumDirtySpine = HOL`Drule`Private`collectAppSpine[rumDirtyStableTerm];
rumDirtyStableHead = rumDirtySpine[[1]];
rumDirtyStableArgs = rumDirtySpine[[2]];

Print["RUM_PAIR_PROBE dirty-repeat-count=", rumDirtyN];
Print["RUM_PAIR_PROBE dirty-term-head=", Head[rumDirtyStableTerm]];
Print["RUM_PAIR_PROBE dirty-spine-arg-count=", Length[rumDirtyStableArgs]];

SetAttributes[rumDirtyMeasure, HoldAll];
rumDirtyMeasure[label_, expr_] := (
  RumProfileReset[];
  rumDirtyResult =
    rumPairProbeMeasure[label, expr, Function[{value}, Null]];
  RumProfileSnapshot[label];
  rumDirtyResult
);

rumDirtyEvalTermIndexOnly[] :=
  Module[{i = 0},
    While[i < rumDirtyN,
      rumDirtyStableTerm;
      i = i + 1];
    Null
  ];

rumDirtyEvalHeadIndexOnly[] :=
  Module[{i = 0},
    While[i < rumDirtyN,
      rumDirtyStableHead;
      i = i + 1];
    Null
  ];

rumDirtyLiteralIndexOnly[] :=
  Module[{i = 0},
    While[i < rumDirtyN,
      1;
      i = i + 1];
    Null
  ];

rumDirtyEvalTermWithLocalWrites[] :=
  Module[{i = 0, cur, args},
    While[i < rumDirtyN,
      cur = rumDirtyStableHead;
      args = rumDirtyStableArgs;
      rumDirtyStableTerm;
      i = i + 1];
    {cur, args}
  ];

rumDirtyEvalTermWithGlobalWrites[] := (
  rumDirtyGlobalI = 0;
  While[rumDirtyGlobalI < rumDirtyN,
    rumDirtyGlobalCur = rumDirtyStableHead;
    rumDirtyGlobalArgs = rumDirtyStableArgs;
    rumDirtyStableTerm;
    rumDirtyGlobalI = rumDirtyGlobalI + 1];
  {rumDirtyGlobalCur, rumDirtyGlobalArgs}
);

rumDirtyFreshModuleWriteOnce[] :=
  Module[{cur, args},
    cur = rumDirtyStableHead;
    args = rumDirtyStableArgs;
    rumDirtyStableTerm;
    {cur, args}
  ];

rumDirtyEvalTermWithFreshModuleWrites[] :=
  Module[{i = 0},
    While[i < rumDirtyN,
      rumDirtyFreshModuleWriteOnce[];
      i = i + 1];
    Null
  ];

rumDirtyMeasure[
  "dirty-literal-index-only",
  rumDirtyLiteralIndexOnly[]];

rumDirtyMeasure[
  "dirty-head-index-only",
  rumDirtyEvalHeadIndexOnly[]];

rumDirtyMeasure[
  "dirty-term-index-only",
  rumDirtyEvalTermIndexOnly[]];

rumDirtyMeasure[
  "dirty-term-local-writes",
  rumDirtyEvalTermWithLocalWrites[]];

rumDirtyMeasure[
  "dirty-term-global-writes",
  rumDirtyEvalTermWithGlobalWrites[]];

rumDirtyMeasure[
  "dirty-term-fresh-module-writes",
  rumDirtyEvalTermWithFreshModuleWrites[]];

Null
