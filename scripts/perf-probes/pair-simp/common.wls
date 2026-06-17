(* Shared setup for Pair/Simp hotspot probes.

   These probes are external diagnostics for an unmodified mma-hol checkout.
   Run them from the mma-hol repository root so relative Get paths resolve
   through ".". Do not copy these edits into mma-hol itself. *)

rumPairProbeLoadSimpStack[] := (
  Get["ErrorUtil.wl"];
  Get["Types.wl"];
  Get["Terms.wl"];
  Get["Kernel.wl"];
  Get["Bootstrap.wl"];
  Get["Equal.wl"];
  Get["Bool.wl"];
  Get["Drule.wl"];
  Get["Tactics.wl"];
  Get[FileNameJoin[{"auto", "PropTaut.wl"}]];
  Get[FileNameJoin[{"auto", "Meson.wl"}]];
  Get[FileNameJoin[{"auto", "Simp.wl"}]];
  Null
);

rumPairProbeAndC[] :=
  HOL`Kernel`mkConst[
    "∧",
    HOL`Kernel`tyFun[
      HOL`Kernel`boolTy,
      HOL`Kernel`tyFun[HOL`Kernel`boolTy, HOL`Kernel`boolTy]]];

rumPairProbeMkPairLambdaTerm[] :=
  Module[{x, y, a, b, body},
    x = HOL`Terms`mkVar["x", rumPairProbeAlphaTy];
    y = HOL`Terms`mkVar["y", rumPairProbeBetaTy];
    a = HOL`Terms`mkVar["a", rumPairProbeAlphaTy];
    b = HOL`Terms`mkVar["b", rumPairProbeBetaTy];
    body = HOL`Terms`mkAbs[a, HOL`Terms`mkAbs[b,
      HOL`Terms`mkComb[
        HOL`Terms`mkComb[rumPairProbeAndC[], HOL`Kernel`mkEq[a, x]],
        HOL`Kernel`mkEq[b, y]]]];
    HOL`Terms`mkAbs[x, HOL`Terms`mkAbs[y, body]]
  ];

rumPairProbeMkPairConst[] :=
  HOL`Kernel`mkConst["mkPair", rumPairProbeMkPairTy];

rumPairProbeSetupMkPair[] := (
  rumPairProbeAlphaTy = HOL`Types`mkVarType["A"];
  rumPairProbeBetaTy = HOL`Types`mkVarType["B"];
  rumPairProbeCharTy =
    HOL`Kernel`tyFun[
      rumPairProbeAlphaTy,
      HOL`Kernel`tyFun[rumPairProbeBetaTy, HOL`Kernel`boolTy]];
  rumPairProbeMkPairTy =
    HOL`Kernel`tyFun[
      rumPairProbeAlphaTy,
      HOL`Kernel`tyFun[rumPairProbeBetaTy, rumPairProbeCharTy]];

  rumPairProbeMkPairDefThm = HOL`Kernel`newDefinition[
    HOL`Kernel`mkEq[
      HOL`Terms`mkVar["mkPair", rumPairProbeMkPairTy],
      rumPairProbeMkPairLambdaTerm[]]];

  rumPairProbeXV = HOL`Terms`mkVar["x", rumPairProbeAlphaTy];
  rumPairProbeYV = HOL`Terms`mkVar["y", rumPairProbeBetaTy];
  rumPairProbePair2Same =
    HOL`Terms`mkComb[
      HOL`Terms`mkComb[rumPairProbeMkPairConst[], rumPairProbeXV],
      rumPairProbeYV];
  rumPairProbePair3Same =
    HOL`Terms`mkComb[rumPairProbePair2Same, rumPairProbeXV];
  rumPairProbePair4Same =
    HOL`Terms`mkComb[rumPairProbePair3Same, rumPairProbeYV];
  Null
);

