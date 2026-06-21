Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-single-basic-failure-split"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumSingleProfileN = 80;
rumSingleProfileRules = rumPairProbeRules["basic"];
rumSingleProfilePreparedRules =
  HOL`Auto`Simp`simpPrepareRule /@ rumSingleProfileRules;
rumSingleProfileProductiveRules =
  Select[
    rumSingleProfilePreparedRules,
    HOL`Auto`Simp`Private`productiveSimpRule];
rumSingleProfileRule = First[rumSingleProfileProductiveRules];
rumSingleProfileProver =
  HOL`Auto`Simp`Private`makeProver[
    rumSingleProfileRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit];
rumSingleProfileConv =
  HOL`Auto`Simp`Private`simpRuleConv[
    rumSingleProfileRule, rumSingleProfileProver];
rumSingleProfileTarget = rumPairProbeXV;
rumSingleProfileConcl = HOL`Kernel`concl[rumSingleProfileRule];
rumSingleProfileLhs = rumSingleProfileConcl[[1, 2]];
rumSingleProfileEmptySubst = {<||>, <||>};

Print["RUM_PAIR_PROBE runtime-single-repeat-count=", rumSingleProfileN];
Print["RUM_PAIR_PROBE runtime-single-basic-rule-count=",
  Length[rumSingleProfileRules]];
Print["RUM_PAIR_PROBE runtime-single-productive-rule-count=",
  Length[rumSingleProfileProductiveRules]];
Print["RUM_PAIR_PROBE runtime-single-target-head=",
  Head[rumSingleProfileTarget]];
Print["RUM_PAIR_PROBE runtime-single-lhs-head=", Head[rumSingleProfileLhs]];
Print["RUM_PAIR_PROBE runtime-single-tryconv-failed=",
  HOL`Drule`Private`tryConv[
    rumSingleProfileConv[rumSingleProfileTarget]] ===
    HOL`Drule`Private`$convFailed];
Print["RUM_PAIR_PROBE runtime-single-tryTermMatch-failed=",
  HOL`Drule`Private`tryTermMatch[
    rumSingleProfileLhs, rumSingleProfileTarget,
    rumSingleProfileEmptySubst, 0] === HOL`Drule`Private`$matchFail];

SetAttributes[rumSingleProfileRepeat, HoldAll];
rumSingleProfileRepeat[label_, expr_] :=
  rumPairProbeMeasure[
    label,
    Module[{i = 0},
      While[i < rumSingleProfileN,
        expr;
        i = i + 1]],
    Function[{value}, Null]];

SetAttributes[rumSingleProfileMeasure, HoldAll];
rumSingleProfileMeasure[label_, expr_] := (
  RumProfileReset[];
  rumSingleProfileResult = rumSingleProfileRepeat[label, expr];
  RumProfileSnapshot[label];
  rumSingleProfileResult
);

rumSingleProfileManualRewr[] :=
  Module[{c, lhs, matchRes},
    c = HOL`Kernel`concl[rumSingleProfileRule];
    If[! MatchQ[
        c,
        HOL`Terms`comb[
          HOL`Terms`comb[HOL`Terms`const["=", _], _],
          _]],
      HOL`Drule`Private`convFail[
        "REWRCONV: theorem is not an equation",
        <|"concl" -> c|>]];
    lhs = c[[1, 2]];
    matchRes =
      HOL`Drule`Private`tryTermMatch[
        lhs, rumSingleProfileTarget,
        rumSingleProfileEmptySubst, 0];
    If[matchRes === HOL`Drule`Private`$matchFail,
      HOL`Drule`Private`convFail[
        "REWRCONV: term does not match LHS",
        <|"lhs" -> lhs, "target" -> rumSingleProfileTarget|>]];
    matchRes
  ];

rumSingleProfileMeasure[
  "runtime-single-full-tryConv-simpRuleConv",
  HOL`Drule`Private`tryConv[
    rumSingleProfileConv[rumSingleProfileTarget]]];

rumSingleProfileMeasure[
  "runtime-single-catch-REWRCONV-body",
  Catch[
    HOL`Drule`REWRCONV[rumSingleProfileRule][rumSingleProfileTarget],
    HOL`Error`holErrorTag]];

rumSingleProfileMeasure[
  "runtime-single-catch-manual-REWRCONV-body",
  Catch[rumSingleProfileManualRewr[], HOL`Error`holErrorTag]];

rumSingleProfileMeasure[
  "runtime-single-tryTermMatch-prebuilt-subst",
  HOL`Drule`Private`tryTermMatch[
    rumSingleProfileLhs, rumSingleProfileTarget,
    rumSingleProfileEmptySubst, 0]];

rumSingleProfileMeasure[
  "runtime-single-convFail-full-extra-catch",
  Catch[
    HOL`Drule`Private`convFail[
      "REWRCONV: term does not match LHS",
      <|"lhs" -> rumSingleProfileLhs,
        "target" -> rumSingleProfileTarget|>],
    HOL`Error`holErrorTag]];

Null
