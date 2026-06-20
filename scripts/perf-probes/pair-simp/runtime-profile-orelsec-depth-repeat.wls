Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-orelsec-depth-repeat"];
Get["ErrorUtil.wl"];
Get["Types.wl"];
Get["Terms.wl"];
Get["Kernel.wl"];
Get["Bootstrap.wl"];
Get["Equal.wl"];
Get["Bool.wl"];
Get["Drule.wl"];

rumOrelsecDepth = 19;
rumOrelsecRepeats = 1000;
rumOrelsecTerm = HOL`Terms`mkVar["p", HOL`Kernel`boolTy];

rumOrelsecMakeFailChain[n_] :=
  If[n <= 0,
    HOL`Drule`ALLCONV,
    HOL`Drule`ORELSEC[
      HOL`Drule`NOCONV, rumOrelsecMakeFailChain[n - 1]]];

rumOrelsecMakeAllFailChain[n_] :=
  If[n <= 0,
    HOL`Drule`NOCONV,
    HOL`Drule`ORELSEC[
      HOL`Drule`NOCONV, rumOrelsecMakeAllFailChain[n - 1]]];

SetAttributes[rumOrelsecRepeat, HoldFirst];
rumOrelsecRepeat[body_] :=
  Module[{i = 0},
    While[i < rumOrelsecRepeats,
      body;
      i++];
    Null
  ];

rumOrelsecFailChain = rumOrelsecMakeFailChain[rumOrelsecDepth];
rumOrelsecAllFailChain = rumOrelsecMakeAllFailChain[rumOrelsecDepth];

rumPairProbeMeasure[
  "orelsec-depth-loop-null",
  rumOrelsecRepeat[Null],
  Function[{value}, Null]];

rumPairProbeMeasure[
  "orelsec-depth-nested-fail-chain",
  rumOrelsecRepeat[
    rumOrelsecLast =
      rumOrelsecFailChain[rumOrelsecTerm]],
  Function[{value},
    Print["RUM_PAIR_PROBE orelsec-depth=", rumOrelsecDepth];
    Print["RUM_PAIR_PROBE orelsec-repeats=", rumOrelsecRepeats];
    Print["RUM_PAIR_PROBE orelsec-nested is_thm=",
      HOL`Kernel`isThm[rumOrelsecLast]];
    Null]];

rumPairProbeMeasure[
  "orelsec-depth-manual-noconv-loop",
  rumOrelsecRepeat[
    Module[{i = 0, res = HOL`Drule`Private`$convFailed},
      While[i < rumOrelsecDepth,
        res = HOL`Drule`Private`tryConv[
          HOL`Drule`NOCONV[rumOrelsecTerm]];
        If[res =!= HOL`Drule`Private`$convFailed, Break[]];
        i++];
      rumOrelsecLast = HOL`Kernel`REFL[rumOrelsecTerm]]],
  Function[{value},
    Print["RUM_PAIR_PROBE orelsec-manual is_thm=",
      HOL`Kernel`isThm[rumOrelsecLast]];
    Null]];

rumPairProbeMeasure[
  "orelsec-depth-repeatc-all-fail-chain",
  rumOrelsecRepeat[
    rumOrelsecLast =
      HOL`Drule`TRYCONV[
        HOL`Drule`REPEATC[rumOrelsecAllFailChain]][rumOrelsecTerm]],
  Function[{value},
    Print["RUM_PAIR_PROBE orelsec-repeatc-all-fail is_thm=",
      HOL`Kernel`isThm[rumOrelsecLast]];
    Null]];

rumPairProbeMeasure[
  "orelsec-depth-manual-all-fail-loop-refl",
  rumOrelsecRepeat[
    Module[{i = 0, res = HOL`Drule`Private`$convFailed},
      While[i <= rumOrelsecDepth,
        res = HOL`Drule`Private`tryConv[
          HOL`Drule`NOCONV[rumOrelsecTerm]];
        If[res =!= HOL`Drule`Private`$convFailed, Break[]];
        i++];
      rumOrelsecLast = HOL`Kernel`REFL[rumOrelsecTerm]]],
  Function[{value},
    Print["RUM_PAIR_PROBE orelsec-manual-all-fail is_thm=",
      HOL`Kernel`isThm[rumOrelsecLast]];
    Null]];

Null
