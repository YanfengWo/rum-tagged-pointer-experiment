Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-pair4-prebuilt-step-repeat8"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbePrebuiltBaseConv =
  HOL`Auto`Simp`Private`buildBaseConv[
    rumPairProbeBasicRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit];
rumPairProbePrebuiltStepConv =
  HOL`Auto`Simp`Private`simpStepConv[
    rumPairProbePrebuiltBaseConv, rumPairProbeBasicRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit];

rumPairProbePrebuiltFixpoint[stepConv_][t_] :=
  Module[{cur, rhs, next, newRhs, seen},
    cur = stepConv[t];
    rhs = HOL`Kernel`concl[cur][[2]];
    seen = <|t -> True, rhs -> True|>;
    While[True,
      next = stepConv[rhs];
      newRhs = HOL`Kernel`concl[next][[2]];
      If[newRhs === rhs, Break[]];
      If[KeyExistsQ[seen, newRhs], Break[]];
      cur = HOL`Kernel`TRANS[cur, next];
      rhs = newRhs;
      seen[newRhs] = True
    ];
    cur
  ];

RumProfileReset[];
rumPairProbeMeasure[
  "pair4-prebuilt-step-repeat8",
  Do[
    rumPairProbePrebuiltStepThm =
      rumPairProbePrebuiltFixpoint[
        rumPairProbePrebuiltStepConv][rumPairProbePair4Same],
    {8}],
  Function[{value},
    Print["RUM_PAIR_PROBE pair4-prebuilt-step-repeat8 repeats=", 8];
    Print["RUM_PAIR_PROBE pair4-prebuilt-step-repeat8 is_thm=",
      HOL`Kernel`isThm[rumPairProbePrebuiltStepThm]];
    Print["RUM_PAIR_PROBE pair4-prebuilt-step-repeat8 changed=",
      HOL`Kernel`concl[rumPairProbePrebuiltStepThm][[2]] =!=
        rumPairProbePair4Same];
    Null]];
RumProfileSnapshot["pair4-prebuilt-step-repeat8-target"];

Null
