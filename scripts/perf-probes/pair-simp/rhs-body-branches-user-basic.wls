Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-body-branches-user-basic"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeUserBasicRules = rumPairProbeRules["userBasic"];
rumPairProbeOpenedBody =
  rumPairProbeOpenAbsN[rumPairProbeRhsLambda[], 4];
rumPairProbeBodyLeft = rumPairProbeOpenedBody[[1, 2]];
rumPairProbeBodyRight = rumPairProbeOpenedBody[[2]];

rumPairProbeMeasure[
  "rhs-body-and-left-user-basic",
  rumPairProbeBodyLeftThm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeBodyLeft],
  rumPairProbePrintThmResult[
    "rhs-body-and-left-user-basic",
    rumPairProbeBodyLeft]];

rumPairProbeMeasure[
  "rhs-body-and-right-no-context-user-basic",
  rumPairProbeBodyRightThm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeBodyRight],
  rumPairProbePrintThmResult[
    "rhs-body-and-right-no-context-user-basic",
    rumPairProbeBodyRight]];

Null
