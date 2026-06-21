Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-loop-control-split"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumLoopProfileN = 500;
rumLoopProfileRules = rumPairProbeRules["basic"];
rumLoopProfilePreparedRules =
  HOL`Auto`Simp`simpPrepareRule /@ rumLoopProfileRules;
rumLoopProfileProductiveRules =
  Select[
    rumLoopProfilePreparedRules,
    HOL`Auto`Simp`Private`productiveSimpRule];
rumLoopProfileRule = First[rumLoopProfileProductiveRules];
rumLoopProfileConcl = HOL`Kernel`concl[rumLoopProfileRule];
rumLoopProfileLhs = rumLoopProfileConcl[[1, 2]];
rumLoopProfileSpine = HOL`Drule`Private`collectAppSpine[rumLoopProfileLhs];
rumLoopProfileHead = rumLoopProfileSpine[[1]];
rumLoopProfileArgs = rumLoopProfileSpine[[2]];
rumLoopProfileDepth = Length[rumLoopProfileArgs];
rumLoopProfileFirstArg = rumLoopProfileArgs[[1]];
rumLoopProfileTag = HOL`Error`holErrorTag;

Print["RUM_PAIR_PROBE loop-repeat-count=", rumLoopProfileN];
Print["RUM_PAIR_PROBE loop-lhs-head=", Head[rumLoopProfileLhs]];
Print["RUM_PAIR_PROBE loop-spine-arg-count=", rumLoopProfileDepth];

SetAttributes[rumLoopProfileRepeat, HoldAll];
rumLoopProfileRepeat[label_, expr_] :=
  rumPairProbeMeasure[
    label,
    Module[{i = 0},
      While[i < rumLoopProfileN,
        expr;
        i = i + 1]],
    Function[{value}, Null]];

SetAttributes[rumLoopProfileMeasure, HoldAll];
rumLoopProfileMeasure[label_, expr_] := (
  RumProfileReset[];
  rumLoopProfileResult = rumLoopProfileRepeat[label, expr];
  RumProfileSnapshot[label];
  rumLoopProfileResult
);

rumLoopCollectAppSpine[] :=
  HOL`Drule`Private`collectAppSpine[rumLoopProfileLhs];

rumLoopManualCollect[] :=
  Module[{cur, args},
    cur = rumLoopProfileLhs;
    args = {};
    While[MatchQ[cur, HOL`Terms`comb[_, _]],
      args = Prepend[args, cur[[2]]];
      cur = cur[[1]]];
    {cur, args}
  ];

rumLoopModuleInitOnly[] :=
  Module[{cur, args},
    cur = rumLoopProfileLhs;
    args = {};
    {cur, args}
  ];

rumLoopModuleCounterOnly[] :=
  Module[{i = 0},
    While[i < rumLoopProfileDepth,
      i = i + 1];
    i
  ];

rumLoopModuleTwoSetOnly[] :=
  Module[{cur, args, i = 0},
    cur = rumLoopProfileLhs;
    args = {};
    While[i < rumLoopProfileDepth,
      cur = rumLoopProfileHead;
      args = rumLoopProfileArgs;
      i = i + 1];
    {cur, args}
  ];

rumLoopModuleMatchOnly[] :=
  Module[{cur, matched, i = 0},
    cur = rumLoopProfileLhs;
    While[i < rumLoopProfileDepth,
      matched = MatchQ[cur, HOL`Terms`comb[_, _]];
      i = i + 1];
    matched
  ];

rumLoopModulePartSetOnly[] :=
  Module[{cur, left, right, i = 0},
    cur = rumLoopProfileLhs;
    While[i < rumLoopProfileDepth,
      right = cur[[2]];
      left = cur[[1]];
      cur = left;
      i = i + 1];
    {cur, left, right}
  ];

rumLoopGlobalCollect[] := (
  rumLoopGlobalCur = rumLoopProfileLhs;
  rumLoopGlobalArgs = {};
  While[MatchQ[rumLoopGlobalCur, HOL`Terms`comb[_, _]],
    rumLoopGlobalArgs = Prepend[rumLoopGlobalArgs, rumLoopGlobalCur[[2]]];
    rumLoopGlobalCur = rumLoopGlobalCur[[1]]];
  {rumLoopGlobalCur, rumLoopGlobalArgs}
);

rumLoopCatchSuccess[] :=
  Catch[rumLoopProfileFirstArg, rumLoopProfileTag];

rumLoopCatchThrow[] :=
  Catch[Throw[rumLoopProfileFirstArg, rumLoopProfileTag], rumLoopProfileTag];

rumLoopCatchConvFail[] :=
  Catch[
    HOL`Drule`Private`convFail[
      "runtime-profile-loop-control-split",
      <|"target" -> rumLoopProfileLhs|>],
    rumLoopProfileTag];

rumLoopProfileMeasure[
  "loop-collectAppSpine",
  rumLoopCollectAppSpine[]];

rumLoopProfileMeasure[
  "loop-manual-collect",
  rumLoopManualCollect[]];

rumLoopProfileMeasure[
  "loop-module-init-only",
  rumLoopModuleInitOnly[]];

rumLoopProfileMeasure[
  "loop-module-counter-only",
  rumLoopModuleCounterOnly[]];

rumLoopProfileMeasure[
  "loop-module-two-set-only",
  rumLoopModuleTwoSetOnly[]];

rumLoopProfileMeasure[
  "loop-module-match-only",
  rumLoopModuleMatchOnly[]];

rumLoopProfileMeasure[
  "loop-module-part-set-only",
  rumLoopModulePartSetOnly[]];

rumLoopProfileMeasure[
  "loop-global-collect",
  rumLoopGlobalCollect[]];

rumLoopProfileMeasure[
  "loop-catch-success",
  rumLoopCatchSuccess[]];

rumLoopProfileMeasure[
  "loop-catch-throw",
  rumLoopCatchThrow[]];

rumLoopProfileMeasure[
  "loop-catch-convFail",
  rumLoopCatchConvFail[]];

Null
