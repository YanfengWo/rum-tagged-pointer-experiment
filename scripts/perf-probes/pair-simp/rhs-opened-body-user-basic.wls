Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-opened-body-user-basic"];
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

rumPairProbeOpenedBody =
  rumPairProbeOpenAbsN[rumPairProbeRhsLambda[], 4];
rumPairProbeUserBasicRules = rumPairProbeRules["userBasic"];
rumPairProbeMeasure[
  "rhs-opened-body-user-basic",
  rumPairProbeOpenedBodyThm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeOpenedBody],
  rumPairProbePrintThmResult[
    "rhs-opened-body-user-basic",
    rumPairProbeOpenedBody]];

Null
