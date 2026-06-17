Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=basic-only-pair4"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeBasicOnlyRules = rumPairProbeRules["basic"];
rumPairProbeMeasure[
  "basic-only-pair4-simpFixpointConv",
  rumPairProbeBasicOnlyThm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4Same],
  rumPairProbePrintThmResult[
    "basic-only-pair4-simpFixpointConv",
    rumPairProbePair4Same]];

Null

