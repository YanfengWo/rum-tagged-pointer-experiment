# Typed payload operation timing report

Validated 11,700 rows; round 0 excluded, leaving 27 samples per representation and operation.
Ratios above 1 mean slower. All ratios are medians of within-run, within-round pairs.
Construction measures fresh alias-preserving conversion and normal release, not an evaluator workload.
Drop uses a separately constructed fresh graph outside each timer; it includes per-sample timer overhead.
Ranges and per-run medians in the JSON are descriptive; these runs do not establish statistical significance.

| Dataset | Operation | Original ms/iteration | Safe24 / original | Typed16 move / original | Typed16 borrow / original | Typed16 borrow / previous | Typed16 borrow / Safe24 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| gMicroWideTraversal | scan_full | 0.002026667 | 0.886 | 0.877 | 0.873 | 0.970 | 0.986 |
| gMicroWideTraversal | scan_key | 0.002035667 | 0.775 | 0.786 | 0.787 | 0.890 | 1.014 |
| gMicroWideTraversal | build_drop | 0.012743000 | 1.077 | 1.089 | 1.076 | 1.077 | 0.999 |
| gMicroWideTraversal | cow | 0.008311500 | 0.463 | 0.453 | 0.449 | 1.060 | 0.982 |
| gMicroWideTraversal | clone_drop | 0.000015935 | 0.974 | 1.046 | 1.041 | 1.020 | 1.068 |
| gMicroWideTraversal | drop | 0.002071000 | 0.854 | 0.858 | 0.854 | 1.046 | 1.000 |
| gMicroSortSymbolWidth16000 | scan_full | 0.021166333 | 1.549 | 1.823 | 1.775 | 0.899 | 1.141 |
| gMicroSortSymbolWidth16000 | scan_key | 0.021187667 | 0.532 | 0.535 | 0.530 | 0.288 | 0.995 |
| gMicroSortSymbolWidth16000 | build_drop | 0.102375000 | 1.212 | 1.231 | 1.212 | 1.082 | 1.006 |
| gMicroSortSymbolWidth16000 | cow | 0.038669500 | 0.806 | 0.737 | 0.741 | 1.083 | 0.946 |
| gMicroSortSymbolWidth16000 | clone_drop | 0.000015890 | 0.977 | 1.049 | 1.041 | 1.017 | 1.062 |
| gMicroSortSymbolWidth16000 | drop | 0.015806000 | 0.996 | 0.996 | 0.994 | 1.005 | 0.999 |
| gMicroAssociationPartWidth32000 | scan_full | 0.172428667 | 0.904 | 0.924 | 0.925 | 0.961 | 0.952 |
| gMicroAssociationPartWidth32000 | scan_key | 0.160936000 | 0.805 | 0.790 | 0.747 | 0.755 | 0.986 |
| gMicroAssociationPartWidth32000 | build_drop | 6.706542000 | 0.904 | 0.862 | 0.874 | 0.999 | 0.973 |
| gMicroAssociationPartWidth32000 | cow | 0.562531000 | 0.976 | 0.901 | 0.902 | 0.988 | 0.937 |
| gMicroAssociationPartWidth32000 | clone_drop | 0.000015935 | 0.983 | 1.058 | 1.041 | 1.020 | 1.050 |
| gMicroAssociationPartWidth32000 | drop | 1.225573000 | 0.876 | 0.939 | 0.943 | 1.003 | 1.073 |
| gMicroReplacePartRules | scan_full | 0.012196667 | 0.983 | 1.079 | 1.080 | 0.977 | 1.101 |
| gMicroReplacePartRules | scan_key | 0.012221000 | 0.782 | 0.823 | 0.823 | 0.739 | 1.041 |
| gMicroReplacePartRules | build_drop | 0.279922500 | 1.005 | 1.026 | 1.015 | 1.023 | 1.046 |
| gMicroReplacePartRules | cow | 0.026882000 | 0.981 | 0.953 | 0.948 | 0.994 | 0.971 |
| gMicroReplacePartRules | clone_drop | 0.000015930 | 0.974 | 1.044 | 1.041 | 1.017 | 1.069 |
| gMicroReplacePartRules | drop | 0.049231000 | 1.027 | 1.242 | 1.238 | 1.001 | 1.173 |
| rulesSimple | scan_full | 0.000051155 | 0.992 | 1.020 | 1.015 | 0.972 | 1.031 |
| rulesSimple | scan_key | 0.000051110 | 0.875 | 0.896 | 0.896 | 0.845 | 1.021 |
| rulesSimple | build_drop | 0.000568585 | 1.041 | 1.000 | 0.994 | 0.963 | 0.963 |
| rulesSimple | cow | 0.000111495 | 0.997 | 0.929 | 0.901 | 0.992 | 0.903 |
| rulesSimple | clone_drop | 0.000015890 | 0.977 | 1.046 | 1.041 | 1.014 | 1.065 |
| rulesSimple | drop | 0.000109545 | 1.007 | 1.018 | 1.020 | 1.011 | 1.011 |
| synthetic-repeated-symbol-16000 | scan_full | 0.021028333 | 1.028 | 1.219 | 1.224 | 0.935 | 1.185 |
| synthetic-repeated-symbol-16000 | scan_key | 0.021062000 | 0.521 | 0.534 | 0.535 | 0.405 | 1.026 |
| synthetic-repeated-symbol-16000 | build_drop | 0.102099500 | 1.204 | 1.225 | 1.206 | 1.057 | 1.002 |
| synthetic-repeated-symbol-16000 | cow | 0.038040500 | 0.816 | 0.777 | 0.824 | 1.083 | 0.950 |
| synthetic-repeated-symbol-16000 | clone_drop | 0.000015890 | 0.977 | 1.046 | 1.040 | 1.017 | 1.065 |
| synthetic-repeated-symbol-16000 | drop | 0.015745500 | 0.998 | 0.995 | 0.992 | 1.004 | 0.990 |
| synthetic-generated-symbol-16000 | scan_full | 0.020995000 | 1.130 | 1.220 | 1.222 | 0.921 | 1.081 |
| synthetic-generated-symbol-16000 | scan_key | 0.021037667 | 0.588 | 0.640 | 0.639 | 0.488 | 1.088 |
| synthetic-generated-symbol-16000 | build_drop | 0.101962000 | 1.231 | 1.243 | 1.202 | 1.073 | 0.997 |
| synthetic-generated-symbol-16000 | cow | 0.036057000 | 0.866 | 0.787 | 0.762 | 1.076 | 0.905 |
| synthetic-generated-symbol-16000 | clone_drop | 0.000015890 | 0.977 | 1.046 | 1.041 | 1.017 | 1.068 |
| synthetic-generated-symbol-16000 | drop | 0.015783000 | 1.002 | 0.994 | 0.996 | 1.006 | 0.999 |
| synthetic-strings-16000 | scan_full | 0.025599000 | 1.117 | 1.045 | 1.029 | 0.812 | 0.929 |
| synthetic-strings-16000 | scan_key | 0.025265333 | 1.018 | 1.038 | 1.046 | 0.798 | 1.021 |
| synthetic-strings-16000 | build_drop | 0.335518500 | 1.318 | 1.680 | 1.330 | 0.656 | 1.008 |
| synthetic-strings-16000 | cow | 0.298322000 | 0.472 | 0.442 | 0.437 | 1.004 | 0.924 |
| synthetic-strings-16000 | clone_drop | 0.000015890 | 0.977 | 1.047 | 1.041 | 1.017 | 1.065 |
| synthetic-strings-16000 | drop | 0.135089500 | 1.739 | 1.759 | 1.798 | 0.707 | 1.022 |
| synthetic-deep-256 | scan_full | 0.001858333 | 0.965 | 0.994 | 0.974 | 0.907 | 1.009 |
| synthetic-deep-256 | scan_key | 0.001974667 | 0.828 | 0.830 | 0.845 | 0.784 | 0.977 |
| synthetic-deep-256 | build_drop | 0.040212500 | 0.981 | 1.021 | 1.000 | 1.003 | 1.026 |
| synthetic-deep-256 | cow | 0.000973500 | 0.849 | 0.802 | 0.826 | 1.037 | 0.971 |
| synthetic-deep-256 | clone_drop | 0.000015935 | 0.974 | 1.046 | 1.038 | 1.014 | 1.066 |
| synthetic-deep-256 | drop | 0.006029500 | 0.991 | 1.232 | 1.230 | 1.034 | 1.243 |
| synthetic-shared-args-200 | scan_full | 0.196500000 | 0.836 | 0.856 | 0.835 | 0.999 | 1.010 |
| synthetic-shared-args-200 | scan_key | 0.196246000 | 0.770 | 0.830 | 0.838 | 0.993 | 1.117 |
| synthetic-shared-args-200 | build_drop | 0.016127500 | 1.028 | 1.034 | 1.035 | 1.053 | 1.004 |
| synthetic-shared-args-200 | cow | 0.004050500 | 0.972 | 0.979 | 0.969 | 1.008 | 0.989 |
| synthetic-shared-args-200 | clone_drop | 0.000015890 | 0.977 | 1.049 | 1.041 | 1.017 | 1.068 |
| synthetic-shared-args-200 | drop | 0.003123000 | 0.932 | 0.985 | 0.984 | 1.164 | 1.056 |
| synthetic-manual-big-small-16000 | scan_full | 0.469403333 | 0.985 | 1.005 | 0.988 | 0.965 | 1.005 |
| synthetic-manual-big-small-16000 | scan_key | 0.470749333 | 1.010 | 0.998 | 1.001 | 0.990 | 1.003 |
| synthetic-manual-big-small-16000 | build_drop | 0.518249500 | 1.171 | 1.383 | 1.163 | 0.682 | 1.021 |
| synthetic-manual-big-small-16000 | cow | 0.492883000 | 0.291 | 0.267 | 0.268 | 0.980 | 0.922 |
| synthetic-manual-big-small-16000 | clone_drop | 0.000015890 | 0.977 | 1.044 | 1.041 | 1.017 | 1.068 |
| synthetic-manual-big-small-16000 | drop | 0.235462500 | 1.374 | 1.429 | 1.427 | 0.700 | 1.015 |
| synthetic-big-large-16000 | scan_full | 1.026375667 | 0.996 | 1.001 | 0.998 | 1.002 | 1.006 |
| synthetic-big-large-16000 | scan_key | 1.032334000 | 1.000 | 0.999 | 0.999 | 1.007 | 1.007 |
| synthetic-big-large-16000 | build_drop | 0.519143000 | 1.193 | 1.436 | 1.201 | 0.680 | 1.001 |
| synthetic-big-large-16000 | cow | 0.493107500 | 0.291 | 0.273 | 0.272 | 0.981 | 0.909 |
| synthetic-big-large-16000 | clone_drop | 0.000015935 | 0.977 | 1.043 | 1.038 | 1.014 | 1.065 |
| synthetic-big-large-16000 | drop | 0.240591500 | 1.371 | 1.411 | 1.382 | 0.698 | 1.003 |
| synthetic-rational-16000 | scan_full | 1.003266333 | 0.986 | 1.003 | 1.003 | 0.973 | 1.033 |
| synthetic-rational-16000 | scan_key | 1.010943667 | 0.987 | 0.996 | 1.006 | 1.008 | 1.017 |
| synthetic-rational-16000 | build_drop | 0.742752500 | 1.103 | 1.277 | 1.102 | 0.760 | 1.007 |
| synthetic-rational-16000 | cow | 0.699631000 | 0.223 | 0.206 | 0.209 | 1.005 | 0.902 |
| synthetic-rational-16000 | clone_drop | 0.000015930 | 0.980 | 1.060 | 1.040 | 1.014 | 1.059 |
| synthetic-rational-16000 | drop | 0.331385500 | 1.252 | 1.223 | 1.239 | 0.739 | 0.994 |
