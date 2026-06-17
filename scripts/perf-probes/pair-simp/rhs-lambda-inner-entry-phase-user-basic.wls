Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-lambda-inner-entry-phase-user-basic"];
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
rumPairProbeInnerTarget = rumPairProbeOpenAbsN[rumPairProbeRhsLambda[], 1];

rumPairProbeMeasure[
  "inner-entry-buildBaseConv",
  rumPairProbeInnerBaseConv =
    HOL`Auto`Simp`Private`buildBaseConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit],
  Function[{value}, Null]];

rumPairProbeMeasure[
  "inner-entry-step1-top-TRYCONV-REPEATC",
  rumPairProbeInnerStep1TopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeInnerBaseConv]][
        rumPairProbeInnerTarget],
  rumPairProbePrintThmResult[
    "inner-entry-step1-top-TRYCONV-REPEATC",
    rumPairProbeInnerTarget]];

rumPairProbeInnerStep1Rhs1 =
  HOL`Kernel`concl[rumPairProbeInnerStep1TopTh][[2]];

rumPairProbeMeasure[
  "inner-entry-step1-descendOnce",
  rumPairProbeInnerStep1DescTh =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][
        rumPairProbeInnerStep1Rhs1],
  rumPairProbePrintThmResult[
    "inner-entry-step1-descendOnce",
    rumPairProbeInnerStep1Rhs1]];

rumPairProbeInnerStep1Th =
  HOL`Kernel`TRANS[
    rumPairProbeInnerStep1TopTh,
    rumPairProbeInnerStep1DescTh];
rumPairProbeInnerStep1Rhs2 =
  HOL`Kernel`concl[rumPairProbeInnerStep1Th][[2]];

rumPairProbeMeasure[
  "inner-entry-step2-top-TRYCONV-REPEATC",
  rumPairProbeInnerStep2TopTh =
    HOL`Drule`TRYCONV[
      HOL`Drule`REPEATC[rumPairProbeInnerBaseConv]][
        rumPairProbeInnerStep1Rhs2],
  rumPairProbePrintThmResult[
    "inner-entry-step2-top-TRYCONV-REPEATC",
    rumPairProbeInnerStep1Rhs2]];

rumPairProbeInnerStep2Rhs1 =
  HOL`Kernel`concl[rumPairProbeInnerStep2TopTh][[2]];

rumPairProbeMeasure[
  "inner-entry-step2-descendOnce",
  rumPairProbeInnerStep2DescTh =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][
        rumPairProbeInnerStep2Rhs1],
  rumPairProbePrintThmResult[
    "inner-entry-step2-descendOnce",
    rumPairProbeInnerStep2Rhs1]];

rumPairProbeInnerStep2Th =
  HOL`Kernel`TRANS[
    rumPairProbeInnerStep2TopTh,
    rumPairProbeInnerStep2DescTh];
rumPairProbeInnerStep2Rhs2 =
  HOL`Kernel`concl[rumPairProbeInnerStep2Th][[2]];

Print["RUM_PAIR_PROBE inner-entry-step1-total changed=",
  rumPairProbeInnerStep1Rhs2 =!= rumPairProbeInnerTarget];
Print["RUM_PAIR_PROBE inner-entry-step2-newRhs-eq-rhs=",
  rumPairProbeInnerStep2Rhs2 === rumPairProbeInnerStep1Rhs2];

Null

