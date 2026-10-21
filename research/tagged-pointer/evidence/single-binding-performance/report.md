# Borrowed single-binding comparison

Elapsed ratios below one are faster. Every original label and all A/A controls are retained.

## Original suite

| Label | Matcher/base | Single/base | Single/matcher | base A/A | matcher A/A | single A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| association-part-key-read-width4000 | 1.0004 | 1.0247 | 1.0185 | 0.9966 | 1.0080 | 0.9724 |
| association-part-key-read-width32000 | 0.9981 | 0.9682 | 0.9667 | 0.9958 | 0.9989 | 0.9963 |
| association-part-positional-read-width4000 | 1.0063 | 1.0087 | 1.0071 | 1.0078 | 1.0031 | 0.9998 |
| association-part-positional-read-width32000 | 1.0043 | 1.0077 | 1.0081 | 1.0114 | 1.0080 | 1.0074 |
| loop-empty | 1.0107 | 0.9629 | 0.9701 | 1.0146 | 0.9785 | 1.0012 |
| do-count-null | 1.0235 | 0.9900 | 0.9795 | 1.0082 | 0.9932 | 0.9987 |
| do-iterator-null | 0.9957 | 1.0149 | 1.0163 | 1.0012 | 1.0001 | 1.0005 |
| table-count-null | 1.0095 | 1.0083 | 0.9985 | 0.9942 | 0.9916 | 1.0097 |
| table-iterator-null | 1.0080 | 1.0095 | 1.0007 | 0.9929 | 1.0054 | 1.0007 |
| table-iterator-symbol | 1.0090 | 1.0382 | 1.0079 | 0.9954 | 0.9943 | 1.0021 |
| literal-integer | 1.0154 | 0.9662 | 0.9744 | 1.0208 | 0.9605 | 1.0095 |
| literal-string | 1.0005 | 0.9681 | 0.9510 | 1.0138 | 0.9613 | 1.0031 |
| symbol-ownvalue-read | 1.0127 | 1.0030 | 0.9895 | 1.0081 | 0.9690 | 1.0056 |
| symbol-ownvalue-guarded-read | 1.0128 | 1.0050 | 0.9918 | 1.0023 | 0.9647 | 1.0027 |
| set-ownvalue | 0.9835 | 0.9780 | 0.9956 | 1.0066 | 0.9968 | 1.0014 |
| list-literal-small | 1.0004 | 0.9847 | 0.9883 | 1.0083 | 0.9711 | 0.9933 |
| head-list | 0.9920 | 1.0003 | 1.0080 | 1.0066 | 0.9930 | 1.0099 |
| length-list | 0.9940 | 1.0053 | 1.0093 | 1.0045 | 0.9976 | 1.0091 |
| part-list | 0.9934 | 1.0034 | 1.0106 | 1.0026 | 1.0048 | 1.0044 |
| sameq-integer | 1.0081 | 1.0015 | 0.9957 | 1.0024 | 0.9999 | 1.0097 |
| sameq-list | 0.9954 | 1.0006 | 1.0085 | 0.9997 | 0.9932 | 1.0013 |
| if-true | 0.9961 | 0.9947 | 0.9965 | 1.0048 | 0.9933 | 1.0046 |
| module-local | 1.0047 | 0.9950 | 0.9935 | 1.0399 | 1.0105 | 1.0211 |
| module-chain-escape-small | 0.9930 | 0.9963 | 0.9960 | 1.0259 | 0.9999 | 1.0003 |
| module-chain-escape-large | 0.9896 | 0.9948 | 1.0040 | 0.9865 | 0.9893 | 0.9983 |
| block-local | 1.0023 | 1.0019 | 1.0036 | 1.0163 | 0.9888 | 0.9989 |
| block-symbol-only-repeat | 0.9977 | 0.9806 | 0.9791 | 1.0105 | 0.9833 | 1.0124 |
| block-init-null-repeat | 1.0091 | 1.0024 | 0.9961 | 1.0023 | 0.9772 | 1.0043 |
| block-init-read-repeat | 0.9885 | 0.9935 | 1.0043 | 1.0014 | 0.9751 | 1.0001 |
| with-local | 1.0057 | 1.0022 | 0.9966 | 1.0106 | 1.0081 | 1.0069 |
| table-small | 1.0022 | 0.9924 | 0.9869 | 1.0067 | 1.0104 | 1.0066 |
| table-iterator-plus | 1.0146 | 1.0238 | 1.0050 | 0.9948 | 1.0072 | 0.9683 |
| table-iterator-pair-list | 0.9961 | 1.0011 | 1.0079 | 0.9735 | 0.9985 | 1.0203 |
| do-small | 0.9975 | 0.9885 | 0.9910 | 1.0059 | 0.9884 | 1.0060 |
| downvalue-exact-hit | 1.0074 | 0.9998 | 0.9981 | 1.0045 | 1.0084 | 1.0016 |
| downvalue-exact-miss | 1.0064 | 1.0009 | 0.9974 | 1.0099 | 0.9914 | 1.0176 |
| downvalue-pattern-hit | 0.9930 | 1.0039 | 1.0129 | 0.9816 | 0.9986 | 0.9964 |
| downvalue-pattern-miss | 1.0192 | 1.0011 | 0.9859 | 1.0042 | 0.9850 | 1.0185 |
| downvalue-exact-miss-repeat | 1.0052 | 0.9965 | 0.9909 | 0.9998 | 0.9978 | 1.0005 |
| downvalue-pattern-miss-repeat | 1.0070 | 0.9984 | 0.9931 | 1.0069 | 0.9950 | 1.0094 |
| downvalue-guarded-hit | 0.9841 | 0.9911 | 1.0034 | 0.9810 | 0.9896 | 0.9970 |
| downvalue-guarded-decline-fallback | 1.0005 | 0.9999 | 0.9996 | 0.9990 | 1.0089 | 0.9867 |
| upvalue-guarded-hit | 0.9859 | 0.9897 | 1.0050 | 0.9973 | 0.9920 | 0.9955 |
| subvalue-unguarded-hit | 0.9972 | 0.9904 | 0.9938 | 0.9842 | 0.9997 | 1.0069 |
| subvalue-unguarded-miss | 0.9987 | 0.9825 | 0.9799 | 0.9943 | 0.9986 | 1.0001 |
| subvalue-guarded-hit | 0.9841 | 0.9849 | 0.9983 | 0.9878 | 0.9982 | 1.0064 |
| default-unguarded-hit | 1.0027 | 1.0046 | 1.0009 | 0.9965 | 1.0045 | 0.9852 |
| default-unguarded-miss | 1.0024 | 0.9981 | 0.9953 | 0.9901 | 0.9973 | 0.9798 |
| default-guarded-hit | 1.0094 | 1.0068 | 0.9954 | 0.9960 | 1.0057 | 0.9876 |
| matchq-integer-success | 1.0088 | 0.9996 | 0.9927 | 1.0026 | 1.0003 | 1.0054 |
| matchq-integer-fail | 1.0014 | 0.9974 | 0.9951 | 1.0013 | 1.0039 | 1.0064 |
| matcher-single-tiny | 1.0012 | 0.9881 | 0.9887 | 1.0044 | 0.9989 | 0.9962 |
| matcher-single-wide-small | 0.9979 | 1.0088 | 1.0074 | 0.9952 | 1.0149 | 0.9810 |
| matcher-single-wide-large | 1.0014 | 1.0107 | 1.0096 | 0.9925 | 1.0061 | 0.9882 |
| replaceall-simple | 1.0048 | 1.0047 | 0.9976 | 1.0136 | 0.9895 | 1.0037 |
| rewrite-guarded-hit | 0.9974 | 0.9923 | 0.9955 | 0.9929 | 0.9979 | 0.9965 |
| rewrite-guarded-decline-fallback | 0.9824 | 0.9615 | 0.9763 | 0.9975 | 0.9944 | 0.9950 |
| cases-wide-no-match | 1.0021 | 0.9966 | 0.9961 | 1.0002 | 0.9939 | 0.9930 |
| count-wide-no-match | 1.0092 | 0.9863 | 0.9828 | 0.9971 | 1.0003 | 1.0058 |
| position-wide-no-match | 0.9993 | 1.0032 | 1.0018 | 1.0023 | 0.9986 | 0.9964 |
| deletecases-wide-no-match | 0.9986 | 0.9960 | 0.9902 | 1.0000 | 1.0035 | 1.0081 |
| deletecases-wide-association-no-match | 0.9992 | 0.9825 | 0.9859 | 1.0007 | 0.9915 | 1.0050 |
| replace-wide-no-match | 1.0030 | 1.0171 | 1.0131 | 1.0006 | 1.0017 | 1.0035 |
| replacepart-wide-rule-list | 1.0075 | 0.9884 | 0.9852 | 1.0059 | 0.9985 | 0.9724 |
| replacepart-guarded-hit | 1.0280 | 1.0022 | 0.9752 | 0.9958 | 0.9740 | 0.9963 |
| replacepart-guarded-decline-fallback | 1.0155 | 0.9878 | 0.9721 | 1.0033 | 0.9998 | 0.9968 |
| string-repeat-small | 1.0072 | 0.9827 | 0.9790 | 0.9838 | 0.9895 | 1.0006 |
| character-range-small | 1.0015 | 0.9981 | 0.9884 | 1.0132 | 1.0093 | 1.0118 |
| stringtrim-literal-list-4000 | 0.9981 | 0.9826 | 0.9801 | 0.9963 | 1.0017 | 1.0004 |
| stringtrim-literal-list-32000 | 0.9873 | 0.9782 | 0.9888 | 0.9997 | 1.0017 | 1.0060 |
| stringtrim-single-literal-4000 | 1.0009 | 0.9819 | 0.9869 | 1.0005 | 1.0008 | 0.9973 |
| stringtrim-single-literal-32000 | 0.9920 | 0.9814 | 0.9894 | 1.0035 | 0.9984 | 1.0030 |
| stringtrim-whitespace-4000 | 0.9976 | 0.9969 | 0.9948 | 0.9870 | 0.9996 | 1.0073 |
| stringtrim-whitespace-32000 | 0.9876 | 0.9786 | 0.9926 | 1.0050 | 0.9980 | 1.0037 |
| stringreplace-single-literal-unguarded | 1.0100 | 0.9914 | 0.9845 | 0.9966 | 0.9995 | 1.0098 |
| stringreplace-pattern-unguarded | 1.0001 | 0.9904 | 0.9898 | 0.9851 | 1.0043 | 0.9988 |
| stringreplace-guarded-hit | 1.0022 | 0.9908 | 0.9917 | 0.9951 | 0.9841 | 1.0058 |
| stringreplace-guarded-decline-fallback | 1.0000 | 0.9930 | 0.9931 | 0.9941 | 0.9852 | 1.0068 |
| appendto-list-incremental-small | 1.0019 | 0.9964 | 0.9982 | 0.9977 | 0.9962 | 0.9988 |
| appendto-list-incremental-large | 1.0024 | 0.9969 | 0.9961 | 1.0017 | 1.0011 | 1.0003 |
| appendto-part-outer1000-small | 1.0004 | 1.0006 | 0.9999 | 1.0030 | 1.0008 | 1.0009 |
| appendto-part-outer1000-large | 1.0005 | 1.0017 | 1.0006 | 1.0011 | 0.9984 | 1.0014 |
| association-set-key-incremental-small | 0.9269 | 0.9214 | 0.9954 | 1.1658 | 0.9969 | 0.9963 |
| association-set-key-incremental-large | 0.9305 | 0.9261 | 0.9948 | 1.1596 | 0.9964 | 0.9955 |
| association-small-literal-repeat | 0.9971 | 0.9983 | 1.0038 | 0.9909 | 1.0065 | 0.9939 |
| set-part-fixed100-len100 | 0.9992 | 1.0069 | 1.0069 | 1.0164 | 1.0109 | 0.9961 |
| set-part-fixed100-len800 | 1.0181 | 1.0037 | 0.9942 | 1.0033 | 1.0076 | 1.0059 |
| set-part-fill-small | 0.9967 | 1.0038 | 1.0052 | 1.0146 | 1.0043 | 0.9905 |
| set-part-fill-large | 0.9949 | 0.9924 | 0.9957 | 1.0093 | 1.0046 | 1.0020 |
| upvalue-candidates-tiny | 1.0051 | 0.9983 | 0.9934 | 1.0079 | 0.9955 | 0.9895 |
| upvalue-candidates-wide-small | 0.9957 | 1.0045 | 1.0076 | 0.9864 | 1.0125 | 1.0095 |
| upvalue-candidates-wide-large | 0.9919 | 1.0081 | 1.0081 | 0.9850 | 1.0187 | 1.0081 |
| sort-callback-small | 0.9964 | 1.0302 | 1.0232 | 1.0128 | 0.9820 | 1.0174 |
| sort-callback-large | 0.9986 | 1.0311 | 1.0336 | 1.0136 | 0.9866 | 1.0143 |
| sortby-callback-small | 0.9926 | 1.0154 | 1.0170 | 1.0066 | 0.9887 | 1.0073 |
| sortby-callback-large | 0.9911 | 1.0105 | 1.0182 | 1.0037 | 0.9923 | 1.0064 |
| reversesort-callback-small | 1.0072 | 1.0097 | 1.0084 | 0.9988 | 0.9930 | 1.0035 |
| reversesort-callback-large | 1.0031 | 1.0122 | 1.0105 | 1.0035 | 0.9982 | 1.0097 |
| keysort-callback-small | 0.9936 | 0.9992 | 1.0099 | 1.0019 | 0.9830 | 1.0063 |
| keysort-callback-large | 0.9996 | 0.9949 | 0.9993 | 1.0051 | 0.9954 | 1.0118 |
| keysortby-callback-small | 0.9950 | 1.0020 | 1.0094 | 1.0166 | 1.0043 | 1.0005 |
| keysortby-callback-large | 0.9979 | 1.0024 | 1.0059 | 1.0134 | 0.9975 | 1.0051 |
| sort-default-wide | 0.7657 | 0.7671 | 1.0010 | 0.5877 | 0.9963 | 1.0006 |
| sort-symbol-default-width4000 | 0.9896 | 1.0195 | 1.0262 | 0.9904 | 0.9876 | 1.0048 |
| sort-symbol-default-width16000 | 0.9959 | 0.9963 | 1.0023 | 0.9865 | 1.0011 | 0.9938 |
| sortby-default-wide | 0.9994 | 0.9951 | 0.9935 | 0.9952 | 0.9966 | 1.0054 |
| reversesort-default-wide | 1.0000 | 1.0007 | 1.0023 | 0.9989 | 0.9997 | 1.0006 |
| keysort-default-wide | 0.9563 | 0.9814 | 1.0160 | 1.0615 | 0.9962 | 1.0331 |
| keysortby-default-wide | 0.9836 | 0.9745 | 0.9957 | 1.0025 | 0.9957 | 1.0073 |
| valueq-undefined-symbol-definitions-2000 | 0.9957 | 1.0018 | 1.0045 | 0.9989 | 0.9948 | 1.0067 |
| valueq-undefined-symbol-definitions-8000 | 0.9987 | 1.0019 | 1.0023 | 0.9998 | 0.9957 | 1.0002 |
| module-retained-gensym-gc-batch | 1.0145 | 0.9829 | 0.9796 | 1.0057 | 1.0004 | 1.0241 |

