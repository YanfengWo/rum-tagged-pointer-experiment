Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-lambda-basic-only"];
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
rumPairProbeBasicOnlyRules = rumPairProbeRules["basic"];
rumPairProbeMeasure[
  "rhs-lambda-descendOnce-basic-only",
  rumPairProbeRhsLambdaBasicOnlyThm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeRhsLambdaTerm],
  rumPairProbePrintThmResult[
    "rhs-lambda-descendOnce-basic-only",
    rumPairProbeRhsLambdaTerm]];

Null
