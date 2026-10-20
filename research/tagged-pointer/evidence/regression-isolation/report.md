# Rebuild and synthetic pipeline verification

Elapsed candidate/base ratios below one are faster. A/A medians are controls, not corrections.

| Scope | Label | candidate/base | faster blocks | base A/A | candidate A/A |
| --- | --- | ---: | ---: | ---: | ---: |
| original suite | association-part-key-read-width4000 | 0.9932 | 5/8 | 1.0080 | 1.0069 |
| original suite | association-part-key-read-width32000 | 0.9653 | 7/8 | 1.0010 | 0.9947 |
| original suite | association-part-positional-read-width4000 | 0.9955 | 6/8 | 1.0221 | 1.0119 |
| original suite | association-part-positional-read-width32000 | 0.9979 | 4/8 | 1.0217 | 1.0000 |
| original suite | loop-empty | 0.9937 | 5/8 | 1.0337 | 1.0025 |
| original suite | do-count-null | 1.0077 | 4/8 | 0.9927 | 1.0418 |
| original suite | do-iterator-null | 0.9962 | 5/8 | 1.0076 | 0.9551 |
| original suite | table-count-null | 0.9952 | 6/8 | 0.9990 | 1.0083 |
| original suite | table-iterator-null | 0.9918 | 5/8 | 0.9870 | 0.9724 |
| original suite | table-iterator-symbol | 1.0371 | 3/8 | 1.0080 | 1.0107 |
| original suite | literal-integer | 0.9888 | 6/8 | 1.0214 | 0.9795 |
| original suite | literal-string | 1.0133 | 3/8 | 1.0138 | 1.0233 |
| original suite | symbol-ownvalue-read | 1.0006 | 3/8 | 1.0109 | 1.0209 |
| original suite | symbol-ownvalue-guarded-read | 0.9844 | 6/8 | 1.0122 | 1.0318 |
| original suite | set-ownvalue | 0.9775 | 7/8 | 1.0037 | 1.0043 |
| original suite | list-literal-small | 1.0011 | 3/8 | 1.0148 | 0.9951 |
| original suite | head-list | 0.9889 | 6/8 | 1.0189 | 1.0026 |
| original suite | length-list | 0.9900 | 6/8 | 1.0092 | 1.0000 |
| original suite | part-list | 0.9982 | 4/8 | 1.0161 | 1.0072 |
| original suite | sameq-integer | 0.9920 | 6/8 | 1.0031 | 1.0080 |
| original suite | sameq-list | 0.9909 | 7/8 | 1.0042 | 1.0063 |
| original suite | if-true | 0.9860 | 6/8 | 1.0106 | 1.0047 |
| original suite | module-local | 1.0114 | 1/8 | 0.9911 | 1.0056 |
| original suite | module-chain-escape-small | 1.0262 | 2/8 | 0.9806 | 1.0025 |
| original suite | module-chain-escape-large | 1.0103 | 3/8 | 0.9873 | 1.0005 |
| original suite | block-local | 0.9934 | 6/8 | 1.0024 | 0.9993 |
| original suite | block-symbol-only-repeat | 0.9836 | 6/8 | 1.0295 | 1.0160 |
| original suite | block-init-null-repeat | 0.9778 | 7/8 | 1.0147 | 1.0137 |
| original suite | block-init-read-repeat | 1.0065 | 2/8 | 1.0053 | 1.0087 |
| original suite | with-local | 1.0163 | 0/8 | 1.0088 | 1.0117 |
| original suite | table-small | 1.0185 | 1/8 | 1.0107 | 0.9926 |
| original suite | table-iterator-plus | 1.0266 | 2/8 | 0.9958 | 0.9732 |
| original suite | table-iterator-pair-list | 1.0238 | 3/8 | 0.9780 | 0.9880 |
| original suite | do-small | 0.9911 | 7/8 | 1.0050 | 1.0029 |
| original suite | downvalue-exact-hit | 1.0010 | 3/8 | 1.0024 | 1.0055 |
| original suite | downvalue-exact-miss | 0.9963 | 5/8 | 1.0086 | 1.0031 |
| original suite | downvalue-pattern-hit | 0.9872 | 6/8 | 1.0119 | 1.0034 |
| original suite | downvalue-pattern-miss | 0.9925 | 5/8 | 1.0064 | 1.0061 |
| original suite | downvalue-exact-miss-repeat | 1.0114 | 3/8 | 1.0059 | 1.0082 |
| original suite | downvalue-pattern-miss-repeat | 1.0134 | 2/8 | 0.9997 | 1.0130 |
| original suite | downvalue-guarded-hit | 0.9973 | 5/8 | 1.0005 | 1.0038 |
| original suite | downvalue-guarded-decline-fallback | 1.0201 | 0/8 | 0.9959 | 1.0035 |
| original suite | upvalue-guarded-hit | 0.9927 | 7/8 | 0.9936 | 0.9997 |
| original suite | subvalue-unguarded-hit | 1.0007 | 4/8 | 1.0007 | 1.0028 |
| original suite | subvalue-unguarded-miss | 1.0078 | 1/8 | 0.9982 | 1.0031 |
| original suite | subvalue-guarded-hit | 0.9958 | 6/8 | 0.9918 | 1.0044 |
| original suite | default-unguarded-hit | 1.0149 | 0/8 | 1.0021 | 1.0039 |
| original suite | default-unguarded-miss | 1.0156 | 0/8 | 1.0046 | 0.9927 |
| original suite | default-guarded-hit | 1.0158 | 1/8 | 1.0006 | 1.0058 |
| original suite | matchq-integer-success | 0.9934 | 7/8 | 0.9969 | 1.0011 |
| original suite | matchq-integer-fail | 0.9931 | 6/8 | 1.0094 | 1.0018 |
| original suite | matcher-single-tiny | 1.0277 | 0/8 | 1.0099 | 0.9861 |
| original suite | matcher-single-wide-small | 1.0064 | 2/8 | 0.9845 | 0.9962 |
| original suite | matcher-single-wide-large | 1.0029 | 2/8 | 0.9894 | 1.0042 |
| original suite | replaceall-simple | 0.9906 | 6/8 | 1.0059 | 1.0050 |
| original suite | rewrite-guarded-hit | 1.0027 | 2/8 | 1.0050 | 0.9987 |
| original suite | rewrite-guarded-decline-fallback | 0.9895 | 7/8 | 1.0024 | 0.9982 |
| original suite | cases-wide-no-match | 0.9975 | 5/8 | 0.9945 | 0.9998 |
| original suite | count-wide-no-match | 0.9903 | 6/8 | 0.9915 | 0.9941 |
| original suite | position-wide-no-match | 0.9926 | 6/8 | 0.9893 | 1.0019 |
| original suite | deletecases-wide-no-match | 0.9991 | 4/8 | 1.0001 | 0.9826 |
| original suite | deletecases-wide-association-no-match | 1.0211 | 3/8 | 0.9779 | 0.9306 |
| original suite | replace-wide-no-match | 0.9150 | 8/8 | 0.9967 | 0.9957 |
| original suite | replacepart-wide-rule-list | 0.9958 | 5/8 | 1.0000 | 1.0020 |
| original suite | replacepart-guarded-hit | 1.0189 | 0/8 | 0.9950 | 1.0060 |
| original suite | replacepart-guarded-decline-fallback | 1.0084 | 1/8 | 0.9907 | 1.0104 |
| original suite | string-repeat-small | 0.9876 | 5/8 | 1.0201 | 1.0136 |
| original suite | character-range-small | 1.0004 | 4/8 | 1.0001 | 0.9946 |
| original suite | stringtrim-literal-list-4000 | 0.9791 | 6/8 | 1.0034 | 0.9947 |
| original suite | stringtrim-literal-list-32000 | 0.9911 | 6/8 | 1.0069 | 1.0000 |
| original suite | stringtrim-single-literal-4000 | 0.9830 | 7/8 | 1.0045 | 0.9890 |
| original suite | stringtrim-single-literal-32000 | 0.9935 | 5/8 | 1.0038 | 1.0039 |
| original suite | stringtrim-whitespace-4000 | 0.9964 | 5/8 | 1.0060 | 1.0094 |
| original suite | stringtrim-whitespace-32000 | 0.9993 | 4/8 | 1.0096 | 0.9952 |
| original suite | stringreplace-single-literal-unguarded | 0.9985 | 5/8 | 1.0071 | 1.0046 |
| original suite | stringreplace-pattern-unguarded | 1.0076 | 3/8 | 1.0049 | 0.9960 |
| original suite | stringreplace-guarded-hit | 1.0007 | 3/8 | 1.0100 | 1.0026 |
| original suite | stringreplace-guarded-decline-fallback | 0.9993 | 5/8 | 1.0018 | 1.0020 |
| original suite | appendto-list-incremental-small | 1.1095 | 0/8 | 0.9906 | 0.9969 |
| original suite | appendto-list-incremental-large | 0.9892 | 7/8 | 0.9932 | 1.0017 |
| original suite | appendto-part-outer1000-small | 1.0032 | 3/8 | 1.0047 | 1.0001 |
| original suite | appendto-part-outer1000-large | 1.0044 | 0/8 | 0.9992 | 1.0006 |
| original suite | association-set-key-incremental-small | 1.0076 | 0/8 | 0.9998 | 0.9905 |
| original suite | association-set-key-incremental-large | 1.0113 | 0/8 | 1.0041 | 0.9837 |
| original suite | association-small-literal-repeat | 1.0108 | 3/8 | 0.9929 | 1.0015 |
| original suite | set-part-fixed100-len100 | 1.0318 | 1/8 | 1.0119 | 1.0043 |
| original suite | set-part-fixed100-len800 | 1.0568 | 0/8 | 1.0050 | 0.9916 |
| original suite | set-part-fill-small | 1.0280 | 1/8 | 1.0017 | 0.9894 |
| original suite | set-part-fill-large | 1.0359 | 0/8 | 0.9924 | 1.0012 |
| original suite | upvalue-candidates-tiny | 1.0180 | 0/8 | 1.0054 | 1.0062 |
| original suite | upvalue-candidates-wide-small | 0.9932 | 4/8 | 1.0014 | 0.9825 |
| original suite | upvalue-candidates-wide-large | 0.9989 | 4/8 | 0.9989 | 0.9774 |
| original suite | sort-callback-small | 1.0067 | 0/8 | 1.0115 | 1.0087 |
| original suite | sort-callback-large | 1.0069 | 1/8 | 1.0165 | 1.0156 |
| original suite | sortby-callback-small | 1.0078 | 2/8 | 1.0143 | 1.0053 |
| original suite | sortby-callback-large | 1.0108 | 1/8 | 1.0170 | 1.0073 |
| original suite | reversesort-callback-small | 0.9960 | 5/8 | 1.0000 | 0.9970 |
| original suite | reversesort-callback-large | 0.9922 | 6/8 | 1.0023 | 0.9922 |
| original suite | keysort-callback-small | 0.9891 | 6/8 | 1.0006 | 1.0016 |
| original suite | keysort-callback-large | 0.9962 | 4/8 | 1.0069 | 0.9876 |
| original suite | keysortby-callback-small | 0.9880 | 6/8 | 1.0232 | 0.9769 |
| original suite | keysortby-callback-large | 1.0006 | 4/8 | 1.0181 | 0.9735 |
| original suite | sort-default-wide | 1.0263 | 0/8 | 0.9963 | 0.9975 |
| original suite | sort-symbol-default-width4000 | 1.0475 | 2/8 | 1.0153 | 1.0063 |
| original suite | sort-symbol-default-width16000 | 1.0083 | 2/8 | 1.0165 | 0.9893 |
| original suite | sortby-default-wide | 1.0006 | 3/8 | 0.9955 | 0.9968 |
| original suite | reversesort-default-wide | 1.0143 | 0/8 | 0.9998 | 0.9976 |
| original suite | keysort-default-wide | 1.0941 | 2/8 | 0.9811 | 0.9485 |
| original suite | keysortby-default-wide | 1.0077 | 3/8 | 0.9964 | 0.9882 |
| original suite | valueq-undefined-symbol-definitions-2000 | 0.9826 | 7/8 | 1.0103 | 1.0056 |
| original suite | valueq-undefined-symbol-definitions-8000 | 0.9805 | 7/8 | 1.0024 | 1.0086 |
| original suite | module-retained-gensym-gc-batch | 0.9932 | 4/8 | 1.0000 | 1.0060 |
| standalone controls | control-set-part-400 | 0.9839 | 5/8 | 1.0184 | 1.0076 |
| standalone controls | control-set-part-1600 | 0.9932 | 7/8 | 1.0194 | 1.0057 |
| standalone controls | control-sort-8000 | 1.0215 | 0/8 | 1.0003 | 0.9984 |
| standalone controls | control-sort-16000 | 1.0204 | 3/8 | 1.0074 | 0.9960 |
| standalone controls | control-rewrite-miss-16000 | 0.9074 | 8/8 | 1.0057 | 0.9995 |
| standalone controls | control-rewrite-hit-16000 | 0.9937 | 5/8 | 1.0258 | 1.0283 |
| combined pipelines | synthetic-rewrite-heavy-hit-4000 | 0.9978 | 4/8 | 1.0202 | 1.0181 |
| combined pipelines | synthetic-rewrite-heavy-hit-16000 | 0.9974 | 5/8 | 1.0217 | 1.0198 |
| combined pipelines | synthetic-mixed-hit-4000 | 0.9935 | 6/8 | 1.0190 | 1.0225 |
| combined pipelines | synthetic-mixed-hit-16000 | 0.9986 | 4/8 | 1.0037 | 1.0138 |
| combined pipelines | synthetic-mixed-control-4000 | 1.0022 | 4/8 | 1.0226 | 0.9901 |
| combined pipelines | synthetic-mixed-control-16000 | 1.0359 | 2/8 | 0.9910 | 0.7424 |
| fresh case followup | exact-part-400-null | 1.0276 | 1/8 | 0.9792 | 0.9819 |
| fresh case followup | exact-sort-8000 | 1.0153 | 2/8 | 1.0026 | 1.0074 |
| fresh case followup | fresh-rewrite-hit-16000 | 0.9833 | 5/8 | 1.0142 | 0.9921 |
| fresh case followup | synthetic-rewrite-heavy-hit-16000 | 1.0093 | 2/8 | 0.9848 | 0.9934 |
| fresh case followup | synthetic-mixed-control-16000 | 1.0066 | 3/8 | 0.9880 | 0.9985 |

55 of 112 suite medians are slower; all per-block ratios and raw retained samples are in summary.json.

The standalone Part control retains an output-list alias; it is a distinct ownership context. Fresh-case exact-part-400-null returns Null, matching the original suite pattern. Synthetic pipelines construct fresh input inside each timed window and retain explicit input and sorted aliases.

The followup was selected after primary results, is descriptive, and does not establish a GC/allocator or code-layout cause. Repetition counts are not independent block counts.

No source implementation changed in this stage. The isolated safe candidate remains under review.
