Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=single-basic-failure-split"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbePreheatBasic[];

rumSingleN = 500;
rumSingleBasicRules = rumPairProbeRules["basic"];
rumSinglePreparedRules =
  HOL`Auto`Simp`simpPrepareRule /@ rumSingleBasicRules;
rumSingleProductiveRules =
  Select[
    rumSinglePreparedRules,
    HOL`Auto`Simp`Private`productiveSimpRule];
rumSingleRule = First[rumSingleProductiveRules];
rumSingleProver =
  HOL`Auto`Simp`Private`makeProver[
    rumSingleBasicRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit];
rumSingleConv =
  HOL`Auto`Simp`Private`simpRuleConv[rumSingleRule, rumSingleProver];
rumSingleTarget = rumPairProbeXV;
rumSingleConcl = HOL`Kernel`concl[rumSingleRule];
rumSingleLhs = rumSingleConcl[[1, 2]];
rumSingleEmptySubst = {<||>, <||>};
rumSingleFailure =
  Catch[
    HOL`Drule`Private`convFail[
      "REWRCONV: term does not match LHS",
      <|"lhs" -> rumSingleLhs, "target" -> rumSingleTarget|>],
    HOL`Error`holErrorTag];

Print["RUM_PAIR_PROBE single-repeat-count=", rumSingleN];
Print["RUM_PAIR_PROBE single-basic-rule-count=", Length[rumSingleBasicRules]];
Print["RUM_PAIR_PROBE single-productive-rule-count=",
  Length[rumSingleProductiveRules]];
Print["RUM_PAIR_PROBE single-target-head=", Head[rumSingleTarget]];
Print["RUM_PAIR_PROBE single-lhs-head=", Head[rumSingleLhs]];
Print["RUM_PAIR_PROBE single-tryconv-failed=",
  HOL`Drule`Private`tryConv[rumSingleConv[rumSingleTarget]] ===
    HOL`Drule`Private`$convFailed];
Print["RUM_PAIR_PROBE single-catch-failure=",
  MatchQ[Catch[rumSingleConv[rumSingleTarget], HOL`Error`holErrorTag],
    _Failure]];
Print["RUM_PAIR_PROBE single-tryTermMatch-failed=",
  HOL`Drule`Private`tryTermMatch[
    rumSingleLhs, rumSingleTarget, {<||>, <||>}, 0] ===
    HOL`Drule`Private`$matchFail];

rumSingleCountCheck = Module[{i = 0, count = 0},
  While[i < rumSingleN,
    count = count + 1;
    i = i + 1];
  count];
Print["RUM_PAIR_PROBE single-repeat-count-check=", rumSingleCountCheck];

SetAttributes[rumSingleRepeat, HoldAll];
rumSingleRepeat[label_, expr_] :=
  rumPairProbeMeasure[
    label,
    Module[{i = 0},
      While[i < rumSingleN,
        expr;
        i = i + 1]],
    Function[{value}, Null]];

rumSingleManualRewr[] :=
  Module[{c, lhs, matchRes},
    c = HOL`Kernel`concl[rumSingleRule];
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
        lhs, rumSingleTarget, {<||>, <||>}, 0];
    If[matchRes === HOL`Drule`Private`$matchFail,
      HOL`Drule`Private`convFail[
        "REWRCONV: term does not match LHS",
        <|"lhs" -> lhs, "target" -> rumSingleTarget|>]];
    matchRes
  ];

rumSingleRepeat[
  "single-full-tryConv-simpRuleConv",
  HOL`Drule`Private`tryConv[rumSingleConv[rumSingleTarget]]];

rumSingleRepeat[
  "single-inline-tryConv-simpRuleConv",
  Module[{res},
    res = Catch[rumSingleConv[rumSingleTarget], HOL`Error`holErrorTag];
    If[MatchQ[res, _Failure], HOL`Drule`Private`$convFailed, res]]];

rumSingleRepeat[
  "single-catch-simpRuleConv-body",
  Catch[rumSingleConv[rumSingleTarget], HOL`Error`holErrorTag]];

rumSingleRepeat[
  "single-catch-REWRCONV-body",
  Catch[
    HOL`Drule`REWRCONV[rumSingleRule][rumSingleTarget],
    HOL`Error`holErrorTag]];

rumSingleRepeat[
  "single-catch-manual-REWRCONV-body",
  Catch[rumSingleManualRewr[], HOL`Error`holErrorTag]];

rumSingleRepeat[
  "single-tryTermMatch-fresh-subst",
  HOL`Drule`Private`tryTermMatch[
    rumSingleLhs, rumSingleTarget, {<||>, <||>}, 0]];

rumSingleRepeat[
  "single-tryTermMatch-prebuilt-subst",
  HOL`Drule`Private`tryTermMatch[
    rumSingleLhs, rumSingleTarget, rumSingleEmptySubst, 0]];

rumSingleRepeat[
  "single-concl",
  HOL`Kernel`concl[rumSingleRule]];

rumSingleRepeat[
  "single-equation-MatchQ",
  MatchQ[
    rumSingleConcl,
    HOL`Terms`comb[
      HOL`Terms`comb[HOL`Terms`const["=", _], _],
      _]]];

rumSingleRepeat[
  "single-isEquation",
  HOL`Auto`Simp`Private`isEquation[rumSingleConcl]];

rumSingleRepeat[
  "single-lhs-part",
  rumSingleConcl[[1, 2]]];

rumSingleRepeat[
  "single-convFail-full-extra-catch",
  Catch[
    HOL`Drule`Private`convFail[
      "REWRCONV: term does not match LHS",
      <|"lhs" -> rumSingleLhs, "target" -> rumSingleTarget|>],
    HOL`Error`holErrorTag]];

rumSingleRepeat[
  "single-holError-full-extra-catch",
  Catch[
    HOL`Error`holError[
      "conv",
      "REWRCONV: term does not match LHS",
      <|"lhs" -> rumSingleLhs, "target" -> rumSingleTarget|>],
    HOL`Error`holErrorTag]];

rumSingleRepeat[
  "single-failure-construct-full-extra",
  Failure[
    "HOLError",
    Join[
      <|
        "MessageTemplate" -> "[`tag`] `msg`",
        "MessageParameters" -> <|
          "tag" -> "conv",
          "msg" -> "REWRCONV: term does not match LHS"|>,
        "tag" -> "conv",
        "msg" -> "REWRCONV: term does not match LHS",
        "stack" -> Stack[]
      |>,
      <|"lhs" -> rumSingleLhs, "target" -> rumSingleTarget|>]]];

rumSingleRepeat[
  "single-throw-catch-prebuilt-failure",
  Catch[Throw[rumSingleFailure, HOL`Error`holErrorTag],
    HOL`Error`holErrorTag]];

rumSingleRepeat[
  "single-catch-return-prebuilt-failure",
  Catch[rumSingleFailure, HOL`Error`holErrorTag]];

rumSingleRepeat[
  "single-MatchQ-failure",
  MatchQ[rumSingleFailure, _Failure]];

rumSingleRepeat[
  "single-If-MatchQ-failure",
  If[MatchQ[rumSingleFailure, _Failure],
    HOL`Drule`Private`$convFailed,
    rumSingleFailure]];

rumSingleRepeat[
  "single-loop-empty",
  Null];

Null
