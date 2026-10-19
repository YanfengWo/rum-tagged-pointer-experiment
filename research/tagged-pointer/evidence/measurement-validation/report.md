# Balanced A/A and native Sort replication

Time ratios below 1 are faster. Each result has ten paired process blocks. The original suite generally takes min9; no whole-application average is computed.

| Label | A/A median [range] | api/base | sort-api/base | sort24/base [p10,p90] | sort24/sort-api | Sort wins |
| --- | --- | ---: | ---: | --- | ---: | ---: |
| association-part-key-read-width4000 | 1.006 [0.580,1.096] | 1.097 | 1.105 | 0.989 [0.938,1.025] | 0.898 | 7/10 |
| association-part-key-read-width32000 | 1.004 [0.495,1.094] | 1.093 | 1.094 | 0.999 [0.964,1.014] | 0.911 | 5/10 |
| sort-symbol-default-width4000 | 0.989 [0.832,1.271] | 1.036 | 0.989 | 0.909 [0.838,1.005] | 0.891 | 8/10 |
| sort-symbol-default-width16000 | 0.997 [0.970,1.121] | 0.989 | 0.999 | 0.940 [0.916,0.961] | 0.942 | 9/10 |

All labels (exploratory outside predeclared focus):

| Label | A/A | api/base | sort-api/base | sort24/base |
| --- | ---: | ---: | ---: | ---: |
| association-part-key-read-width4000 | 1.006 | 1.097 | 1.105 | 0.989 |
| association-part-key-read-width32000 | 1.004 | 1.093 | 1.094 | 0.999 |
| association-part-positional-read-width4000 | 1.014 | 1.000 | 1.001 | 1.020 |
| association-part-positional-read-width32000 | 1.011 | 1.007 | 1.002 | 1.026 |
| loop-empty | 0.975 | 0.973 | 0.976 | 1.026 |
| do-count-null | 0.996 | 0.957 | 0.961 | 0.952 |
| do-iterator-null | 1.015 | 0.980 | 0.986 | 0.977 |
| table-count-null | 0.987 | 0.994 | 0.979 | 0.940 |
| table-iterator-null | 0.997 | 0.963 | 1.008 | 0.985 |
| table-iterator-symbol | 0.997 | 0.985 | 1.014 | 0.976 |
| literal-integer | 0.986 | 0.971 | 0.998 | 1.058 |
| literal-string | 1.002 | 0.960 | 0.991 | 1.046 |
| symbol-ownvalue-read | 0.978 | 0.983 | 0.974 | 1.025 |
| symbol-ownvalue-guarded-read | 0.981 | 0.977 | 0.978 | 0.989 |
| set-ownvalue | 1.000 | 0.976 | 0.985 | 0.982 |
| list-literal-small | 0.986 | 0.982 | 1.008 | 1.033 |
| head-list | 0.999 | 0.983 | 0.992 | 0.996 |
| length-list | 0.992 | 0.993 | 0.996 | 0.995 |
| part-list | 0.999 | 1.010 | 1.007 | 1.014 |
| sameq-integer | 1.006 | 1.007 | 1.012 | 1.010 |
| sameq-list | 1.000 | 0.990 | 0.991 | 0.996 |
| if-true | 1.001 | 1.013 | 1.013 | 0.996 |
| module-local | 0.996 | 0.985 | 1.000 | 0.994 |
| module-chain-escape-small | 0.993 | 0.968 | 1.027 | 1.022 |
| module-chain-escape-large | 0.990 | 0.971 | 0.999 | 1.027 |
| block-local | 1.005 | 0.983 | 0.987 | 0.994 |
| block-symbol-only-repeat | 1.007 | 0.959 | 0.970 | 0.972 |
| block-init-null-repeat | 1.013 | 0.962 | 0.965 | 0.960 |
| block-init-read-repeat | 1.010 | 0.982 | 1.003 | 0.992 |
| with-local | 1.002 | 1.006 | 1.009 | 0.997 |
| table-small | 0.995 | 0.990 | 1.015 | 1.008 |
| table-iterator-plus | 1.010 | 1.035 | 1.007 | 0.999 |
| table-iterator-pair-list | 1.010 | 1.008 | 1.015 | 1.007 |
| do-small | 0.999 | 0.994 | 1.006 | 1.002 |
| downvalue-exact-hit | 1.008 | 1.008 | 1.002 | 1.008 |
| downvalue-exact-miss | 1.003 | 0.991 | 1.009 | 1.024 |
| downvalue-pattern-hit | 1.015 | 1.007 | 0.985 | 0.984 |
| downvalue-pattern-miss | 1.006 | 0.994 | 1.008 | 1.031 |
| downvalue-exact-miss-repeat | 1.009 | 0.989 | 1.000 | 0.996 |
| downvalue-pattern-miss-repeat | 0.998 | 0.987 | 1.002 | 0.999 |
| downvalue-guarded-hit | 1.005 | 0.999 | 0.995 | 0.981 |
| downvalue-guarded-decline-fallback | 1.003 | 1.011 | 1.005 | 1.002 |
| upvalue-guarded-hit | 1.002 | 1.005 | 1.001 | 0.997 |
| subvalue-unguarded-hit | 1.002 | 0.999 | 1.000 | 0.987 |
| subvalue-unguarded-miss | 1.004 | 0.985 | 0.980 | 0.983 |
| subvalue-guarded-hit | 1.002 | 0.999 | 0.993 | 0.988 |
| default-unguarded-hit | 0.998 | 1.021 | 1.023 | 1.012 |
| default-unguarded-miss | 1.003 | 1.009 | 1.016 | 0.996 |
| default-guarded-hit | 1.001 | 1.035 | 1.032 | 1.026 |
| matchq-integer-success | 1.013 | 1.001 | 1.006 | 1.007 |
| matchq-integer-fail | 1.015 | 0.991 | 0.996 | 0.994 |
| matcher-single-tiny | 1.007 | 0.981 | 0.992 | 0.983 |
| matcher-single-wide-small | 1.005 | 0.996 | 0.995 | 1.001 |
| matcher-single-wide-large | 1.010 | 0.998 | 1.003 | 0.997 |
| replaceall-simple | 1.019 | 0.984 | 0.997 | 1.005 |
| rewrite-guarded-hit | 0.998 | 1.006 | 1.004 | 0.994 |
| rewrite-guarded-decline-fallback | 0.997 | 1.001 | 1.010 | 0.989 |
| cases-wide-no-match | 1.011 | 0.994 | 1.002 | 1.007 |
| count-wide-no-match | 1.008 | 0.999 | 1.005 | 0.994 |
| position-wide-no-match | 1.005 | 0.993 | 0.995 | 0.998 |
| deletecases-wide-no-match | 1.011 | 0.997 | 1.000 | 0.993 |
| deletecases-wide-association-no-match | 0.966 | 1.014 | 1.044 | 0.996 |
| replace-wide-no-match | 1.002 | 0.997 | 0.997 | 1.001 |
| replacepart-wide-rule-list | 0.996 | 0.982 | 1.003 | 0.964 |
| replacepart-guarded-hit | 0.991 | 0.997 | 0.999 | 1.000 |
| replacepart-guarded-decline-fallback | 0.997 | 0.997 | 1.000 | 1.000 |
| string-repeat-small | 0.997 | 0.987 | 0.990 | 0.999 |
| character-range-small | 0.984 | 1.009 | 1.004 | 0.996 |
| stringtrim-literal-list-4000 | 1.006 | 0.979 | 0.991 | 0.983 |
| stringtrim-literal-list-32000 | 1.001 | 0.982 | 0.987 | 0.975 |
| stringtrim-single-literal-4000 | 1.023 | 0.973 | 0.976 | 0.988 |
| stringtrim-single-literal-32000 | 1.002 | 0.971 | 0.976 | 0.992 |
| stringtrim-whitespace-4000 | 1.000 | 0.970 | 0.973 | 0.971 |
| stringtrim-whitespace-32000 | 1.000 | 0.971 | 0.970 | 0.970 |
| stringreplace-single-literal-unguarded | 1.004 | 0.999 | 1.007 | 0.976 |
| stringreplace-pattern-unguarded | 0.998 | 0.992 | 0.999 | 0.983 |
| stringreplace-guarded-hit | 1.000 | 1.002 | 1.004 | 0.995 |
| stringreplace-guarded-decline-fallback | 1.007 | 1.000 | 1.015 | 1.006 |
| appendto-list-incremental-small | 1.001 | 0.976 | 0.956 | 0.957 |
| appendto-list-incremental-large | 0.993 | 1.038 | 1.020 | 1.086 |
| appendto-part-outer1000-small | 1.004 | 1.003 | 1.001 | 1.006 |
| appendto-part-outer1000-large | 0.998 | 1.002 | 1.001 | 0.996 |
| association-set-key-incremental-small | 0.991 | 0.992 | 0.999 | 1.149 |
| association-set-key-incremental-large | 0.994 | 0.994 | 0.999 | 1.138 |
| association-small-literal-repeat | 0.986 | 0.995 | 0.985 | 0.989 |
| set-part-fixed100-len100 | 0.999 | 0.996 | 1.005 | 1.005 |
| set-part-fixed100-len800 | 1.005 | 0.989 | 0.993 | 1.001 |
| set-part-fill-small | 0.987 | 0.995 | 0.996 | 1.009 |
| set-part-fill-large | 0.986 | 0.984 | 0.983 | 0.996 |
| upvalue-candidates-tiny | 0.996 | 1.020 | 1.021 | 1.001 |
| upvalue-candidates-wide-small | 1.029 | 0.979 | 0.984 | 0.974 |
| upvalue-candidates-wide-large | 1.031 | 1.000 | 1.004 | 0.983 |
| sort-callback-small | 1.011 | 1.004 | 0.964 | 0.995 |
| sort-callback-large | 1.005 | 0.996 | 0.954 | 0.994 |
| sortby-callback-small | 1.001 | 0.990 | 0.960 | 0.988 |
| sortby-callback-large | 1.006 | 0.996 | 0.967 | 0.995 |
| reversesort-callback-small | 0.989 | 0.995 | 0.993 | 1.008 |
| reversesort-callback-large | 1.003 | 0.994 | 1.006 | 0.999 |
| keysort-callback-small | 1.001 | 0.970 | 0.970 | 0.975 |
| keysort-callback-large | 0.993 | 0.978 | 0.981 | 0.968 |
| keysortby-callback-small | 0.998 | 0.986 | 1.011 | 0.992 |
| keysortby-callback-large | 0.996 | 0.977 | 1.001 | 0.975 |
| sort-default-wide | 0.997 | 1.000 | 1.007 | 0.984 |
| sort-symbol-default-width4000 | 0.989 | 1.036 | 0.989 | 0.909 |
| sort-symbol-default-width16000 | 0.997 | 0.989 | 0.999 | 0.940 |
| sortby-default-wide | 1.000 | 0.998 | 1.006 | 0.996 |
| reversesort-default-wide | 0.999 | 1.006 | 1.009 | 0.997 |
| keysort-default-wide | 0.976 | 0.907 | 0.948 | 1.005 |
| keysortby-default-wide | 1.013 | 0.984 | 0.988 | 0.979 |
| valueq-undefined-symbol-definitions-2000 | 0.999 | 0.997 | 0.998 | 1.002 |
| valueq-undefined-symbol-definitions-8000 | 1.006 | 0.994 | 0.994 | 0.999 |
| module-retained-gensym-gc-batch | 0.978 | 1.054 | 1.030 | 1.005 |

Limits:

- Ten paired blocks, not ninety independent repetitions; most script measurements are min9, retained GC uses the original final-batch convention.
- Position-balanced rotation and reversal do not balance all treatment carryover pairs or remove arbitrary within-block drift.
- Each position stratum contains only two blocks; filtered per_block values are mapped by explicit block_ids and do not support strong position-specific conclusions.
- All modes inherit one common parent environment. Non-profile RUM variables were not individually recorded; do not infer equality with the earlier session.
- Base A/A measures base variability; it does not bound all candidate-specific variance or establish a code-generation cause.
- Screen is exploratory and does not perform multiplicity correction across 112 labels; no statistical significance or general no-regression claim.
- Wall-clock engine elapsed is used for successful-run diagnostics only. Decisions use the original monotonic in-language timing rows.
- Same machine and old binaries; independent builds, hardware and representative user applications are not covered.
