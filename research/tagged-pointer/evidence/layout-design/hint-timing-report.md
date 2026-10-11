# Layout operation timing report

Validated 7,500 rows; round 0 excluded, leaving 27 samples per representation and operation.
Ratios above 1 mean slower. All ratios are medians of within-run, within-round pairs.
Construction measures fresh alias-preserving conversion and normal release, not an evaluator workload.
Drop uses a separately constructed fresh graph outside each timer; it includes per-sample timer overhead.
Ranges and per-run medians in the JSON are descriptive; these runs do not establish statistical significance.

| Dataset | Operation | Original ms/iteration | Safe24 / original | Tagged16 / original | Tagged8 / original | Tagged16 / previous | Tagged8 / previous |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| gMicroWideTraversal | scan | 0.001622333 | 1.912 | 1.977 | 1.836 | 1.173 | 1.109 |
| gMicroWideTraversal | build_drop | 0.015287000 | 0.829 | 0.814 | 0.594 | 1.265 | 0.929 |
| gMicroWideTraversal | cow | 0.007540000 | 0.439 | 0.419 | 0.205 | 1.943 | 0.966 |
| gMicroWideTraversal | clone_drop | 0.000015705 | 0.991 | 1.023 | 0.570 | 1.813 | 1.010 |
| gMicroWideTraversal | drop | 0.002016000 | 0.718 | 0.704 | 0.249 | 2.897 | 1.029 |
| gMicroSortSymbolWidth16000 | scan | 0.020823333 | 4.511 | 4.468 | 4.474 | 1.010 | 1.006 |
| gMicroSortSymbolWidth16000 | build_drop | 0.127640500 | 0.897 | 0.872 | 0.569 | 0.212 | 0.139 |
| gMicroSortSymbolWidth16000 | cow | 0.034509000 | 0.790 | 0.708 | 0.254 | 0.192 | 0.072 |
| gMicroSortSymbolWidth16000 | clone_drop | 0.000015705 | 0.991 | 1.024 | 0.569 | 1.811 | 1.000 |
| gMicroSortSymbolWidth16000 | drop | 0.015622000 | 0.815 | 0.816 | 0.207 | 0.053 | 0.014 |
| gMicroAssociationPartWidth32000 | scan | 0.142001667 | 2.015 | 2.089 | 2.030 | 1.045 | 1.040 |
| gMicroAssociationPartWidth32000 | build_drop | 7.330887000 | 0.980 | 0.925 | 1.026 | 0.886 | 0.967 |
| gMicroAssociationPartWidth32000 | cow | 0.636776000 | 0.915 | 0.906 | 0.447 | 2.000 | 1.025 |
| gMicroAssociationPartWidth32000 | clone_drop | 0.000015800 | 0.988 | 1.023 | 0.567 | 1.814 | 1.010 |
| gMicroAssociationPartWidth32000 | drop | 1.211498500 | 0.869 | 0.931 | 1.263 | 0.679 | 0.929 |
| gMicroReplacePartRules | scan | 0.011823000 | 2.098 | 2.106 | 2.109 | 1.033 | 1.025 |
| gMicroReplacePartRules | build_drop | 0.276175500 | 1.108 | 1.046 | 1.214 | 0.817 | 0.943 |
| gMicroReplacePartRules | cow | 0.025701500 | 0.977 | 0.962 | 0.484 | 1.962 | 0.997 |
| gMicroReplacePartRules | clone_drop | 0.000015750 | 0.991 | 1.023 | 0.567 | 1.813 | 1.010 |
| gMicroReplacePartRules | drop | 0.049052500 | 1.177 | 1.011 | 1.623 | 0.582 | 0.936 |
| rulesSimple | scan | 0.000050470 | 1.529 | 1.514 | 1.498 | 1.002 | 0.996 |
| rulesSimple | build_drop | 0.000560045 | 1.094 | 1.085 | 1.070 | 0.721 | 0.709 |
| rulesSimple | cow | 0.000103780 | 1.045 | 0.963 | 0.847 | 1.118 | 0.983 |
| rulesSimple | clone_drop | 0.000015705 | 0.991 | 1.023 | 0.569 | 1.810 | 1.006 |
| rulesSimple | drop | 0.000108490 | 1.005 | 1.015 | 1.346 | 0.538 | 0.715 |
| synthetic-repeated-symbol-16000 | scan | 0.020796000 | 4.406 | 4.414 | 4.436 | 1.000 | 0.999 |
| synthetic-repeated-symbol-16000 | build_drop | 0.127604000 | 0.898 | 0.879 | 0.575 | 0.557 | 0.365 |
| synthetic-repeated-symbol-16000 | cow | 0.032126000 | 0.844 | 0.719 | 0.247 | 0.195 | 0.064 |
| synthetic-repeated-symbol-16000 | clone_drop | 0.000015705 | 0.991 | 1.023 | 0.570 | 1.805 | 1.005 |
| synthetic-repeated-symbol-16000 | drop | 0.015608500 | 0.816 | 0.816 | 0.207 | 0.214 | 0.054 |
| synthetic-generated-symbol-16000 | scan | 0.020777333 | 4.405 | 4.394 | 4.426 | 0.995 | 1.001 |
| synthetic-generated-symbol-16000 | build_drop | 0.125197500 | 0.895 | 0.873 | 3.346 | 0.210 | 0.812 |
| synthetic-generated-symbol-16000 | cow | 0.034417500 | 0.880 | 0.739 | 3.392 | 0.194 | 0.999 |
| synthetic-generated-symbol-16000 | clone_drop | 0.000015705 | 0.991 | 1.024 | 0.567 | 1.818 | 1.010 |
| synthetic-generated-symbol-16000 | drop | 0.015599500 | 0.817 | 0.816 | 15.703 | 0.054 | 1.017 |
| synthetic-strings-16000 | scan | 0.024870667 | 1.684 | 1.860 | 1.752 | 1.056 | 1.005 |
| synthetic-strings-16000 | build_drop | 0.343389500 | 1.877 | 1.884 | 1.723 | 1.066 | 0.963 |
| synthetic-strings-16000 | cow | 0.294121000 | 0.469 | 0.434 | 0.407 | 1.060 | 0.999 |
| synthetic-strings-16000 | clone_drop | 0.000015795 | 0.989 | 1.023 | 0.567 | 1.814 | 1.005 |
| synthetic-strings-16000 | drop | 0.126980000 | 2.528 | 2.574 | 2.527 | 1.016 | 0.995 |
| synthetic-deep-256 | scan | 0.001708000 | 1.595 | 1.590 | 1.815 | 0.893 | 1.028 |
| synthetic-deep-256 | build_drop | 0.040300000 | 1.044 | 1.043 | 1.116 | 0.866 | 0.919 |
| synthetic-deep-256 | cow | 0.000937000 | 0.836 | 0.736 | 0.736 | 0.948 | 0.950 |
| synthetic-deep-256 | clone_drop | 0.000015705 | 0.989 | 1.023 | 0.569 | 1.818 | 1.010 |
| synthetic-deep-256 | drop | 0.005942000 | 0.992 | 1.205 | 1.492 | 0.722 | 0.892 |
| synthetic-shared-args-200 | scan | 0.161668000 | 2.518 | 2.624 | 2.324 | 1.589 | 1.394 |
| synthetic-shared-args-200 | build_drop | 0.017716000 | 0.922 | 0.907 | 0.972 | 0.890 | 0.946 |
| synthetic-shared-args-200 | cow | 0.003977000 | 0.980 | 0.966 | 0.595 | 1.597 | 0.989 |
| synthetic-shared-args-200 | clone_drop | 0.000015705 | 0.989 | 1.023 | 0.566 | 1.823 | 1.010 |
| synthetic-shared-args-200 | drop | 0.003040000 | 0.911 | 0.863 | 1.247 | 0.692 | 0.998 |
