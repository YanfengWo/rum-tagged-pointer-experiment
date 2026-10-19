# Scoped copy attribution and safe rewrite experiment

Exploratory rum-vs-rum evidence. Ratios below 1 mean less time. No whole-program equal-label average is computed.

Scoped hit probes at N=64/256/1024 confirm one shallow N-slot copy removed from Replace's parent traversal; N-1 two-slot normalization copies and one into_owned copy remain. These event counts do not measure speed or total allocation bytes.

Timing uses eight paired process blocks. Focus: eleven retained monotonic repetitions per process, reduced to one median before pairing. Original complete suite: generally min9; retained GC keeps the suite's original final-batch convention.

| Focus label | Candidate/fresh median [p10,p90] | Faster blocks | Original A/A | Fresh/original pair | Candidate/original pair |
| --- | --- | ---: | ---: | ---: | ---: |
| rewrite-hit-4000 | 0.934 [0.789,0.974] | 8/8 | 1.019 | 1.028 | 0.960 |
| rewrite-hit-16000 | 0.944 [0.847,0.969] | 8/8 | 1.012 | 1.014 | 0.958 |
| rewrite-identity-4000 | 0.972 [0.807,0.980] | 8/8 | 0.996 | 0.987 | 0.959 |
| rewrite-identity-16000 | 0.960 [0.928,0.993] | 7/8 | 1.017 | 0.981 | 0.954 |
| rewrite-miss-4000 | 0.911 [0.771,0.960] | 7/8 | 1.018 | 1.004 | 0.917 |
| rewrite-miss-16000 | 0.908 [0.881,0.926] | 8/8 | 1.008 | 0.994 | 0.901 |
| rewrite-compound-miss-4000 | 0.865 [0.704,0.893] | 8/8 | 1.029 | 1.002 | 0.867 |
| rewrite-compound-miss-16000 | 0.898 [0.814,0.915] | 8/8 | 0.996 | 0.998 | 0.899 |

All complete-suite labels, including regressions:

