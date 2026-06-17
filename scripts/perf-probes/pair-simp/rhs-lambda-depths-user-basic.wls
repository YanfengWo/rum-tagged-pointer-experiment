Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-lambda-depths-user-basic"];
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
rumPairProbeRhsDepth0 = rumPairProbeRhsLambda[];
rumPairProbeRhsDepth1 = rumPairProbeOpenAbsN[rumPairProbeRhsDepth0, 1];
rumPairProbeRhsDepth2 = rumPairProbeOpenAbsN[rumPairProbeRhsDepth0, 2];
rumPairProbeRhsDepth3 = rumPairProbeOpenAbsN[rumPairProbeRhsDepth0, 3];
rumPairProbeRhsDepth4 = rumPairProbeOpenAbsN[rumPairProbeRhsDepth0, 4];

rumPairProbeMeasure[
  "rhs-lambda-depth4-opened-body-descendOnce",
  rumPairProbeDepth4Thm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeRhsDepth4],
  rumPairProbePrintThmResult[
    "rhs-lambda-depth4-opened-body-descendOnce",
    rumPairProbeRhsDepth4]];

rumPairProbeMeasure[
  "rhs-lambda-depth3-one-abs-descendOnce",
  rumPairProbeDepth3Thm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeRhsDepth3],
  rumPairProbePrintThmResult[
    "rhs-lambda-depth3-one-abs-descendOnce",
    rumPairProbeRhsDepth3]];

rumPairProbeMeasure[
  "rhs-lambda-depth2-two-abs-descendOnce",
  rumPairProbeDepth2Thm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeRhsDepth2],
  rumPairProbePrintThmResult[
    "rhs-lambda-depth2-two-abs-descendOnce",
    rumPairProbeRhsDepth2]];

rumPairProbeMeasure[
  "rhs-lambda-depth1-three-abs-descendOnce",
  rumPairProbeDepth1Thm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeRhsDepth1],
  rumPairProbePrintThmResult[
    "rhs-lambda-depth1-three-abs-descendOnce",
    rumPairProbeRhsDepth1]];

rumPairProbeMeasure[
  "rhs-lambda-depth0-four-abs-descendOnce",
  rumPairProbeDepth0Thm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeRhsDepth0],
  rumPairProbePrintThmResult[
    "rhs-lambda-depth0-four-abs-descendOnce",
    rumPairProbeRhsDepth0]];

Null

