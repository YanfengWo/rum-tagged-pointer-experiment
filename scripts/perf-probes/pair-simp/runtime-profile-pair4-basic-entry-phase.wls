Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-pair4-basic-entry-phase"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeBasicOnlyRules = rumPairProbeRules["basic"];

rumPairProbeMeasure[
  "pair4-basic-buildBaseConv",
  rumPairProbeBaseConv =
    HOL`Auto`Simp`Private`buildBaseConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit],
  Function[{value}, Null]];

rumPairProbeMeasure[
  "pair4-basic-step1-top-TRYCONV-REPEATC",
  rumPairProbeStep1TopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeBaseConv]][rumPairProbePair4Same],
  rumPairProbePrintThmResult[
    "pair4-basic-step1-top-TRYCONV-REPEATC",
    rumPairProbePair4Same]];

rumPairProbeStep1Rhs1 = HOL`Kernel`concl[rumPairProbeStep1TopTh][[2]];

rumPairProbeMeasure[
  "pair4-basic-step1-descendOnce",
  rumPairProbeStep1DescTh =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeStep1Rhs1],
  rumPairProbePrintThmResult[
    "pair4-basic-step1-descendOnce",
    rumPairProbeStep1Rhs1]];

rumPairProbeStep1Th =
  HOL`Kernel`TRANS[rumPairProbeStep1TopTh, rumPairProbeStep1DescTh];
rumPairProbeStep1Rhs2 = HOL`Kernel`concl[rumPairProbeStep1Th][[2]];

rumPairProbeMeasure[
  "pair4-basic-step2-top-TRYCONV-REPEATC",
  rumPairProbeStep2TopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeBaseConv]][rumPairProbeStep1Rhs2],
  rumPairProbePrintThmResult[
    "pair4-basic-step2-top-TRYCONV-REPEATC",
    rumPairProbeStep1Rhs2]];

rumPairProbeStep2Rhs1 = HOL`Kernel`concl[rumPairProbeStep2TopTh][[2]];

rumPairProbeMeasure[
  "pair4-basic-step2-descendOnce",
  rumPairProbeStep2DescTh =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeStep2Rhs1],
  rumPairProbePrintThmResult[
    "pair4-basic-step2-descendOnce",
    rumPairProbeStep2Rhs1]];

rumPairProbeStep2Th =
  HOL`Kernel`TRANS[rumPairProbeStep2TopTh, rumPairProbeStep2DescTh];
rumPairProbeStep2Rhs2 = HOL`Kernel`concl[rumPairProbeStep2Th][[2]];

Print["RUM_PAIR_PROBE pair4-basic-step1-total changed=",
  rumPairProbeStep1Rhs2 =!= rumPairProbePair4Same];
Print["RUM_PAIR_PROBE pair4-basic-step2-newRhs-eq-rhs=",
  rumPairProbeStep2Rhs2 === rumPairProbeStep1Rhs2];

RumProfileSnapshot["pair4-basic-entry-phase"];

Null