rumPairProbePreheatBasic[] := (
  rumPairProbeBasicRules = HOL`Auto`Simp`basicSimpset[];
  Null
);

rumPairProbePreheatCongruence[] := (
  HOL`Auto`Simp`Private`impCongLemma[];
  HOL`Auto`Simp`Private`andCongLemma[];
  HOL`Auto`Simp`Private`orCongLemma[];
  Null
);

rumPairProbeBoolVar[name_String] :=
  HOL`Terms`mkVar[name, HOL`Kernel`boolTy];

rumPairProbeImpCongSchema[] :=
  Module[{p1, p2, q1, q2},
    p1 = rumPairProbeBoolVar["p1"]; p2 = rumPairProbeBoolVar["p2"];
    q1 = rumPairProbeBoolVar["q1"]; q2 = rumPairProbeBoolVar["q2"];
    HOL`Auto`Simp`Private`mkImp[
      HOL`Auto`Simp`Private`mkAnd[
        HOL`Kernel`mkEq[p1, p2],
        HOL`Auto`Simp`Private`mkImp[p2, HOL`Kernel`mkEq[q1, q2]]],
      HOL`Kernel`mkEq[
        HOL`Auto`Simp`Private`mkImp[p1, q1],
        HOL`Auto`Simp`Private`mkImp[p2, q2]]]
  ];

rumPairProbeTyMatchPattern[] :=
  HOL`Types`tyApp[
    "fun",
    {
      HOL`Types`tyVar["A"],
      HOL`Types`tyApp[
        "fun",
        {HOL`Types`tyVar["B"], HOL`Types`tyVar["A"]}]
    }];

rumPairProbeTyMatchTarget[] :=
  HOL`Kernel`tyFun[
    HOL`Kernel`boolTy,
    HOL`Kernel`tyFun[HOL`Kernel`indTy, HOL`Kernel`boolTy]];

rumPairProbeRules[name_] :=
  Which[
    name === "none", {},
    name === "basic", rumPairProbeBasicRules,
    name === "user", {rumPairProbeMkPairDefThm},
    name === "userBasic", Join[{rumPairProbeMkPairDefThm}, rumPairProbeBasicRules],
    True, {}
  ];

rumPairProbeRhsLambda[] :=
  HOL`Kernel`concl[HOL`Drule`REWRCONV[rumPairProbeMkPairDefThm][
    rumPairProbeMkPairConst[]]][[2]];

rumPairProbeOpenAbs[t_] :=
  Module[{bty, body, origin, forbidden, name, v, openTh},
    bty = t[[1, 2]];
    body = t[[2]];
    origin = t[[3]];
    forbidden = First /@ HOL`Terms`freesIn[body];
    name = HOL`Auto`Simp`Private`pickFreshNm[origin, forbidden];
    v = HOL`Terms`mkVar[name, bty];
    openTh = HOL`Kernel`BETA[
      HOL`Terms`mkComb[
        HOL`Terms`abs[HOL`Terms`bvar[0, bty], body, name],
        v]];
    HOL`Kernel`concl[openTh][[2]]
  ];

rumPairProbeOpenAbsN[t_, n_Integer] :=
  Module[{cur = t, i},
    Do[cur = rumPairProbeOpenAbs[cur], {i, 1, n}];
    cur
  ];

SetAttributes[rumPairProbeMeasure, HoldAll];
rumPairProbeMeasure[label_, expr_, resultFn_] :=
  Module[{seconds, result},
    Print["RUM_PAIR_PROBE start=", label];
    seconds = AbsoluteTiming[result = expr;][[1]];
    Print["RUM_PAIR_PROBE label=", label];
    Print["RUM_PAIR_PROBE seconds=", seconds];
    resultFn[result];
    result
  ];

rumPairProbePrintThmResult[label_, original_][th_] := (
  Print["RUM_PAIR_PROBE ", label, " is_thm=", HOL`Kernel`isThm[th]];
  Print["RUM_PAIR_PROBE ", label, " changed=",
    HOL`Kernel`concl[th][[2]] =!= original];
  Null
);
