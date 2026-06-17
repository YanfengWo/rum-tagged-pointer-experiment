Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-outer-abs-inner-user-basic"];
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
rumPairProbeOuterOpened =
  rumPairProbeOpenAbsN[rumPairProbeRhsLambda[], 1];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-target-outer-abs-inner-simpFixpointConv",
  rumPairProbeProfileOuterInnerThm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeOuterOpened],
  rumPairProbePrintThmResult[
    "profile-target-outer-abs-inner-simpFixpointConv",
    rumPairProbeOuterOpened]];

RumProfileSnapshot["outer-abs-inner-simpFixpointConv-target"];

Null

