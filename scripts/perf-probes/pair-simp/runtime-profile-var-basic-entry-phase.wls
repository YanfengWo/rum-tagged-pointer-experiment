Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-var-basic-entry-phase"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeBasicOnlyRules = rumPairProbeRules["basic"];

rumPairProbeMeasure[
  "var-basic-buildBaseConv",
  rumPairProbeBaseConv =
    HOL`Auto`Simp`Private`buildBaseConv[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit],
  Function[{value}, Null]];

rumPairProbeMeasure[
  "var-basic-step1-top-TRYCONV-REPEATC",
  rumPairProbeVarStep1TopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeBaseConv]][rumPairProbeXV],
  rumPairProbePrintThmResult[
    "var-basic-step1-top-TRYCONV-REPEATC",
    rumPairProbeXV]];

rumPairProbeVarStep1Rhs1 = HOL`Kernel`concl[rumPairProbeVarStep1TopTh][[2]];

rumPairProbeMeasure[
  "var-basic-step1-descendOnce",
  rumPairProbeVarStep1DescTh =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeVarStep1Rhs1],
  rumPairProbePrintThmResult[
    "var-basic-step1-descendOnce",
    rumPairProbeVarStep1Rhs1]];

rumPairProbeVarStep1Th =
  HOL`Kernel`TRANS[rumPairProbeVarStep1TopTh, rumPairProbeVarStep1DescTh];
rumPairProbeVarStep1Rhs2 = HOL`Kernel`concl[rumPairProbeVarStep1Th][[2]];

rumPairProbeMeasure[
  "var-basic-step2-top-TRYCONV-REPEATC",
  rumPairProbeVarStep2TopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeBaseConv]][rumPairProbeVarStep1Rhs2],
  rumPairProbePrintThmResult[
    "var-basic-step2-top-TRYCONV-REPEATC",
    rumPairProbeVarStep1Rhs2]];

rumPairProbeVarStep2Rhs1 = HOL`Kernel`concl[rumPairProbeVarStep2TopTh][[2]];

rumPairProbeMeasure[
  "var-basic-step2-descendOnce",
  rumPairProbeVarStep2DescTh =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeBasicOnlyRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeVarStep2Rhs1],
  rumPairProbePrintThmResult[
    "var-basic-step2-descendOnce",
    rumPairProbeVarStep2Rhs1]];

rumPairProbeVarStep2Th =
  HOL`Kernel`TRANS[rumPairProbeVarStep2TopTh, rumPairProbeVarStep2DescTh];
rumPairProbeVarStep2Rhs2 = HOL`Kernel`concl[rumPairProbeVarStep2Th][[2]];

Print["RUM_PAIR_PROBE var-basic-step1-total changed=",
  rumPairProbeVarStep1Rhs2 =!= rumPairProbeXV];
Print["RUM_PAIR_PROBE var-basic-step2-newRhs-eq-rhs=",
  rumPairProbeVarStep2Rhs2 === rumPairProbeVarStep1Rhs2];

RumProfileSnapshot["var-basic-entry-phase"];

Null
