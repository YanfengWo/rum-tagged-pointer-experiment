Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-conv-control-repeat"];

Get["ErrorUtil.wl"];
Get["Types.wl"];
Get["Terms.wl"];
Get["Kernel.wl"];
Get["Bootstrap.wl"];
Get["Equal.wl"];
Get["Bool.wl"];
Get["Drule.wl"];

rumConvProbeN = 1000;
rumConvProbeTerm = HOL`Terms`mkVar["p", HOL`Kernel`boolTy];
rumConvProbeEqTy =
  HOL`Kernel`tyFun[
    HOL`Kernel`boolTy,
    HOL`Kernel`tyFun[HOL`Kernel`boolTy, HOL`Kernel`boolTy]];

SetAttributes[rumConvProbeRepeat, HoldFirst];
rumConvProbeRepeat[body_] :=
  Module[{i = 0},
    While[i < rumConvProbeN,
      body;
      i++];
    Null
  ];

rumPairProbeMeasure[
  "conv-control-loop-null",
  rumConvProbeRepeat[Null],
  Function[{value}, Null]];

RumProfileReset[];
rumPairProbeMeasure[
  "conv-control-mkconst-eq",
  rumConvProbeRepeat[
    rumConvProbeLast =
      HOL`Kernel`mkConst["=", rumConvProbeEqTy]],
  Function[{value},
    Print["RUM_PAIR_PROBE mkconst-eq head=", Head[rumConvProbeLast]];
    Null]];
RumProfileSnapshot["conv-control-mkconst-eq"];

RumProfileReset[];
rumPairProbeMeasure[
  "conv-control-mkeq-var",
  rumConvProbeRepeat[
    rumConvProbeLast =
      HOL`Kernel`mkEq[rumConvProbeTerm, rumConvProbeTerm]],
  Function[{value},
    Print["RUM_PAIR_PROBE mkeq-var head=", Head[rumConvProbeLast]];
    Null]];
RumProfileSnapshot["conv-control-mkeq-var"];

RumProfileReset[];
rumPairProbeMeasure[
  "conv-control-refl-var",
  rumConvProbeRepeat[
    rumConvProbeLast =
      HOL`Kernel`REFL[rumConvProbeTerm]],
  Function[{value},
    Print["RUM_PAIR_PROBE refl-var is_thm=",
      HOL`Kernel`isThm[rumConvProbeLast]];
    Null]];
RumProfileSnapshot["conv-control-refl-var"];

RumProfileReset[];
rumPairProbeMeasure[
  "conv-control-raw-noconv-catch",
  rumConvProbeRepeat[
    rumConvProbeLast =
      Catch[HOL`Drule`NOCONV[rumConvProbeTerm], HOL`Error`holErrorTag]],
  Function[{value},
    Print["RUM_PAIR_PROBE conv-control repeats=", rumConvProbeN];
    Print["RUM_PAIR_PROBE raw-noconv-catch failure=",
      MatchQ[rumConvProbeLast, _Failure]];
    Null]];
RumProfileSnapshot["conv-control-raw-noconv-catch"];

RumProfileReset[];
rumPairProbeMeasure[
  "conv-control-tryconv-noconv",
  rumConvProbeRepeat[
    rumConvProbeLast =
      HOL`Drule`TRYCONV[HOL`Drule`NOCONV][rumConvProbeTerm]],
  Function[{value},
    Print["RUM_PAIR_PROBE tryconv-noconv is_thm=",
      HOL`Kernel`isThm[rumConvProbeLast]];
    Null]];
RumProfileSnapshot["conv-control-tryconv-noconv"];

RumProfileReset[];
rumPairProbeMeasure[
  "conv-control-orelsec-noconv-all",
  rumConvProbeRepeat[
    rumConvProbeLast =
      HOL`Drule`ORELSEC[
        HOL`Drule`NOCONV, HOL`Drule`ALLCONV][rumConvProbeTerm]],
  Function[{value},
    Print["RUM_PAIR_PROBE orelsec-noconv-all is_thm=",
      HOL`Kernel`isThm[rumConvProbeLast]];
    Null]];
RumProfileSnapshot["conv-control-orelsec-noconv-all"];

RumProfileReset[];
rumPairProbeMeasure[
  "conv-control-orelsec-all-noconv",
  rumConvProbeRepeat[
    rumConvProbeLast =
      HOL`Drule`ORELSEC[
        HOL`Drule`ALLCONV, HOL`Drule`NOCONV][rumConvProbeTerm]],
  Function[{value},
    Print["RUM_PAIR_PROBE orelsec-all-noconv is_thm=",
      HOL`Kernel`isThm[rumConvProbeLast]];
    Null]];
RumProfileSnapshot["conv-control-orelsec-all-noconv"];

RumProfileReset[];
rumPairProbeMeasure[
  "conv-control-repeatc-noconv",
  rumConvProbeRepeat[
    rumConvProbeLast =
      HOL`Drule`REPEATC[HOL`Drule`NOCONV][rumConvProbeTerm]],
  Function[{value},
    Print["RUM_PAIR_PROBE repeatc-noconv is_thm=",
      HOL`Kernel`isThm[rumConvProbeLast]];
    Null]];
RumProfileSnapshot["conv-control-repeatc-noconv"];

Null
