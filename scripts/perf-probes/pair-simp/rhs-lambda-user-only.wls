Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-lambda-user-only"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-congruence",
  rumPairProbePreheatCongruence[],
  Function[{value}, Null]];

rumPairProbeRhsLambdaTerm = rumPairProbeRhsLambda[];
rumPairProbeUserOnlyRules = rumPairProbeRules["user"];
rumPairProbeMeasure[
  "rhs-lambda-descendOnce-user-only",
  rumPairProbeRhsLambdaUserOnlyThm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeRhsLambdaTerm],
  rumPairProbePrintThmResult[
    "rhs-lambda-descendOnce-user-only",
    rumPairProbeRhsLambdaTerm]];

Null
