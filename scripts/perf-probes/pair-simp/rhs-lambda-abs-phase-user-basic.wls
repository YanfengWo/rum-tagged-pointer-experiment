Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=rhs-lambda-abs-phase-user-basic"];
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
rumPairProbeAbsTarget = rumPairProbeRhsLambda[];

rumPairProbeMeasure[
  "abs-branch-open-beta",
  rumPairProbeOpenedAbsTarget =
    Module[{bty, body, origin, forbidden, name, v, openTh},
      bty = rumPairProbeAbsTarget[[1, 2]];
      body = rumPairProbeAbsTarget[[2]];
      origin = rumPairProbeAbsTarget[[3]];
      forbidden = First /@ HOL`Terms`freesIn[body];
      name = HOL`Auto`Simp`Private`pickFreshNm[origin, forbidden];
      v = HOL`Terms`mkVar[name, bty];
      rumPairProbeAbsV = v;
      openTh = HOL`Kernel`BETA[
        HOL`Terms`mkComb[
          HOL`Terms`abs[HOL`Terms`bvar[0, bty], body, name],
          v]];
      HOL`Kernel`concl[openTh][[2]]
    ],
  Function[{value}, Null]];

rumPairProbeMeasure[
  "abs-branch-inner-simpFixpointConv",
  rumPairProbeAbsInnerEq =
    HOL`Auto`Simp`Private`simpFixpointConv[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][
        rumPairProbeOpenedAbsTarget],
  rumPairProbePrintThmResult[
    "abs-branch-inner-simpFixpointConv",
    rumPairProbeOpenedAbsTarget]];

rumPairProbeMeasure[
  "abs-branch-ABS",
  rumPairProbeAbsManualThm =
    HOL`Kernel`ABS[rumPairProbeAbsV, rumPairProbeAbsInnerEq],
  rumPairProbePrintThmResult[
    "abs-branch-ABS",
    rumPairProbeAbsTarget]];

rumPairProbeMeasure[
  "abs-branch-original-descendOnce",
  rumPairProbeAbsOriginalThm =
    HOL`Auto`Simp`Private`descendOnce[
      rumPairProbeUserBasicRules, {},
      HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbeAbsTarget],
  rumPairProbePrintThmResult[
    "abs-branch-original-descendOnce",
    rumPairProbeAbsTarget]];

Null