| Label | Candidate/fresh | Original A/A | Fresh/original pair | Candidate/original pair |
| --- | ---: | ---: | ---: | ---: |
| association-part-key-read-width4000 | 0.970 | 0.989 | 1.011 | 0.983 |
| association-part-key-read-width32000 | 0.973 | 0.999 | 0.996 | 0.970 |
| association-part-positional-read-width4000 | 0.998 | 0.999 | 1.007 | 1.005 |
| association-part-positional-read-width32000 | 1.008 | 1.001 | 0.997 | 1.004 |
| loop-empty | 0.983 | 1.018 | 0.969 | 0.961 |
| do-count-null | 0.972 | 1.017 | 0.992 | 0.979 |
| do-iterator-null | 0.968 | 0.976 | 1.019 | 0.991 |
| table-count-null | 0.985 | 1.004 | 0.997 | 0.974 |
| table-iterator-null | 0.982 | 1.009 | 1.008 | 0.970 |
| table-iterator-symbol | 0.991 | 1.030 | 0.948 | 1.012 |
| literal-integer | 0.968 | 1.037 | 0.948 | 0.976 |
| literal-string | 0.961 | 1.145 | 1.004 | 0.936 |
| symbol-ownvalue-read | 0.974 | 1.064 | 0.976 | 0.966 |
| symbol-ownvalue-guarded-read | 0.988 | 1.025 | 0.974 | 0.969 |
| set-ownvalue | 0.988 | 1.000 | 0.986 | 0.968 |
| list-literal-small | 0.990 | 1.020 | 0.986 | 0.985 |
| head-list | 1.007 | 0.995 | 0.991 | 0.993 |
| length-list | 0.998 | 0.995 | 0.993 | 0.992 |
| part-list | 1.001 | 1.007 | 0.999 | 0.994 |
| sameq-integer | 0.984 | 1.013 | 0.997 | 0.978 |
| sameq-list | 0.988 | 0.996 | 0.996 | 0.983 |
| if-true | 0.980 | 1.007 | 1.000 | 0.988 |
| module-local | 0.978 | 1.006 | 1.013 | 0.986 |
| module-chain-escape-small | 0.974 | 0.983 | 1.002 | 1.000 |
| module-chain-escape-large | 0.997 | 0.969 | 0.997 | 0.982 |
| block-local | 1.004 | 0.995 | 1.000 | 1.001 |
| block-symbol-only-repeat | 0.972 | 0.991 | 1.008 | 0.967 |
| block-init-null-repeat | 0.977 | 1.014 | 1.014 | 0.992 |
| block-init-read-repeat | 1.008 | 0.995 | 1.002 | 0.999 |
| with-local | 0.995 | 0.980 | 0.994 | 0.996 |
| table-small | 0.967 | 0.994 | 1.005 | 0.978 |
| table-iterator-plus | 0.987 | 1.006 | 0.999 | 0.976 |
| table-iterator-pair-list | 0.974 | 1.021 | 1.010 | 0.965 |
| do-small | 0.984 | 0.989 | 0.995 | 0.968 |
| downvalue-exact-hit | 1.005 | 1.002 | 0.997 | 0.992 |
| downvalue-exact-miss | 0.974 | 1.001 | 0.996 | 0.982 |
| downvalue-pattern-hit | 0.989 | 1.001 | 0.988 | 0.971 |
| downvalue-pattern-miss | 0.993 | 1.001 | 0.990 | 1.001 |
| downvalue-exact-miss-repeat | 0.974 | 0.991 | 1.009 | 0.990 |
| downvalue-pattern-miss-repeat | 0.971 | 1.010 | 1.015 | 0.994 |
| downvalue-guarded-hit | 0.978 | 0.993 | 1.003 | 0.981 |
| downvalue-guarded-decline-fallback | 0.998 | 1.003 | 1.008 | 1.008 |
| upvalue-guarded-hit | 0.988 | 1.004 | 0.992 | 0.977 |
| subvalue-unguarded-hit | 0.989 | 1.000 | 0.985 | 0.971 |
| subvalue-unguarded-miss | 0.986 | 1.019 | 1.016 | 0.999 |
| subvalue-guarded-hit | 0.991 | 0.992 | 0.995 | 0.985 |
| default-unguarded-hit | 1.007 | 0.997 | 0.984 | 0.998 |
| default-unguarded-miss | 1.012 | 1.006 | 0.985 | 0.997 |
| default-guarded-hit | 1.006 | 0.986 | 0.988 | 0.999 |
| matchq-integer-success | 1.006 | 0.995 | 0.999 | 0.989 |
| matchq-integer-fail | 1.002 | 1.003 | 1.005 | 1.002 |
| matcher-single-tiny | 0.991 | 0.997 | 1.003 | 0.993 |
| matcher-single-wide-small | 0.990 | 0.989 | 1.011 | 0.987 |
| matcher-single-wide-large | 1.004 | 0.993 | 0.987 | 0.992 |
| replaceall-simple | 0.974 | 1.000 | 0.999 | 0.983 |
| rewrite-guarded-hit | 0.988 | 1.000 | 0.994 | 0.977 |
| rewrite-guarded-decline-fallback | 0.986 | 0.993 | 0.993 | 0.976 |
| cases-wide-no-match | 1.023 | 0.977 | 0.996 | 1.020 |
| count-wide-no-match | 1.002 | 0.997 | 0.999 | 0.991 |
| position-wide-no-match | 0.999 | 0.987 | 1.001 | 0.997 |
| deletecases-wide-no-match | 0.997 | 1.008 | 0.998 | 0.993 |
| deletecases-wide-association-no-match | 1.012 | 1.013 | 0.979 | 0.956 |
| replace-wide-no-match | 0.903 | 1.003 | 1.006 | 0.907 |
| replacepart-wide-rule-list | 1.002 | 0.976 | 0.993 | 0.994 |
| replacepart-guarded-hit | 0.954 | 0.993 | 1.030 | 0.977 |
| replacepart-guarded-decline-fallback | 0.969 | 0.997 | 1.014 | 0.966 |
| string-repeat-small | 0.996 | 1.013 | 1.018 | 1.016 |
| character-range-small | 1.015 | 0.992 | 1.006 | 0.992 |
| stringtrim-literal-list-4000 | 0.965 | 1.012 | 1.001 | 0.972 |
| stringtrim-literal-list-32000 | 0.993 | 1.008 | 1.005 | 0.980 |
| stringtrim-single-literal-4000 | 0.990 | 1.004 | 0.998 | 0.984 |
| stringtrim-single-literal-32000 | 0.992 | 0.998 | 0.996 | 0.990 |
| stringtrim-whitespace-4000 | 0.982 | 0.994 | 1.000 | 0.986 |
| stringtrim-whitespace-32000 | 0.982 | 0.977 | 1.000 | 0.996 |
| stringreplace-single-literal-unguarded | 1.002 | 0.995 | 0.984 | 0.987 |
| stringreplace-pattern-unguarded | 0.997 | 0.998 | 1.010 | 0.996 |
| stringreplace-guarded-hit | 1.002 | 1.008 | 0.981 | 0.972 |
| stringreplace-guarded-decline-fallback | 0.984 | 1.006 | 0.987 | 0.970 |
| appendto-list-incremental-small | 1.013 | 0.981 | 0.992 | 0.998 |
| appendto-list-incremental-large | 0.995 | 0.999 | 1.000 | 0.997 |
| appendto-part-outer1000-small | 0.999 | 1.002 | 1.004 | 0.998 |
| appendto-part-outer1000-large | 0.999 | 0.997 | 1.001 | 1.000 |
| association-set-key-incremental-small | 0.987 | 1.006 | 1.003 | 0.998 |
| association-set-key-incremental-large | 0.982 | 1.011 | 1.006 | 0.990 |
| association-small-literal-repeat | 0.998 | 1.001 | 1.036 | 1.007 |
| set-part-fixed100-len100 | 1.012 | 0.988 | 0.992 | 1.025 |
| set-part-fixed100-len800 | 1.022 | 0.984 | 1.015 | 1.014 |
| set-part-fill-small | 1.013 | 1.003 | 1.003 | 1.026 |
| set-part-fill-large | 1.030 | 0.988 | 0.993 | 1.026 |
| upvalue-candidates-tiny | 0.987 | 0.993 | 0.999 | 0.977 |
| upvalue-candidates-wide-small | 0.999 | 1.006 | 0.986 | 0.984 |
| upvalue-candidates-wide-large | 0.993 | 1.000 | 0.989 | 0.987 |
| sort-callback-small | 0.972 | 0.991 | 1.007 | 0.990 |
| sort-callback-large | 0.967 | 0.999 | 1.024 | 0.990 |
| sortby-callback-small | 0.979 | 1.004 | 1.025 | 0.990 |
| sortby-callback-large | 0.981 | 0.987 | 1.015 | 0.990 |
| reversesort-callback-small | 0.986 | 1.015 | 1.005 | 0.992 |
| reversesort-callback-large | 0.983 | 0.993 | 1.004 | 0.989 |
| keysort-callback-small | 0.983 | 0.999 | 0.999 | 0.977 |
| keysort-callback-large | 0.989 | 0.989 | 0.993 | 0.986 |
| keysortby-callback-small | 0.990 | 1.004 | 1.008 | 0.995 |
| keysortby-callback-large | 0.995 | 0.989 | 1.000 | 0.995 |
| sort-default-wide | 1.023 | 0.997 | 1.003 | 1.028 |
| sort-symbol-default-width4000 | 1.009 | 0.989 | 0.959 | 0.965 |
| sort-symbol-default-width16000 | 1.004 | 0.996 | 0.997 | 0.995 |
| sortby-default-wide | 0.988 | 0.999 | 0.998 | 0.985 |
| reversesort-default-wide | 1.017 | 1.003 | 1.002 | 1.016 |
| keysort-default-wide | 0.962 | 1.040 | 1.042 | 0.992 |
| keysortby-default-wide | 0.975 | 1.009 | 1.012 | 0.982 |
| valueq-undefined-symbol-definitions-2000 | 0.983 | 0.992 | 1.003 | 0.996 |
| valueq-undefined-symbol-definitions-8000 | 0.988 | 1.000 | 0.987 | 0.976 |
| module-retained-gensym-gc-batch | 1.021 | 1.049 | 1.022 | 0.992 |

Limits:

- Eight paired process blocks, not 88 independent focused repetitions. p10/p90 describe eight observed ratios; they are not confidence intervals.
- Single machine/toolchain and semantic micro workloads; representative teacher applications remain unmeasured.
- Position-balanced rotation and reversal do not remove all within-block drift, carryover or binary-specific variance. Original A/A is not a bound on every candidate's noise.
- All labels are retained. Exploratory comparisons across many labels do not establish statistical significance or a general no-regression claim.
- Matched source-root builds reuse dependency artifacts. A fresh baseline controls rebuilding; unexplained rebuild effects cannot be attributed to a source optimization.
- Diagnostics use separate instrumented binaries and fresh-process controls. Shallow slot bytes omit recursive ownership work, allocator overhead and total memory traffic.
- Copy traces only cover the selected ownership operations. Remaining two-slot copies arise in normalization; no complete Expr clone census or hardware profile is claimed.
- The candidate is isolated and safe Rust; default kernel source bytes are unchanged. Passing semantic tests is not a general semantic-equivalence proof.
- Initial candidate workspace failed two Jupyter pump checks; the isolated five-check retry and complete rerun passed. The initial failure's root cause is unestablished and its log is retained.
