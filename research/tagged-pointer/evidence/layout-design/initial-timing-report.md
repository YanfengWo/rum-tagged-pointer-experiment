# Layout operation timing report

Validated 7,500 rows; round 0 excluded, leaving 27 samples per representation and operation.
Ratios above 1 mean slower. All ratios are medians of within-run, within-round pairs.
Construction measures fresh alias-preserving conversion and normal release, not an evaluator workload.
Drop uses a separately constructed fresh graph outside each timer; it includes per-sample timer overhead.
Ranges and per-run medians in the JSON are descriptive; these runs do not establish statistical significance.

| Dataset | Operation | Original ms/iteration | Safe24 / original | Tagged16 / original | Tagged8 / original | Tagged16 / previous | Tagged8 / previous |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| gMicroWideTraversal | scan | 0.001631667 | 1.934 | 1.981 | 2.113 | 1.169 | 1.246 |
| gMicroWideTraversal | build_drop | 0.015874500 | 0.821 | 0.807 | 0.591 | 1.260 | 0.928 |
| gMicroWideTraversal | cow | 0.008417500 | 0.417 | 0.391 | 0.199 | 1.869 | 0.958 |
| gMicroWideTraversal | clone_drop | 0.000015795 | 0.991 | 1.023 | 0.570 | 1.815 | 1.005 |
| gMicroWideTraversal | drop | 0.002048000 | 0.699 | 0.697 | 0.255 | 2.898 | 1.056 |
| gMicroSortSymbolWidth16000 | scan | 0.021043667 | 4.573 | 4.597 | 4.595 | 1.044 | 1.027 |
| gMicroSortSymbolWidth16000 | build_drop | 0.128132000 | 0.886 | 0.873 | 0.574 | 0.214 | 0.138 |
| gMicroSortSymbolWidth16000 | cow | 0.035565500 | 0.770 | 0.653 | 0.244 | 0.196 | 0.066 |
| gMicroSortSymbolWidth16000 | clone_drop | 0.000015795 | 0.991 | 1.023 | 0.570 | 1.818 | 1.015 |
| gMicroSortSymbolWidth16000 | drop | 0.015709500 | 0.816 | 0.813 | 0.208 | 0.053 | 0.014 |
| gMicroAssociationPartWidth32000 | scan | 0.145978333 | 1.967 | 2.000 | 1.987 | 1.047 | 1.031 |
| gMicroAssociationPartWidth32000 | build_drop | 7.422874500 | 0.981 | 0.916 | 1.024 | 0.877 | 0.974 |
| gMicroAssociationPartWidth32000 | cow | 0.649063500 | 0.919 | 0.936 | 0.426 | 1.939 | 0.991 |
| gMicroAssociationPartWidth32000 | clone_drop | 0.000015750 | 0.991 | 1.023 | 0.570 | 1.805 | 1.010 |
| gMicroAssociationPartWidth32000 | drop | 1.210157500 | 0.882 | 0.936 | 1.273 | 0.684 | 0.935 |
| gMicroReplacePartRules | scan | 0.011905667 | 2.100 | 2.106 | 2.091 | 1.026 | 1.020 |
| gMicroReplacePartRules | build_drop | 0.277020000 | 1.091 | 1.072 | 1.224 | 0.784 | 0.926 |
| gMicroReplacePartRules | cow | 0.026216000 | 0.990 | 0.954 | 0.480 | 1.961 | 0.992 |
| gMicroReplacePartRules | clone_drop | 0.000015750 | 0.988 | 1.023 | 0.568 | 1.814 | 1.006 |
| gMicroReplacePartRules | drop | 0.049213000 | 1.168 | 1.160 | 1.627 | 0.704 | 0.935 |
| rulesSimple | scan | 0.000050695 | 1.522 | 1.521 | 1.491 | 1.005 | 0.995 |
| rulesSimple | build_drop | 0.000624475 | 0.993 | 0.974 | 0.961 | 0.691 | 0.681 |
| rulesSimple | cow | 0.000104470 | 1.047 | 0.958 | 0.844 | 1.121 | 0.987 |
| rulesSimple | clone_drop | 0.000015750 | 0.991 | 1.023 | 0.571 | 1.814 | 1.010 |
| rulesSimple | drop | 0.000111180 | 1.000 | 1.008 | 1.323 | 0.541 | 0.715 |
| synthetic-repeated-symbol-16000 | scan | 0.020924667 | 4.442 | 4.403 | 4.388 | 1.002 | 1.000 |
| synthetic-repeated-symbol-16000 | build_drop | 0.129009000 | 0.896 | 0.879 | 0.579 | 0.559 | 0.366 |
| synthetic-repeated-symbol-16000 | cow | 0.036718000 | 0.770 | 0.653 | 0.248 | 0.195 | 0.076 |
| synthetic-repeated-symbol-16000 | clone_drop | 0.000015750 | 0.989 | 1.023 | 0.568 | 1.809 | 1.005 |
| synthetic-repeated-symbol-16000 | drop | 0.015618000 | 0.816 | 0.818 | 0.208 | 0.214 | 0.055 |
| synthetic-generated-symbol-16000 | scan | 0.020942667 | 4.404 | 4.413 | 4.423 | 0.995 | 0.989 |
| synthetic-generated-symbol-16000 | build_drop | 0.128472000 | 0.895 | 0.872 | 3.303 | 0.209 | 0.786 |
| synthetic-generated-symbol-16000 | cow | 0.036598500 | 0.788 | 0.625 | 3.250 | 0.193 | 0.998 |
| synthetic-generated-symbol-16000 | clone_drop | 0.000015750 | 0.991 | 1.023 | 0.568 | 1.813 | 1.005 |
| synthetic-generated-symbol-16000 | drop | 0.015631500 | 0.816 | 0.815 | 15.259 | 0.053 | 1.006 |
| synthetic-strings-16000 | scan | 0.025265333 | 1.722 | 1.856 | 1.745 | 1.064 | 0.999 |
| synthetic-strings-16000 | build_drop | 0.355682000 | 1.915 | 1.856 | 1.749 | 1.073 | 0.986 |
| synthetic-strings-16000 | cow | 0.291122500 | 0.474 | 0.445 | 0.421 | 1.076 | 1.006 |
| synthetic-strings-16000 | clone_drop | 0.000015750 | 0.991 | 1.026 | 0.567 | 1.818 | 1.005 |
| synthetic-strings-16000 | drop | 0.127916500 | 2.504 | 2.548 | 2.586 | 1.014 | 1.021 |
| synthetic-deep-256 | scan | 0.001824333 | 1.602 | 1.557 | 1.708 | 0.902 | 1.017 |
| synthetic-deep-256 | build_drop | 0.040640000 | 1.045 | 1.033 | 1.132 | 0.854 | 0.946 |
| synthetic-deep-256 | cow | 0.000978000 | 0.829 | 0.768 | 0.758 | 1.006 | 0.963 |
| synthetic-deep-256 | clone_drop | 0.000015750 | 0.988 | 1.023 | 0.567 | 1.817 | 1.011 |
| synthetic-deep-256 | drop | 0.005979000 | 0.988 | 1.207 | 1.486 | 0.722 | 0.892 |
| synthetic-shared-args-200 | scan | 0.164900667 | 2.334 | 2.492 | 2.265 | 1.515 | 1.350 |
| synthetic-shared-args-200 | build_drop | 0.018221000 | 0.904 | 0.893 | 0.957 | 0.869 | 0.935 |
| synthetic-shared-args-200 | cow | 0.003977000 | 0.984 | 0.954 | 0.597 | 1.583 | 0.992 |
| synthetic-shared-args-200 | clone_drop | 0.000015750 | 0.991 | 1.023 | 0.567 | 1.815 | 1.010 |
| synthetic-shared-args-200 | drop | 0.003063000 | 0.903 | 0.858 | 1.242 | 0.690 | 0.996 |
