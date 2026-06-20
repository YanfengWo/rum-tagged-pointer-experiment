Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-pair4-threaded-comb-repeat8"];
rumPairProbeLoadSimpStack[];
rumPairProbeSetupMkPair[];
rumPairProbeMeasure[
  "preheat-basic",
  rumPairProbePreheatBasic[],
  Function[{value}, Null]];

rumPairProbeThreadedBaseConv =
  HOL`Auto`Simp`Private`buildBaseConv[
    rumPairProbeBasicRules, {},
    HOL`Auto`Simp`Private`$simpDepthLimit];

rumPairProbeThreadedDescend[baseConv_, ruleThms_, ctxThms_, depth_][t_] :=
  Which[
    MatchQ[t, HOL`Terms`comb[_, _]],
      Module[{f = t[[1]], x = t[[2]], fEq, xEq},
        fEq = rumPairProbeThreadedFixpoint[
          baseConv, ruleThms, ctxThms, depth][f];
        xEq = rumPairProbeThreadedFixpoint[
          baseConv, ruleThms, ctxThms, depth][x];
        HOL`Kernel`MKCOMB[fEq, xEq]
      ],
    True, HOL`Kernel`REFL[t]
  ];

rumPairProbeThreadedStep[baseConv_, ruleThms_, ctxThms_, depth_][t_] :=
  Module[{topTh, rhs1, descTh},
    topTh = HOL`Drule`TRYCONV[HOL`Drule`REPEATC[baseConv]][t];
    rhs1 = HOL`Kernel`concl[topTh][[2]];
    descTh = rumPairProbeThreadedDescend[
      baseConv, ruleThms, ctxThms, depth][rhs1];
    HOL`Kernel`TRANS[topTh, descTh]
  ];

rumPairProbeThreadedFixpoint[baseConv_, ruleThms_, ctxThms_, depth_][t_] :=
  Module[{cur, rhs, next, newRhs, seen},
    cur = rumPairProbeThreadedStep[baseConv, ruleThms, ctxThms, depth][t];
    rhs = HOL`Kernel`concl[cur][[2]];
    seen = <|t -> True, rhs -> True|>;
    While[True,
      next = rumPairProbeThreadedStep[baseConv, ruleThms, ctxThms, depth][rhs];
      newRhs = HOL`Kernel`concl[next][[2]];
      If[newRhs === rhs, Break[]];
      If[KeyExistsQ[seen, newRhs], Break[]];
      cur = HOL`Kernel`TRANS[cur, next];
      rhs = newRhs;
      seen[newRhs] = True
    ];
    cur
  ];

RumProfileReset[];
rumPairProbeMeasure[
  "pair4-threaded-comb-repeat8",
  Do[
    rumPairProbeThreadedCombThm =
      rumPairProbeThreadedFixpoint[
        rumPairProbeThreadedBaseConv, rumPairProbeBasicRules, {},
        HOL`Auto`Simp`Private`$simpDepthLimit][rumPairProbePair4Same],
    {8}],
  Function[{value},
    Print["RUM_PAIR_PROBE pair4-threaded-comb-repeat8 repeats=", 8];
    Print["RUM_PAIR_PROBE pair4-threaded-comb-repeat8 is_thm=",
      HOL`Kernel`isThm[rumPairProbeThreadedCombThm]];
    Print["RUM_PAIR_PROBE pair4-threaded-comb-repeat8 changed=",
      HOL`Kernel`concl[rumPairProbeThreadedCombThm][[2]] =!=
        rumPairProbePair4Same];
    Null]];
RumProfileSnapshot["pair4-threaded-comb-repeat8-target"];

Null
