Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-pair4-descend-comb-branches"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeBasicOnlyRules = rumPairProbeRules["basic"];
rumPairProbePair4F = rumPairProbePair4Same[[1]];
rumPairProbePair4X = rumPairProbePair4Same[[2]];

rumPairProbeMeasure[
  "pair4-descend-f-simpFixpointConv",
  rumPairProbePair4FEq =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4F],
  rumPairProbePrintThmResult[
    "pair4-descend-f-simpFixpointConv",
    rumPairProbePair4F]];

rumPairProbeMeasure[
  "pair4-descend-x-simpFixpointConv",
  rumPairProbePair4XEq =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4X],
  rumPairProbePrintThmResult[
    "pair4-descend-x-simpFixpointConv",
    rumPairProbePair4X]];

rumPairProbeMeasure[
  "pair4-descend-MKCOMB",
  rumPairProbePair4CombEq =
    HOL`Kernel`MKCOMB[rumPairProbePair4FEq, rumPairProbePair4XEq],
  rumPairProbePrintThmResult[
    "pair4-descend-MKCOMB",
    rumPairProbePair4Same]];

rumPairProbeMeasure[
  "pair4-descend-original",
  rumPairProbePair4DescEq =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4Same],
  rumPairProbePrintThmResult[
    "pair4-descend-original",
    rumPairProbePair4Same]];

RumProfileSnapshot["pair4-descend-comb-branches"];

Null
