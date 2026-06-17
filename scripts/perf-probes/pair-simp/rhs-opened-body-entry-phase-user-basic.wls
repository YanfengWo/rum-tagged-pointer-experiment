Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-opened-body-entry-phase-user-basic"];
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
rumPairProbeOpenedBody =
  rumPairProbeOpenAbsN[rumPairProbeRhsLambda[], 4];

rumPairProbeMeasure[
  "entry-buildBaseConv",
  rumPairProbeBaseConv =
    HOL`Auto`Simp`Private`buildBaseConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit],
  Function[{value}, Null]];

rumPairProbeMeasure[
  "entry-step1-top-TRYCONV-REPEATC",
  rumPairProbeStep1TopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeBaseConv]][rumPairProbeOpenedBody],
  rumPairProbePrintThmResult[
    "entry-step1-top-TRYCONV-REPEATC",
    rumPairProbeOpenedBody]];

rumPairProbeStep1Rhs1 = HOL`Kernel`concl[rumPairProbeStep1TopTh][[2]];

rumPairProbeMeasure[
  "entry-step1-descendOnce",
  rumPairProbeStep1DescTh =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeStep1Rhs1],
  rumPairProbePrintThmResult[
    "entry-step1-descendOnce",
    rumPairProbeStep1Rhs1]];

rumPairProbeStep1Th =
  HOL`Kernel`TRANS[rumPairProbeStep1TopTh, rumPairProbeStep1DescTh];
rumPairProbeStep1Rhs2 = HOL`Kernel`concl[rumPairProbeStep1Th][[2]];

rumPairProbeMeasure[
  "entry-step2-top-TRYCONV-REPEATC",
  rumPairProbeStep2TopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeBaseConv]][rumPairProbeStep1Rhs2],
  rumPairProbePrintThmResult[
    "entry-step2-top-TRYCONV-REPEATC",
    rumPairProbeStep1Rhs2]];

rumPairProbeStep2Rhs1 = HOL`Kernel`concl[rumPairProbeStep2TopTh][[2]];

rumPairProbeMeasure[
  "entry-step2-descendOnce",
  rumPairProbeStep2DescTh =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeStep2Rhs1],
  rumPairProbePrintThmResult[
    "entry-step2-descendOnce",
    rumPairProbeStep2Rhs1]];

rumPairProbeStep2Th =
  HOL`Kernel`TRANS[rumPairProbeStep2TopTh, rumPairProbeStep2DescTh];
rumPairProbeStep2Rhs2 = HOL`Kernel`concl[rumPairProbeStep2Th][[2]];

Print["RUM_PAIR_PROBE entry-step1-total changed=",
  rumPairProbeStep1Rhs2 =!= rumPairProbeOpenedBody];
Print["RUM_PAIR_PROBE entry-step2-newRhs-eq-rhs=",
  rumPairProbeStep2Rhs2 === rumPairProbeStep1Rhs2];

Null
