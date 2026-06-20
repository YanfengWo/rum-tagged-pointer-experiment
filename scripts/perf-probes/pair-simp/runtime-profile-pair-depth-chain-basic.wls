Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-pair-depth-chain-basic"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeBasicOnlyRules = rumPairProbeRules["basic"];
rumPairProbePair1Same = rumPairProbePair2Same[[1]];
rumPairProbePair0Same = rumPairProbePair1Same[[1]];

rumPairProbeMeasure[
  "pair-depth-pair4-simpFixpointConv",
  rumPairProbeDepthPair4Thm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4Same],
  rumPairProbePrintThmResult[
    "pair-depth-pair4-simpFixpointConv",
    rumPairProbePair4Same]];

rumPairProbeMeasure[
  "pair-depth-pair3-simpFixpointConv",
  rumPairProbeDepthPair3Thm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair3Same],
  rumPairProbePrintThmResult[
    "pair-depth-pair3-simpFixpointConv",
    rumPairProbePair3Same]];

rumPairProbeMeasure[
  "pair-depth-pair2-simpFixpointConv",
  rumPairProbeDepthPair2Thm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair2Same],
  rumPairProbePrintThmResult[
    "pair-depth-pair2-simpFixpointConv",
    rumPairProbePair2Same]];

rumPairProbeMeasure[
  "pair-depth-pair1-simpFixpointConv",
  rumPairProbeDepthPair1Thm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair1Same],
  rumPairProbePrintThmResult[
    "pair-depth-pair1-simpFixpointConv",
    rumPairProbePair1Same]];

rumPairProbeMeasure[
  "pair-depth-pair0-simpFixpointConv",
  rumPairProbeDepthPair0Thm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair0Same],
  rumPairProbePrintThmResult[
    "pair-depth-pair0-simpFixpointConv",
    rumPairProbePair0Same]];

rumPairProbeMeasure[
  "pair-depth-x-simpFixpointConv",
  rumPairProbeDepthXThm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeXV],
  rumPairProbePrintThmResult[
    "pair-depth-x-simpFixpointConv",
    rumPairProbeXV]];

rumPairProbeMeasure[
  "pair-depth-y-simpFixpointConv",
  rumPairProbeDepthYThm =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeYV],
  rumPairProbePrintThmResult[
    "pair-depth-y-simpFixpointConv",
    rumPairProbeYV]];

RumProfileSnapshot["pair-depth-chain-basic"];

Null
