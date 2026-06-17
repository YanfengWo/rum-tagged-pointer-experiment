Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-or-cong-lemma"];
rumPairProbeLoadSimpStack[];
rumPairProbePreheatBasic[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-or-cong-lemma",
  rumPairProbeOrCongThm = HOL`Auto`Simp`Private`orCongLemma[],
  Function[{value},
    Print["RUM_PAIR_PROBE profile-or-cong-lemma is_thm=",
      HOL`Kernel`isThm[value]]]];

RumProfileSnapshot["or-cong-lemma"];

Null
