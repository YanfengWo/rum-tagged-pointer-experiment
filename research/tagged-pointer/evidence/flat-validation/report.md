# Uninstrumented Flat candidate validation

Elapsed ratios below one are faster. Every original label and all A/A controls are retained.

## Original suite

| Label | Wide/base | Flat/base | Combined/base | Combined/wide | Combined/flat | base A/A | wide A/A | flat A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| association-part-key-read-width4000 | 0.9841 | 1.0032 | 1.0257 | 1.0432 | 1.0249 | 1.0179 | 1.0063 | 0.9905 | 0.9975 |
| association-part-key-read-width32000 | 0.9682 | 1.0029 | 0.9824 | 1.0164 | 0.9808 | 1.0014 | 1.0002 | 0.9956 | 1.0057 |
| association-part-positional-read-width4000 | 0.9870 | 1.0220 | 1.0149 | 1.0282 | 1.0000 | 0.9966 | 0.9868 | 1.0055 | 0.9982 |
| association-part-positional-read-width32000 | 0.9948 | 1.0209 | 1.0190 | 1.0298 | 1.0014 | 0.9987 | 0.9888 | 1.0076 | 0.9984 |
| loop-empty | 0.9915 | 0.9887 | 0.9906 | 0.9887 | 0.9695 | 0.9785 | 0.9802 | 0.9985 | 0.9931 |
| do-count-null | 0.9963 | 1.0013 | 1.0028 | 1.0119 | 0.9875 | 1.0255 | 1.0104 | 0.9949 | 1.0395 |
| do-iterator-null | 0.9754 | 0.9614 | 0.9765 | 0.9978 | 0.9994 | 1.0103 | 1.0202 | 1.0013 | 0.9947 |
| table-count-null | 0.9961 | 1.0121 | 1.0026 | 1.0078 | 0.9874 | 1.0051 | 1.0125 | 0.9998 | 0.9961 |
| table-iterator-null | 0.9898 | 1.0019 | 0.9875 | 0.9971 | 0.9753 | 1.0010 | 0.9526 | 0.9764 | 0.9930 |
| table-iterator-symbol | 0.9909 | 0.9790 | 0.9873 | 0.9819 | 1.0050 | 1.0025 | 0.9982 | 1.0091 | 1.0069 |
| literal-integer | 0.9966 | 1.0128 | 1.0080 | 0.9969 | 0.9805 | 0.9810 | 1.0055 | 1.0132 | 1.0083 |
| literal-string | 0.9803 | 1.0313 | 1.0498 | 1.0311 | 0.9562 | 1.0083 | 0.9906 | 1.0020 | 1.0927 |
| symbol-ownvalue-read | 0.9900 | 1.0204 | 1.0073 | 1.0151 | 0.9637 | 0.9645 | 0.9717 | 1.0056 | 0.9999 |
| symbol-ownvalue-guarded-read | 0.9951 | 0.9963 | 0.9979 | 1.0105 | 0.9838 | 0.9863 | 0.9807 | 0.9917 | 1.0009 |
| set-ownvalue | 0.9821 | 0.9893 | 0.9804 | 1.0001 | 0.9952 | 0.9958 | 0.9961 | 0.9872 | 0.9904 |
| list-literal-small | 0.9950 | 1.0065 | 0.9932 | 1.0010 | 0.9775 | 0.9988 | 0.9987 | 0.9993 | 1.0114 |
| head-list | 0.9930 | 0.9901 | 0.9929 | 1.0014 | 1.0022 | 0.9904 | 1.0000 | 0.9982 | 0.9933 |
| length-list | 0.9921 | 0.9924 | 0.9922 | 1.0027 | 1.0064 | 0.9946 | 1.0064 | 0.9943 | 0.9961 |
| part-list | 1.0026 | 1.0027 | 1.0046 | 1.0032 | 1.0047 | 1.0012 | 1.0083 | 0.9908 | 0.9933 |
| sameq-integer | 0.9965 | 1.0062 | 1.0098 | 1.0194 | 1.0134 | 1.0066 | 1.0014 | 0.9875 | 0.9889 |
| sameq-list | 0.9892 | 0.9873 | 0.9902 | 1.0018 | 1.0040 | 1.0011 | 1.0022 | 0.9976 | 0.9908 |
| if-true | 0.9990 | 1.0037 | 1.0139 | 1.0158 | 0.9993 | 0.9922 | 0.9958 | 0.9927 | 0.9986 |
| module-local | 1.0162 | 1.0237 | 1.0205 | 0.9988 | 0.9962 | 0.9983 | 1.0416 | 0.9863 | 0.9925 |
| module-chain-escape-small | 0.9967 | 1.0285 | 0.9886 | 0.9939 | 0.9507 | 1.0032 | 0.9954 | 1.0160 | 1.0181 |
| module-chain-escape-large | 0.9934 | 1.0178 | 1.0315 | 1.0331 | 1.0097 | 0.9872 | 0.9917 | 1.0078 | 1.0048 |
| block-local | 0.9992 | 1.0067 | 1.0249 | 1.0272 | 1.0189 | 1.0026 | 1.0076 | 0.9993 | 0.9797 |
| block-symbol-only-repeat | 0.9889 | 1.0107 | 1.0058 | 1.0161 | 0.9871 | 0.9988 | 1.0263 | 0.9752 | 1.0089 |
| block-init-null-repeat | 0.9974 | 1.0336 | 1.0336 | 1.0318 | 1.0017 | 1.0075 | 1.0180 | 0.9944 | 1.0089 |
| block-init-read-repeat | 1.0294 | 1.0192 | 1.0413 | 1.0095 | 1.0150 | 1.0046 | 1.0270 | 1.0057 | 1.0063 |
| with-local | 1.0046 | 0.9998 | 1.0168 | 1.0134 | 1.0164 | 1.0008 | 0.9909 | 1.0018 | 0.9993 |
| table-small | 0.9933 | 1.0125 | 1.0059 | 1.0116 | 0.9901 | 1.0050 | 1.0179 | 1.0015 | 0.9953 |
| table-iterator-plus | 0.9945 | 0.9772 | 0.9785 | 0.9710 | 1.0143 | 0.9544 | 0.9759 | 1.0119 | 1.0045 |
| table-iterator-pair-list | 1.0037 | 1.0019 | 1.0110 | 1.0142 | 1.0098 | 1.0262 | 0.9780 | 0.9850 | 1.0081 |
| do-small | 1.0056 | 1.0012 | 1.0058 | 1.0010 | 1.0041 | 1.0148 | 1.0143 | 1.0106 | 0.9857 |
| downvalue-exact-hit | 1.0001 | 0.9984 | 0.9866 | 0.9853 | 0.9907 | 1.0051 | 0.9981 | 0.9868 | 0.9947 |
| downvalue-exact-miss | 1.0007 | 1.0100 | 1.0093 | 1.0109 | 0.9976 | 0.9984 | 0.9966 | 0.9870 | 1.0084 |
| downvalue-pattern-hit | 0.9827 | 1.0021 | 1.0025 | 1.0186 | 1.0006 | 0.9790 | 1.0110 | 1.0202 | 0.9837 |
| downvalue-pattern-miss | 1.0030 | 0.9952 | 0.9974 | 0.9959 | 0.9966 | 1.0267 | 1.0075 | 0.9960 | 1.0106 |
| downvalue-exact-miss-repeat | 1.0072 | 0.9997 | 1.0076 | 1.0059 | 1.0038 | 1.0045 | 0.9957 | 0.9946 | 1.0183 |
| downvalue-pattern-miss-repeat | 0.9863 | 0.9937 | 0.9908 | 0.9986 | 1.0052 | 1.0187 | 1.0037 | 0.9833 | 1.0085 |
| downvalue-guarded-hit | 0.9880 | 0.9990 | 1.0051 | 1.0176 | 1.0038 | 0.9826 | 0.9988 | 1.0218 | 1.0003 |
| downvalue-guarded-decline-fallback | 0.9827 | 0.9992 | 0.9989 | 1.0168 | 0.9984 | 0.9732 | 1.0069 | 1.0334 | 0.9925 |
| upvalue-guarded-hit | 0.9841 | 0.9977 | 0.9975 | 1.0127 | 1.0021 | 1.0081 | 0.9966 | 1.0033 | 1.0005 |
| subvalue-unguarded-hit | 0.9995 | 1.0011 | 1.0115 | 1.0102 | 1.0068 | 0.9957 | 1.0086 | 0.9963 | 0.9962 |
| subvalue-unguarded-miss | 0.9979 | 0.9908 | 1.0238 | 1.0255 | 1.0312 | 1.0078 | 1.0051 | 1.0027 | 1.0089 |
| subvalue-guarded-hit | 0.9944 | 0.9969 | 1.0064 | 1.0115 | 1.0082 | 0.9990 | 1.0026 | 1.0096 | 0.9949 |
| default-unguarded-hit | 0.9960 | 1.0078 | 1.0134 | 1.0207 | 1.0061 | 0.9831 | 0.9881 | 1.0140 | 1.0045 |
| default-unguarded-miss | 0.9941 | 1.0003 | 1.0198 | 1.0262 | 1.0170 | 0.9852 | 0.9942 | 1.0002 | 0.9899 |
| default-guarded-hit | 0.9967 | 1.0043 | 1.0161 | 1.0196 | 1.0132 | 0.9888 | 0.9882 | 1.0089 | 0.9999 |
| matchq-integer-success | 0.9971 | 1.0196 | 1.0143 | 1.0166 | 0.9999 | 1.0083 | 1.0006 | 0.9961 | 1.0027 |
| matchq-integer-fail | 0.9991 | 1.0078 | 1.0110 | 1.0139 | 1.0032 | 1.0062 | 1.0051 | 0.9943 | 1.0099 |
| matcher-single-tiny | 1.0000 | 0.9972 | 0.9988 | 1.0004 | 0.9979 | 0.9971 | 1.0001 | 0.9927 | 1.0090 |
| matcher-single-wide-small | 1.0089 | 1.0096 | 1.0161 | 1.0051 | 1.0021 | 0.9953 | 0.9846 | 1.0014 | 1.0130 |
| matcher-single-wide-large | 1.0122 | 1.0101 | 1.0169 | 1.0054 | 1.0008 | 0.9972 | 0.9839 | 1.0016 | 1.0041 |
| replaceall-simple | 0.9921 | 0.9867 | 0.9902 | 1.0061 | 1.0066 | 0.9899 | 0.9852 | 0.9940 | 1.0036 |
| rewrite-guarded-hit | 0.9924 | 0.9952 | 0.9968 | 1.0043 | 1.0014 | 0.9899 | 1.0062 | 1.0019 | 1.0025 |
| rewrite-guarded-decline-fallback | 0.9971 | 0.9914 | 0.9887 | 0.9924 | 0.9961 | 0.9914 | 1.0001 | 1.0125 | 0.9972 |
| cases-wide-no-match | 0.9960 | 0.9883 | 0.9944 | 0.9892 | 0.9937 | 0.9977 | 0.9856 | 1.0024 | 0.9990 |
| count-wide-no-match | 0.9915 | 0.9921 | 0.9909 | 0.9993 | 1.0000 | 1.0092 | 1.0020 | 1.0045 | 1.0083 |
| position-wide-no-match | 0.9949 | 0.9878 | 0.9866 | 0.9848 | 0.9954 | 1.0061 | 0.9985 | 0.9965 | 0.9907 |
| deletecases-wide-no-match | 0.9963 | 0.9932 | 0.9977 | 1.0014 | 1.0037 | 1.0023 | 1.0125 | 1.0052 | 1.0000 |
| deletecases-wide-association-no-match | 1.0073 | 1.0190 | 1.0382 | 1.0232 | 1.0154 | 0.9933 | 0.9976 | 1.0300 | 0.9873 |
| replace-wide-no-match | 0.9085 | 1.0043 | 0.9107 | 1.0021 | 0.9088 | 1.0005 | 1.0018 | 1.0000 | 0.9983 |
| replacepart-wide-rule-list | 0.9916 | 0.9981 | 0.9978 | 0.9943 | 0.9966 | 1.0113 | 1.0032 | 1.0042 | 1.0183 |
| replacepart-guarded-hit | 1.0019 | 1.0339 | 1.0184 | 1.0131 | 0.9844 | 1.0032 | 0.9819 | 1.0302 | 0.9894 |
| replacepart-guarded-decline-fallback | 0.9929 | 1.0222 | 1.0028 | 1.0089 | 0.9821 | 0.9986 | 0.9906 | 1.0243 | 0.9866 |
| string-repeat-small | 0.9844 | 0.9842 | 1.0020 | 1.0122 | 1.0181 | 1.0090 | 1.0227 | 0.9831 | 0.9951 |
| character-range-small | 0.9873 | 1.0010 | 1.0007 | 1.0146 | 1.0046 | 1.0126 | 0.9946 | 0.9885 | 1.0008 |
| stringtrim-literal-list-4000 | 0.9834 | 1.1949 | 1.1923 | 1.2115 | 1.0016 | 0.9995 | 1.0017 | 0.9927 | 1.0021 |
| stringtrim-literal-list-32000 | 0.9886 | 1.2260 | 1.2252 | 1.2367 | 1.0002 | 1.0042 | 0.9951 | 1.0023 | 0.9990 |
| stringtrim-single-literal-4000 | 0.9887 | 1.2120 | 1.2121 | 1.2249 | 1.0001 | 1.0041 | 0.9942 | 0.9930 | 0.9990 |
| stringtrim-single-literal-32000 | 0.9861 | 1.2332 | 1.2383 | 1.2560 | 1.0032 | 0.9989 | 1.0011 | 1.0022 | 1.0007 |
| stringtrim-whitespace-4000 | 0.9867 | 1.2065 | 1.2111 | 1.2288 | 1.0045 | 1.0147 | 1.0049 | 0.9894 | 0.9984 |
| stringtrim-whitespace-32000 | 0.9884 | 1.2311 | 1.2326 | 1.2441 | 1.0000 | 1.0087 | 1.0078 | 0.9986 | 1.0000 |
| stringreplace-single-literal-unguarded | 0.9927 | 1.0142 | 1.0251 | 1.0353 | 1.0124 | 1.0055 | 0.9995 | 1.0019 | 1.0131 |
| stringreplace-pattern-unguarded | 1.0100 | 1.0104 | 1.0127 | 0.9992 | 1.0014 | 1.0107 | 0.9978 | 0.9848 | 1.0083 |
| stringreplace-guarded-hit | 1.0048 | 1.0004 | 1.0050 | 1.0036 | 1.0018 | 1.0079 | 0.9988 | 1.0021 | 1.0048 |
| stringreplace-guarded-decline-fallback | 1.0011 | 0.9990 | 1.0076 | 1.0023 | 1.0072 | 1.0021 | 0.9991 | 0.9968 | 0.9872 |
| appendto-list-incremental-small | 0.9986 | 0.9982 | 0.9308 | 0.9336 | 0.9359 | 1.0061 | 1.0030 | 0.9954 | 0.9914 |
| appendto-list-incremental-large | 1.0008 | 0.9998 | 1.0005 | 0.9999 | 1.0017 | 0.9992 | 0.9993 | 0.9979 | 0.9999 |
| appendto-part-outer1000-small | 1.0022 | 1.0020 | 1.0014 | 0.9999 | 0.9992 | 1.0023 | 0.9970 | 0.9999 | 0.9993 |
| appendto-part-outer1000-large | 1.0039 | 1.0026 | 1.0023 | 0.9988 | 0.9989 | 1.0028 | 0.9945 | 1.0012 | 0.9991 |
| association-set-key-incremental-small | 0.9978 | 1.0088 | 1.0044 | 1.0052 | 0.9959 | 0.9978 | 0.9981 | 1.0037 | 1.0045 |
| association-set-key-incremental-large | 0.9974 | 1.0179 | 1.0093 | 1.0121 | 0.9908 | 0.9985 | 1.0002 | 1.0041 | 1.0055 |
| association-small-literal-repeat | 0.9927 | 0.9938 | 0.9948 | 1.0079 | 1.0081 | 1.0026 | 1.0164 | 1.0050 | 1.0110 |
| set-part-fixed100-len100 | 1.0128 | 1.0092 | 1.0317 | 1.0209 | 1.0179 | 1.0070 | 1.0020 | 0.9996 | 1.0148 |
| set-part-fixed100-len800 | 1.0155 | 1.0014 | 1.0325 | 1.0175 | 1.0301 | 1.0185 | 0.9827 | 1.0104 | 1.0057 |
| set-part-fill-small | 1.0141 | 0.9900 | 1.0088 | 0.9962 | 1.0239 | 1.0085 | 1.0061 | 1.0033 | 1.0130 |
| set-part-fill-large | 1.0223 | 1.0036 | 1.0278 | 1.0084 | 1.0199 | 1.0132 | 1.0114 | 0.9947 | 1.0127 |
| upvalue-candidates-tiny | 1.0051 | 0.9961 | 1.0169 | 1.0094 | 1.0226 | 1.0035 | 0.9937 | 0.9964 | 0.9873 |
| upvalue-candidates-wide-small | 0.9952 | 0.9970 | 0.9904 | 1.0023 | 0.9969 | 1.0138 | 1.0044 | 0.9937 | 0.9933 |
| upvalue-candidates-wide-large | 0.9866 | 0.9969 | 0.9891 | 0.9967 | 1.0007 | 1.0189 | 1.0034 | 1.0065 | 0.9967 |
| sort-callback-small | 1.0124 | 0.9893 | 1.0382 | 1.0243 | 1.0493 | 0.9983 | 1.0424 | 1.0239 | 0.9829 |
| sort-callback-large | 1.0073 | 0.9838 | 1.0338 | 1.0223 | 1.0449 | 0.9966 | 1.0526 | 1.0213 | 0.9770 |
| sortby-callback-small | 1.0129 | 1.0002 | 1.0177 | 1.0063 | 1.0158 | 1.0162 | 1.0125 | 1.0515 | 0.9830 |
| sortby-callback-large | 1.0187 | 1.0126 | 1.0200 | 1.0008 | 1.0081 | 1.0091 | 1.0092 | 1.0705 | 0.9738 |
| reversesort-callback-small | 0.9909 | 0.9880 | 1.0254 | 1.0356 | 1.0408 | 1.0145 | 1.0169 | 1.0063 | 0.9754 |
| reversesort-callback-large | 0.9894 | 0.9815 | 1.0360 | 1.0393 | 1.0456 | 1.0162 | 1.0216 | 1.0001 | 0.9824 |
| keysort-callback-small | 0.9898 | 0.9933 | 1.0088 | 1.0243 | 1.0126 | 1.0227 | 1.0052 | 1.0131 | 0.9792 |
| keysort-callback-large | 0.9884 | 0.9944 | 1.0001 | 1.0104 | 1.0137 | 1.0252 | 0.9942 | 1.0149 | 0.9849 |
| keysortby-callback-small | 0.9928 | 1.0038 | 1.0174 | 1.0207 | 1.0111 | 1.0069 | 0.9946 | 1.0202 | 0.9741 |
| keysortby-callback-large | 0.9962 | 1.0051 | 1.0197 | 1.0239 | 1.0154 | 1.0049 | 0.9986 | 1.0273 | 0.9816 |
| sort-default-wide | 1.0217 | 0.9923 | 0.9943 | 0.9735 | 1.0012 | 1.0023 | 1.0015 | 1.0027 | 1.0001 |
| sort-symbol-default-width4000 | 0.9880 | 1.0118 | 0.9952 | 0.9929 | 0.9696 | 0.9944 | 1.0011 | 1.0073 | 0.9785 |
| sort-symbol-default-width16000 | 0.9952 | 1.0083 | 1.0000 | 1.0002 | 0.9833 | 1.0002 | 1.0039 | 1.0076 | 0.9922 |
| sortby-default-wide | 1.0131 | 0.9784 | 1.0011 | 0.9872 | 1.0117 | 0.9950 | 1.0082 | 0.9958 | 0.9958 |
| reversesort-default-wide | 1.0162 | 1.0004 | 1.0067 | 0.9896 | 1.0050 | 1.0033 | 0.9960 | 1.0021 | 0.9993 |
| keysort-default-wide | 0.9912 | 1.0383 | 1.0151 | 1.0121 | 0.9776 | 0.9607 | 0.9512 | 0.9951 | 1.0058 |
| keysortby-default-wide | 1.0016 | 1.0095 | 1.0115 | 1.0022 | 1.0011 | 1.0001 | 0.9931 | 1.0049 | 0.9938 |
| valueq-undefined-symbol-definitions-2000 | 0.9834 | 0.9919 | 0.9898 | 1.0112 | 1.0002 | 1.0014 | 1.0070 | 0.9861 | 1.0054 |
| valueq-undefined-symbol-definitions-8000 | 0.9851 | 0.9962 | 0.9887 | 1.0034 | 0.9962 | 1.0005 | 1.0044 | 0.9932 | 1.0027 |
| module-retained-gensym-gc-batch | 1.0125 | 1.0191 | 0.9925 | 0.9816 | 0.9664 | 0.9848 | 1.0314 | 0.9833 | 0.9864 |