## focus

| Label | Matcher/base | Single/base | Single/matcher | base A/A | matcher A/A | single A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| focus-increment-1024 | 0.8879 | 0.8244 | 0.9255 | 1.0144 | 1.0402 | 1.0119 |
| focus-increment-16000 | 0.8873 | 0.8215 | 0.9260 | 1.0212 | 1.0399 | 1.0155 |
| focus-increment-reversed-1024 | 0.8861 | 0.8220 | 0.9243 | 1.0204 | 1.0334 | 1.0112 |
| focus-increment-reversed-16000 | 0.8841 | 0.8193 | 0.9244 | 1.0212 | 1.0370 | 1.0152 |
| focus-identity-1024 | 0.6058 | 0.4483 | 0.7419 | 1.0039 | 1.0024 | 1.0018 |
| focus-identity-16000 | 0.5844 | 0.4187 | 0.7193 | 1.0052 | 0.9967 | 0.9986 |
| focus-miss-1024 | 0.9978 | 1.0121 | 1.0151 | 1.0015 | 1.0003 | 0.9952 |
| focus-miss-16000 | 0.9987 | 1.0113 | 1.0110 | 1.0058 | 0.9976 | 0.9948 |
| focus-opaque-body-1024 | 0.6667 | 0.5703 | 0.8467 | 0.9864 | 1.0002 | 0.9813 |
| focus-opaque-body-16000 | 0.7207 | 0.6204 | 0.8608 | 0.9816 | 1.0116 | 0.9867 |
| focus-flat-nested-1024 | 0.9199 | 0.8910 | 0.9688 | 0.9924 | 1.0117 | 1.0159 |
| focus-flat-nested-16000 | 0.9180 | 0.8902 | 0.9716 | 0.9939 | 1.0096 | 1.0123 |

