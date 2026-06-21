Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-stability-fresh-module-only"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumFreshDirtyN = 200;
rumFreshDirtyRules = rumPairProbeRules["basic"];
rumFreshDirtyPreparedRules =
  HOL`Auto`Simp`simpPrepareRule /@ rumFreshDirtyRules;
rumFreshDirtyProductiveRules =
  Select[
    rumFreshDirtyPreparedRules,
    HOL`Auto`Simp`Private`productiveSimpRule];
rumFreshDirtyRule = First[rumFreshDirtyProductiveRules];
rumFreshDirtyConcl = HOL`Kernel`concl[rumFreshDirtyRule];
rumFreshDirtyStableTerm = rumFreshDirtyConcl[[1, 2]];
rumFreshDirtySpine =
  HOL`Drule`Private`collectAppSpine[rumFreshDirtyStableTerm];
rumFreshDirtyStableHead = rumFreshDirtySpine[[1]];
rumFreshDirtyStableArgs = rumFreshDirtySpine[[2]];

Print["RUM_PAIR_PROBE fresh-dirty-repeat-count=", rumFreshDirtyN];
Print["RUM_PAIR_PROBE fresh-dirty-spine-arg-count=",
  Length[rumFreshDirtyStableArgs]];

rumFreshDirtyWriteOnce[] :=
  Module[{cur, args},
    cur = rumFreshDirtyStableHead;
    args = rumFreshDirtyStableArgs;
    rumFreshDirtyStableTerm;
    {cur, args}
  ];

rumFreshDirtyRun[] :=
  Module[{i = 0},
    While[i < rumFreshDirtyN,
      rumFreshDirtyWriteOnce[];
      i = i + 1];
    Null
  ];

RumProfileReset[];
rumPairProbeMeasure[
  "fresh-dirty-module-writes",
  rumFreshDirtyRun[],
  Function[{value}, Null]];
RumProfileSnapshot["fresh-dirty-module-writes"];

Null