## focus

| Label | Wide/base | Flat/base | Combined/base | Combined/wide | Combined/flat | base A/A | wide A/A | flat A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| focus-increment-1024 | 1.0004 | 1.0061 | 1.0133 | 1.0101 | 1.0140 | 1.0104 | 0.9699 | 0.9942 | 1.0097 |
| focus-increment-16000 | 0.9977 | 1.0034 | 1.0127 | 1.0137 | 1.0153 | 1.0058 | 0.9746 | 0.9901 | 1.0198 |
| focus-increment-reversed-1024 | 1.0011 | 1.0047 | 1.0246 | 1.0204 | 1.0197 | 1.0147 | 0.9679 | 0.9928 | 1.0151 |
| focus-increment-reversed-16000 | 1.0005 | 1.0016 | 1.0164 | 1.0156 | 1.0136 | 1.0153 | 0.9743 | 0.9923 | 1.0185 |
| focus-identity-1024 | 0.9778 | 1.0146 | 0.9890 | 1.0056 | 0.9739 | 1.0031 | 1.0080 | 0.9863 | 1.0063 |
| focus-identity-16000 | 0.9815 | 1.0127 | 0.9868 | 1.0048 | 0.9735 | 0.9965 | 1.0050 | 0.9931 | 1.0045 |
| focus-miss-1024 | 0.9158 | 1.0032 | 0.9176 | 1.0016 | 0.9169 | 0.9972 | 1.0011 | 1.0004 | 1.0011 |
| focus-miss-16000 | 0.9089 | 1.0026 | 0.9091 | 0.9992 | 0.9080 | 0.9969 | 1.0076 | 0.9993 | 1.0051 |
| focus-opaque-body-1024 | 0.9937 | 1.0088 | 0.9703 | 0.9784 | 0.9670 | 1.0281 | 1.0242 | 0.9888 | 1.0143 |
| focus-opaque-body-16000 | 0.9918 | 1.0024 | 0.9888 | 0.9973 | 0.9865 | 1.0470 | 1.0024 | 0.9911 | 0.9999 |
| focus-flat-nested-1024 | 0.9988 | 0.9968 | 1.0011 | 1.0074 | 0.9990 | 1.0069 | 0.9963 | 1.0034 | 1.0041 |
| focus-flat-nested-16000 | 0.9992 | 0.9931 | 0.9966 | 1.0039 | 1.0011 | 1.0112 | 0.9960 | 1.0016 | 1.0044 |