## pipeline

| Label | Matcher/base | Single/base | Single/matcher | base A/A | matcher A/A | single A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| synthetic-rewrite-heavy-hit-4000 | 0.8888 | 0.8270 | 0.9306 | 1.0157 | 1.0273 | 0.9978 |
| synthetic-rewrite-heavy-hit-16000 | 0.8882 | 0.8261 | 0.9327 | 1.0061 | 1.0304 | 0.9981 |
| synthetic-mixed-hit-4000 | 0.8956 | 0.8439 | 0.9408 | 1.0046 | 1.0246 | 0.9991 |
| synthetic-mixed-hit-16000 | 0.9050 | 0.8540 | 0.9384 | 1.0127 | 1.0267 | 1.0010 |
| synthetic-mixed-control-4000 | 0.9472 | 0.9647 | 1.0171 | 0.9250 | 0.9979 | 1.0045 |
| synthetic-mixed-control-16000 | 0.9360 | 0.7150 | 0.9724 | 0.9757 | 1.0040 | 0.9987 |

## shape

| Label | Matcher/base | Single/base | Single/matcher | base A/A | matcher A/A | single A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| shape-chain-16-positive | 1.0011 | 1.0010 | 1.0000 | 0.9936 | 0.9996 | 0.9956 |
| shape-chain-16-negative | 0.9998 | 0.9981 | 0.9917 | 1.0037 | 1.0018 | 1.0019 |
| shape-chain-32-positive | 1.0055 | 0.9968 | 0.9900 | 1.0041 | 1.0004 | 1.0051 |
| shape-chain-32-negative | 1.0000 | 0.9964 | 0.9980 | 1.0014 | 1.0010 | 0.9990 |
| shape-chain-64-positive | 1.0009 | 1.0081 | 1.0017 | 1.0020 | 0.9970 | 1.0000 |
| shape-chain-64-negative | 1.0001 | 1.0087 | 1.0033 | 0.9996 | 0.9934 | 0.9990 |
| shape-binary-5-positive | 0.9998 | 1.0056 | 0.9974 | 0.9991 | 1.0040 | 1.0090 |
| shape-binary-5-negative | 0.9948 | 0.9983 | 1.0004 | 1.0034 | 0.9969 | 0.9969 |
| shape-binary-7-positive | 1.0002 | 0.9953 | 0.9884 | 0.9997 | 0.9925 | 0.9983 |
| shape-binary-7-negative | 0.9946 | 0.9809 | 0.9860 | 1.0068 | 0.9932 | 0.9909 |
| shape-binary-9-positive | 1.0022 | 0.9999 | 0.9928 | 1.0054 | 0.9966 | 0.9979 |
| shape-binary-9-negative | 1.0023 | 1.0005 | 0.9994 | 1.0022 | 0.9944 | 0.9898 |

