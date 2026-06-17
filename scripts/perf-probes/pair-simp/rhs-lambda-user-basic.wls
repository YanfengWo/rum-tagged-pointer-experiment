Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-lambda-user-basic"];
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

rumPairProbeRhsLambdaTerm = rumPairProbeRhsLambda[];
rumPairProbeUserBasicRules = rumPairProbeRules["userBasic"];
rumPairProbeMeasure[
  "rhs-lambda-descendOnce-user-basic",
  rumPairProbeRhsLambdaUserBasicThm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeRhsLambdaTerm],
  rumPairProbePrintThmResult[
    "rhs-lambda-descendOnce-user-basic",
    rumPairProbeRhsLambdaTerm]];

Null
