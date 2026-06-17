Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-opened-body-user-basic"];
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

RumProfileReset[];

rumPairProbeUserBasicRules = rumPairProbeRules["userBasic"];
rumPairProbeOpenedBody =
  rumPairProbeOpenAbsN[rumPairProbeRhsLambda[], 4];

rumPairProbeMeasure[
  "profile-target-opened-body-simpFixpointConv",
  rumPairProbeProfileOpenedBodyThm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeOpenedBody],
  rumPairProbePrintThmResult[
    "profile-target-opened-body-simpFixpointConv",
    rumPairProbeOpenedBody]];

RumProfileSnapshot["opened-body-user-basic-target"];

Null

