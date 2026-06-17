Get[FileNameJoin[{DirectoryName[$InputFileName], "common.wls"}]];

Print["RUM_PAIR_PROBE probe=runtime-profile-imp-cong-lemma"];
rumPairProbeLoadSimpStack[];
rumPairProbePreheatBasic[];

RumProfileReset[];

rumPairProbeMeasure[
  "profile-imp-cong-lemma",
  rumPairProbeImpCongThm = HOL`Auto`Simp`Private`impCongLemma[],
  Function[{value},
    Print["RUM_PAIR_PROBE profile-imp-cong-lemma is_thm=",
      HOL`Kernel`isThm[value]]]];

RumProfileSnapshot["imp-cong-lemma"];

Null
