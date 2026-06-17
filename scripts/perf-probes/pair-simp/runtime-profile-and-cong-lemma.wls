Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-and-cong-lemma"];
rumPairProbeLoadSimpStack[];
rumPairProbePreheatBasic[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-and-cong-lemma",
  rumPairProbeAndCongThm = HOL`Auto`Simp`Private`andCongLemma[],
  Function[{value},
    Print["RUM_PAIR_PROBE profile-and-cong-lemma is_thm=",
      HOL`Kernel`isThm[value]]]];

RumProfileSnapshot["and-cong-lemma"];

Null
