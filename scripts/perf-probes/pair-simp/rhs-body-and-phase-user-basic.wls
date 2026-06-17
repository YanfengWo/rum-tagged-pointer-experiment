Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-body-and-phase-user-basic"];
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
rumPairProbeAndLeft = rumPairProbeOpenedBody[[1, 2]];
rumPairProbeAndRight = rumPairProbeOpenedBody[[2]];

rumPairProbeMeasure[
  "doAndCong-left-simpFixpointConv",
  rumPairProbeAndLeftEq =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeAndLeft],
  rumPairProbePrintThmResult[
    "doAndCong-left-simpFixpointConv",
    rumPairProbeAndLeft]];

rumPairProbeAndLeftRhs = HOL`Kernel`concl[rumPairProbeAndLeftEq][[2]];
rumPairProbeAndCtxThm = HOL`Kernel`ASSUME[rumPairProbeAndLeftRhs];

rumPairProbeMeasure[
  "doAndCong-right-simpFixpointConv-with-left-context",
  rumPairProbeAndRightEq =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {rumPairProbeAndCtxThm},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeAndRight],
  rumPairProbePrintThmResult[
    "doAndCong-right-simpFixpointConv-with-left-context",
    rumPairProbeAndRight]];

rumPairProbeAndRightRhs = HOL`Kernel`concl[rumPairProbeAndRightEq][[2]];

rumPairProbeMeasure[
  "doAndCong-theorem-plumbing",
  rumPairProbeAndCongThm =
    Module[{dischEq2, conjAx, p1V, p2V, q1V, q2V, instLemma},
      dischEq2 = HOL`Bool`DISCH[
        rumPairProbeAndLeftRhs, rumPairProbeAndRightEq];
      conjAx = HOL`Bool`CONJ[rumPairProbeAndLeftEq, dischEq2];
      p1V = HOL`Terms`mkVar["p1", HOL`Kernel`boolTy];
      p2V = HOL`Terms`mkVar["p2", HOL`Kernel`boolTy];
      q1V = HOL`Terms`mkVar["q1", HOL`Kernel`boolTy];
      q2V = HOL`Terms`mkVar["q2", HOL`Kernel`boolTy];
      instLemma = HOL`Kernel`INST[
        {
          p1V -> rumPairProbeAndLeft,
          p2V -> rumPairProbeAndLeftRhs,
          q1V -> rumPairProbeAndRight,
          q2V -> rumPairProbeAndRightRhs
        },
        HOL`Auto`Simp`Private`andCongLemma[]];
      HOL`Bool`MP[instLemma, conjAx]
    ],
  rumPairProbePrintThmResult[
    "doAndCong-theorem-plumbing",
    rumPairProbeOpenedBody]];

rumPairProbeMeasure[
  "doAndCong-original-descendOnce",
  rumPairProbeAndDescendThm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeOpenedBody],
  rumPairProbePrintThmResult[
    "doAndCong-original-descendOnce",
    rumPairProbeOpenedBody]];

Null