## pipeline

| Label | Wide/base | Flat/base | Combined/base | Combined/wide | Combined/flat | base A/A | wide A/A | flat A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| synthetic-rewrite-heavy-hit-4000 | 0.9959 | 0.9962 | 1.0150 | 1.0195 | 1.0144 | 1.0044 | 0.9786 | 1.0018 | 1.0098 |
| synthetic-rewrite-heavy-hit-16000 | 0.9967 | 0.9965 | 1.0114 | 1.0159 | 1.0136 | 1.0065 | 0.9803 | 1.0018 | 1.0105 |
| synthetic-mixed-hit-4000 | 0.9933 | 0.9978 | 1.0114 | 1.0186 | 1.0120 | 1.0072 | 0.9820 | 1.0052 | 1.0111 |
| synthetic-mixed-hit-16000 | 0.9975 | 0.9989 | 1.0134 | 1.0171 | 1.0146 | 1.0056 | 0.9782 | 1.0009 | 1.0137 |
| synthetic-mixed-control-4000 | 1.0126 | 1.0145 | 1.0169 | 1.0027 | 1.0070 | 1.0037 | 1.0105 | 1.0118 | 1.0032 |
| synthetic-mixed-control-16000 | 1.0011 | 0.9889 | 0.9909 | 0.9966 | 1.0024 | 1.0043 | 0.9998 | 0.9977 | 0.9956 |

