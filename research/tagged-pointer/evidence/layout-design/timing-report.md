# Layout operation timing report

Validated 7,500 rows; round 0 excluded, leaving 27 samples per representation and operation.
Ratios above 1 mean slower. All ratios are medians of within-run, within-round pairs.
Construction measures fresh alias-preserving conversion and normal release, not an evaluator workload.
Drop uses a separately constructed fresh graph outside each timer; it includes per-sample timer overhead.
Ranges and per-run medians in the JSON are descriptive; these runs do not establish statistical significance.

| Dataset | Operation | Original ms/iteration | Safe24 / original | Tagged16 / original | Tagged8 / original | Tagged16 / previous | Tagged8 / previous |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| gMicroWideTraversal | scan | 0.001625667 | 1.115 | 1.005 | 0.945 | 1.225 | 1.147 |
| gMicroWideTraversal | build_drop | 0.015484500 | 0.818 | 0.863 | 0.601 | 1.187 | 0.828 |
| gMicroWideTraversal | cow | 0.008417000 | 0.426 | 0.418 | 0.205 | 2.078 | 0.964 |
| gMicroWideTraversal | clone_drop | 0.000015795 | 0.985 | 1.020 | 0.573 | 1.800 | 1.010 |
| gMicroWideTraversal | drop | 0.002048000 | 0.779 | 0.837 | 0.251 | 3.362 | 1.009 |
| gMicroSortSymbolWidth16000 | scan | 0.020933667 | 1.705 | 1.733 | 1.617 | 1.516 | 1.410 |
| gMicroSortSymbolWidth16000 | build_drop | 0.130882500 | 0.855 | 0.906 | 0.557 | 0.223 | 0.139 |
| gMicroSortSymbolWidth16000 | cow | 0.037742000 | 0.762 | 0.679 | 0.197 | 0.218 | 0.062 |
| gMicroSortSymbolWidth16000 | clone_drop | 0.000015840 | 0.986 | 1.024 | 0.570 | 1.805 | 1.005 |
| gMicroSortSymbolWidth16000 | drop | 0.018409500 | 0.771 | 0.846 | 0.173 | 0.065 | 0.013 |
| gMicroAssociationPartWidth32000 | scan | 0.138998333 | 1.076 | 1.082 | 1.059 | 1.146 | 1.096 |
| gMicroAssociationPartWidth32000 | build_drop | 7.319112500 | 0.978 | 0.915 | 1.014 | 0.871 | 0.967 |
| gMicroAssociationPartWidth32000 | cow | 0.634401500 | 0.940 | 0.911 | 0.404 | 2.031 | 0.994 |
| gMicroAssociationPartWidth32000 | clone_drop | 0.000015845 | 0.985 | 1.023 | 0.569 | 1.801 | 1.010 |
| gMicroAssociationPartWidth32000 | drop | 1.205850000 | 0.873 | 0.933 | 1.266 | 0.684 | 0.928 |
| gMicroReplacePartRules | scan | 0.011841333 | 1.103 | 1.111 | 1.087 | 1.062 | 1.048 |
| gMicroReplacePartRules | build_drop | 0.280565500 | 1.063 | 1.029 | 1.188 | 0.807 | 0.936 |
| gMicroReplacePartRules | cow | 0.025711000 | 0.978 | 0.955 | 0.482 | 1.974 | 0.998 |
| gMicroReplacePartRules | clone_drop | 0.000015845 | 0.985 | 1.020 | 0.570 | 1.800 | 1.006 |
| gMicroReplacePartRules | drop | 0.048987500 | 1.184 | 1.017 | 1.624 | 0.588 | 0.935 |
| rulesSimple | scan | 0.000049965 | 1.067 | 1.061 | 1.086 | 1.061 | 1.009 |
| rulesSimple | build_drop | 0.000561975 | 1.067 | 1.071 | 1.093 | 0.656 | 0.672 |
| rulesSimple | cow | 0.000104515 | 1.079 | 0.962 | 0.842 | 1.125 | 0.986 |
| rulesSimple | clone_drop | 0.000015845 | 0.985 | 1.023 | 0.570 | 1.811 | 1.010 |
| rulesSimple | drop | 0.000108810 | 1.002 | 1.012 | 1.343 | 0.539 | 0.715 |
| synthetic-repeated-symbol-16000 | scan | 0.020820333 | 1.184 | 1.307 | 1.120 | 1.225 | 1.049 |
| synthetic-repeated-symbol-16000 | build_drop | 0.133289000 | 0.845 | 0.902 | 0.560 | 0.596 | 0.366 |
| synthetic-repeated-symbol-16000 | cow | 0.038243000 | 0.750 | 0.665 | 0.230 | 0.217 | 0.071 |
| synthetic-repeated-symbol-16000 | clone_drop | 0.000015795 | 0.988 | 1.020 | 0.572 | 1.804 | 1.010 |
| synthetic-repeated-symbol-16000 | drop | 0.018391000 | 0.770 | 0.846 | 0.173 | 0.264 | 0.054 |
| synthetic-generated-symbol-16000 | scan | 0.020832667 | 1.199 | 1.336 | 1.327 | 1.172 | 1.163 |
| synthetic-generated-symbol-16000 | build_drop | 0.127875000 | 0.871 | 0.919 | 3.251 | 0.224 | 0.795 |
| synthetic-generated-symbol-16000 | cow | 0.034743500 | 0.836 | 0.753 | 3.332 | 0.217 | 0.999 |
| synthetic-generated-symbol-16000 | clone_drop | 0.000015840 | 0.983 | 1.021 | 0.571 | 1.800 | 1.015 |
| synthetic-generated-symbol-16000 | drop | 0.018404500 | 0.770 | 0.845 | 13.139 | 0.066 | 1.003 |
| synthetic-strings-16000 | scan | 0.024500000 | 1.314 | 1.260 | 1.264 | 1.066 | 1.081 |
| synthetic-strings-16000 | build_drop | 0.346167500 | 1.860 | 1.885 | 1.731 | 1.061 | 0.966 |
| synthetic-strings-16000 | cow | 0.285777500 | 0.466 | 0.441 | 0.412 | 1.063 | 0.997 |
| synthetic-strings-16000 | clone_drop | 0.000015845 | 0.985 | 1.023 | 0.570 | 1.805 | 1.006 |
| synthetic-strings-16000 | drop | 0.126695000 | 2.565 | 2.628 | 2.537 | 1.022 | 0.996 |
| synthetic-deep-256 | scan | 0.001708333 | 1.011 | 1.031 | 1.287 | 0.909 | 1.073 |
| synthetic-deep-256 | build_drop | 0.040263000 | 1.041 | 1.071 | 1.133 | 0.880 | 0.955 |
| synthetic-deep-256 | cow | 0.000960000 | 0.846 | 0.767 | 0.746 | 0.947 | 0.939 |
| synthetic-deep-256 | clone_drop | 0.000015795 | 0.986 | 1.021 | 0.571 | 1.809 | 1.011 |
| synthetic-deep-256 | drop | 0.005983500 | 0.988 | 1.215 | 1.508 | 0.734 | 0.911 |
| synthetic-shared-args-200 | scan | 0.161946333 | 1.019 | 0.803 | 0.765 | 1.394 | 1.291 |
| synthetic-shared-args-200 | build_drop | 0.017858500 | 0.909 | 0.944 | 0.979 | 0.881 | 0.922 |
| synthetic-shared-args-200 | cow | 0.004041000 | 0.978 | 0.969 | 0.600 | 1.568 | 0.980 |
| synthetic-shared-args-200 | clone_drop | 0.000015840 | 0.988 | 1.023 | 0.571 | 1.805 | 1.006 |
| synthetic-shared-args-200 | drop | 0.002994000 | 0.941 | 0.885 | 1.268 | 0.694 | 0.995 |
