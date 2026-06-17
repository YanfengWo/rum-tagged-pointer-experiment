Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=user-basic-pair2"];
Print["RUM_PAIR_PROBE expected=long-or-timeout-in-rum"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeUserBasicRules = rumPairProbeRules["userBasic"];
rumPairProbeMeasure[
  "user-basic-pair2-simpFixpointConv",
  rumPairProbeUserBasicPair2Thm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair2Same],
  rumPairProbePrintThmResult[
    "user-basic-pair2-simpFixpointConv",
    rumPairProbePair2Same]];

Null