## Post-primary fresh-session StringTrim diagnostic

Same original windows and minimum-of-nine wrapper; selected after primary results. All six labels remain slower in every block.

| Label | Flat/base | Combined/wide |
| --- | ---: | ---: |
| stringtrim-literal-list-4000 | 1.1974 | 1.2029 |
| stringtrim-literal-list-32000 | 1.2274 | 1.2520 |
| stringtrim-single-literal-4000 | 1.2012 | 1.2128 |
| stringtrim-single-literal-32000 | 1.2221 | 1.2440 |
| stringtrim-whitespace-4000 | 1.2221 | 1.2186 |
| stringtrim-whitespace-32000 | 1.2359 | 1.2480 |

All slower original-suite medians (including very small differences):

{
  "wide/base": [
    "part-list",
    "module-local",
    "block-init-read-repeat",
    "with-local",
    "table-iterator-pair-list",
    "do-small",
    "downvalue-exact-hit",
    "downvalue-exact-miss",
    "downvalue-pattern-miss",
    "downvalue-exact-miss-repeat",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "deletecases-wide-association-no-match",
    "replacepart-guarded-hit",
    "stringreplace-pattern-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "appendto-list-incremental-large",
    "appendto-part-outer1000-small",
    "appendto-part-outer1000-large",
    "set-part-fixed100-len100",
    "set-part-fixed100-len800",
    "set-part-fill-small",
    "set-part-fill-large",
    "upvalue-candidates-tiny",
    "sort-callback-small",
    "sort-callback-large",
    "sortby-callback-small",
    "sortby-callback-large",
    "sort-default-wide",
    "sortby-default-wide",
    "reversesort-default-wide",
    "keysortby-default-wide",
    "module-retained-gensym-gc-batch"
  ],
  "flat/base": [
    "association-part-key-read-width4000",
    "association-part-key-read-width32000",
    "association-part-positional-read-width4000",
    "association-part-positional-read-width32000",
    "do-count-null",
    "table-count-null",
    "table-iterator-null",
    "literal-integer",
    "literal-string",
    "symbol-ownvalue-read",
    "list-literal-small",
    "part-list",
    "sameq-integer",
    "if-true",
    "module-local",
    "module-chain-escape-small",
    "module-chain-escape-large",
    "block-local",
    "block-symbol-only-repeat",
    "block-init-null-repeat",
    "block-init-read-repeat",
    "table-small",
    "table-iterator-pair-list",
    "do-small",
    "downvalue-exact-miss",
    "downvalue-pattern-hit",
    "subvalue-unguarded-hit",
    "default-unguarded-hit",
    "default-unguarded-miss",
    "default-guarded-hit",
    "matchq-integer-success",
    "matchq-integer-fail",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "deletecases-wide-association-no-match",
    "replace-wide-no-match",
    "replacepart-guarded-hit",
    "replacepart-guarded-decline-fallback",
    "character-range-small",
    "stringtrim-literal-list-4000",
    "stringtrim-literal-list-32000",
    "stringtrim-single-literal-4000",
    "stringtrim-single-literal-32000",
    "stringtrim-whitespace-4000",
    "stringtrim-whitespace-32000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-pattern-unguarded",
    "stringreplace-guarded-hit",
    "appendto-part-outer1000-small",
    "appendto-part-outer1000-large",
    "association-set-key-incremental-small",
    "association-set-key-incremental-large",
    "set-part-fixed100-len100",
    "set-part-fixed100-len800",
    "set-part-fill-large",
    "sortby-callback-small",
    "sortby-callback-large",
    "keysortby-callback-small",
    "keysortby-callback-large",
    "sort-symbol-default-width4000",
    "sort-symbol-default-width16000",
    "reversesort-default-wide",
    "keysort-default-wide",
    "keysortby-default-wide",
    "module-retained-gensym-gc-batch"
  ],
  "combined/base": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width4000",
    "association-part-positional-read-width32000",
    "do-count-null",
    "table-count-null",
    "literal-integer",
    "literal-string",
    "symbol-ownvalue-read",
    "part-list",
    "sameq-integer",
    "if-true",
    "module-local",
    "module-chain-escape-large",
    "block-local",
    "block-symbol-only-repeat",
    "block-init-null-repeat",
    "block-init-read-repeat",
    "with-local",
    "table-small",
    "table-iterator-pair-list",
    "do-small",
    "downvalue-exact-miss",
    "downvalue-pattern-hit",
    "downvalue-exact-miss-repeat",
    "downvalue-guarded-hit",
    "subvalue-unguarded-hit",
    "subvalue-unguarded-miss",
    "subvalue-guarded-hit",
    "default-unguarded-hit",
    "default-unguarded-miss",
    "default-guarded-hit",
    "matchq-integer-success",
    "matchq-integer-fail",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "deletecases-wide-association-no-match",
    "replacepart-guarded-hit",
    "replacepart-guarded-decline-fallback",
    "string-repeat-small",
    "character-range-small",
    "stringtrim-literal-list-4000",
    "stringtrim-literal-list-32000",
    "stringtrim-single-literal-4000",
    "stringtrim-single-literal-32000",
    "stringtrim-whitespace-4000",
    "stringtrim-whitespace-32000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-pattern-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "appendto-list-incremental-large",
    "appendto-part-outer1000-small",
    "appendto-part-outer1000-large",
    "association-set-key-incremental-small",
    "association-set-key-incremental-large",
    "set-part-fixed100-len100",
    "set-part-fixed100-len800",
    "set-part-fill-small",
    "set-part-fill-large",
    "upvalue-candidates-tiny",
    "sort-callback-small",
    "sort-callback-large",
    "sortby-callback-small",
    "sortby-callback-large",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysort-callback-small",
    "keysort-callback-large",
    "keysortby-callback-small",
    "keysortby-callback-large",
    "sortby-default-wide",
    "reversesort-default-wide",
    "keysort-default-wide",
    "keysortby-default-wide"
  ],
  "combined/wide": [
    "association-part-key-read-width4000",
    "association-part-key-read-width32000",
    "association-part-positional-read-width4000",
    "association-part-positional-read-width32000",
    "do-count-null",
    "table-count-null",
    "literal-string",
    "symbol-ownvalue-read",
    "symbol-ownvalue-guarded-read",
    "set-ownvalue",
    "list-literal-small",
    "head-list",
    "length-list",
    "part-list",
    "sameq-integer",
    "sameq-list",
    "if-true",
    "module-chain-escape-large",
    "block-local",
    "block-symbol-only-repeat",
    "block-init-null-repeat",
    "block-init-read-repeat",
    "with-local",
    "table-small",
    "table-iterator-pair-list",
    "do-small",
    "downvalue-exact-miss",
    "downvalue-pattern-hit",
    "downvalue-exact-miss-repeat",
    "downvalue-guarded-hit",
    "downvalue-guarded-decline-fallback",
    "upvalue-guarded-hit",
    "subvalue-unguarded-hit",
    "subvalue-unguarded-miss",
    "subvalue-guarded-hit",
    "default-unguarded-hit",
    "default-unguarded-miss",
    "default-guarded-hit",
    "matchq-integer-success",
    "matchq-integer-fail",
    "matcher-single-tiny",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "replaceall-simple",
    "rewrite-guarded-hit",
    "deletecases-wide-no-match",
    "deletecases-wide-association-no-match",
    "replace-wide-no-match",
    "replacepart-guarded-hit",
    "replacepart-guarded-decline-fallback",
    "string-repeat-small",
    "character-range-small",
    "stringtrim-literal-list-4000",
    "stringtrim-literal-list-32000",
    "stringtrim-single-literal-4000",
    "stringtrim-single-literal-32000",
    "stringtrim-whitespace-4000",
    "stringtrim-whitespace-32000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "association-set-key-incremental-small",
    "association-set-key-incremental-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len100",
    "set-part-fixed100-len800",
    "set-part-fill-large",
    "upvalue-candidates-tiny",
    "upvalue-candidates-wide-small",
    "sort-callback-small",
    "sort-callback-large",
    "sortby-callback-small",
    "sortby-callback-large",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysort-callback-small",
    "keysort-callback-large",
    "keysortby-callback-small",
    "keysortby-callback-large",
    "sort-symbol-default-width16000",
    "keysort-default-wide",
    "keysortby-default-wide",
    "valueq-undefined-symbol-definitions-2000",
    "valueq-undefined-symbol-definitions-8000"
  ],
  "combined/flat": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width32000",
    "table-iterator-symbol",
    "head-list",
    "length-list",
    "part-list",
    "sameq-integer",
    "sameq-list",
    "module-chain-escape-large",
    "block-local",
    "block-init-null-repeat",
    "block-init-read-repeat",
    "with-local",
    "table-iterator-plus",
    "table-iterator-pair-list",
    "do-small",
    "downvalue-pattern-hit",
    "downvalue-exact-miss-repeat",
    "downvalue-pattern-miss-repeat",
    "downvalue-guarded-hit",
    "upvalue-guarded-hit",
    "subvalue-unguarded-hit",
    "subvalue-unguarded-miss",
    "subvalue-guarded-hit",
    "default-unguarded-hit",
    "default-unguarded-miss",
    "default-guarded-hit",
    "matchq-integer-fail",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "replaceall-simple",
    "rewrite-guarded-hit",
    "deletecases-wide-no-match",
    "deletecases-wide-association-no-match",
    "string-repeat-small",
    "character-range-small",
    "stringtrim-literal-list-4000",
    "stringtrim-literal-list-32000",
    "stringtrim-single-literal-4000",
    "stringtrim-single-literal-32000",
    "stringtrim-whitespace-4000",
    "stringtrim-whitespace-32000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-pattern-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "appendto-list-incremental-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len100",
    "set-part-fixed100-len800",
    "set-part-fill-small",
    "set-part-fill-large",
    "upvalue-candidates-tiny",
    "upvalue-candidates-wide-large",
    "sort-callback-small",
    "sort-callback-large",
    "sortby-callback-small",
    "sortby-callback-large",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysort-callback-small",
    "keysort-callback-large",
    "keysortby-callback-small",
    "keysortby-callback-large",
    "sort-default-wide",
    "sortby-default-wide",
    "reversesort-default-wide",
    "keysortby-default-wide",
    "valueq-undefined-symbol-definitions-2000"
  ]
}

One nominally idle desktop, same WSL host and toolchain; external background work is not controlled completely.
Eight paired process blocks; retained in-process repetitions are not independent experiment blocks.
Every slower original-suite median is retained; A/A ratios are controls and are never subtracted.
Same-binary aliases check timing noise, not independent compilation reproducibility.
Synthetic focus/pipeline programs do not establish a real application distribution or whole-runtime speedup.
Resource-observable ByteCount/MemoryInUse and tight memory-limit behavior remain separate semantic boundaries.
