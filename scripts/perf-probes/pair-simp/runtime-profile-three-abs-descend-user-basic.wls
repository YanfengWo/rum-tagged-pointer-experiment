Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-three-abs-descend-user-basic"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];
rumPairProbeMeasure[
  "preheat-congruence",
  rumPairProbePreheatCongruence[],
  Function[{value}, Null]];

rumPairProbeUserBasicRules = rumPairProbeRules["userBasic"];
rumPairProbeThreeAbs =
  rumPairProbeOpenAbsN[rumPairProbeRhsLambda[], 1];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-target-three-abs-descendOnce",
  rumPairProbeProfileThreeAbsThm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeThreeAbs],
  rumPairProbePrintThmResult[
    "profile-target-three-abs-descendOnce",
    rumPairProbeThreeAbs]];

RumProfileSnapshot["three-abs-descendOnce-target"];

Null