All slower original-suite medians (including very small differences):

{
  "matcher/base": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width4000",
    "association-part-positional-read-width32000",
    "loop-empty",
    "do-count-null",
    "table-count-null",
    "table-iterator-null",
    "table-iterator-symbol",
    "literal-integer",
    "literal-string",
    "symbol-ownvalue-read",
    "symbol-ownvalue-guarded-read",
    "list-literal-small",
    "sameq-integer",
    "module-local",
    "block-local",
    "block-init-null-repeat",
    "with-local",
    "table-small",
    "table-iterator-plus",
    "downvalue-exact-hit",
    "downvalue-exact-miss",
    "downvalue-pattern-miss",
    "downvalue-exact-miss-repeat",
    "downvalue-pattern-miss-repeat",
    "downvalue-guarded-decline-fallback",
    "default-unguarded-hit",
    "default-unguarded-miss",
    "default-guarded-hit",
    "matchq-integer-success",
    "matchq-integer-fail",
    "matcher-single-tiny",
    "matcher-single-wide-large",
    "replaceall-simple",
    "cases-wide-no-match",
    "count-wide-no-match",
    "replace-wide-no-match",
    "replacepart-wide-rule-list",
    "replacepart-guarded-hit",
    "replacepart-guarded-decline-fallback",
    "string-repeat-small",
    "character-range-small",
    "stringtrim-single-literal-4000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-pattern-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "appendto-list-incremental-small",
    "appendto-list-incremental-large",
    "appendto-part-outer1000-small",
    "appendto-part-outer1000-large",
    "set-part-fixed100-len800",
    "upvalue-candidates-tiny",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "reversesort-default-wide",
    "module-retained-gensym-gc-batch"
  ],
  "single/base": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width4000",
    "association-part-positional-read-width32000",
    "do-iterator-null",
    "table-count-null",
    "table-iterator-null",
    "table-iterator-symbol",
    "symbol-ownvalue-read",
    "symbol-ownvalue-guarded-read",
    "head-list",
    "length-list",
    "part-list",
    "sameq-integer",
    "sameq-list",
    "block-local",
    "block-init-null-repeat",
    "with-local",
    "table-iterator-plus",
    "table-iterator-pair-list",
    "downvalue-exact-miss",
    "downvalue-pattern-hit",
    "downvalue-pattern-miss",
    "default-unguarded-hit",
    "default-guarded-hit",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "replaceall-simple",
    "position-wide-no-match",
    "replace-wide-no-match",
    "replacepart-guarded-hit",
    "appendto-part-outer1000-small",
    "appendto-part-outer1000-large",
    "set-part-fixed100-len100",
    "set-part-fixed100-len800",
    "set-part-fill-small",
    "upvalue-candidates-wide-small",
    "upvalue-candidates-wide-large",
    "sort-callback-small",
    "sort-callback-large",
    "sortby-callback-small",
    "sortby-callback-large",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysortby-callback-small",
    "keysortby-callback-large",
    "sort-symbol-default-width4000",
    "reversesort-default-wide",
    "valueq-undefined-symbol-definitions-2000",
    "valueq-undefined-symbol-definitions-8000"
  ],
  "single/matcher": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width4000",
    "association-part-positional-read-width32000",
    "do-iterator-null",
    "table-iterator-null",
    "table-iterator-symbol",
    "head-list",
    "length-list",
    "part-list",
    "sameq-list",
    "module-chain-escape-large",
    "block-local",
    "block-init-read-repeat",
    "table-iterator-plus",
    "table-iterator-pair-list",
    "downvalue-pattern-hit",
    "downvalue-guarded-hit",
    "upvalue-guarded-hit",
    "default-unguarded-hit",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "position-wide-no-match",
    "replace-wide-no-match",
    "appendto-part-outer1000-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len100",
    "set-part-fill-small",
    "upvalue-candidates-wide-small",
    "upvalue-candidates-wide-large",
    "sort-callback-small",
    "sort-callback-large",
    "sortby-callback-small",
    "sortby-callback-large",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysort-callback-small",
    "keysortby-callback-small",
    "keysortby-callback-large",
    "sort-default-wide",
    "sort-symbol-default-width4000",
    "sort-symbol-default-width16000",
    "reversesort-default-wide",
    "keysort-default-wide",
    "valueq-undefined-symbol-definitions-2000",
    "valueq-undefined-symbol-definitions-8000"
  ]
}

One nominally idle desktop, same WSL host and toolchain; external background work is not controlled completely.
Twelve paired process blocks; retained in-process repetitions are not independent experiment blocks.
Every slower original-suite median is retained; A/A ratios are controls and are never subtracted.
Same-binary aliases check timing noise, not independent compilation reproducibility.
Synthetic focus/pipeline programs do not establish a real application distribution or whole-runtime speedup.
Value, alias, continuation, trace and selected resource-limit checks are finite evidence; this is not full contextual equivalence.
