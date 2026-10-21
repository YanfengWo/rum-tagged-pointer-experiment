# Same-image rewrite switch diagnostic

Elapsed ratios below one are faster. Every original label and all A/A controls are retained.

## Original suite

| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| association-part-key-read-width4000 | 1.0011 | 1.0043 | 1.0018 | 1.0027 | 1.0003 | 1.0071 | 0.9983 | 1.0109 | 1.0132 |
| association-part-key-read-width32000 | 0.9983 | 1.0002 | 0.9990 | 0.9988 | 0.9962 | 0.9986 | 0.9977 | 1.0011 | 0.9977 |
| association-part-positional-read-width4000 | 0.9992 | 0.9978 | 1.0036 | 0.9978 | 0.9970 | 1.0040 | 0.9945 | 0.9953 | 0.9982 |
| association-part-positional-read-width32000 | 1.0006 | 0.9941 | 0.9965 | 0.9943 | 1.0003 | 0.9998 | 0.9999 | 0.9982 | 1.0042 |
| loop-empty | 0.9796 | 1.0004 | 1.0126 | 1.0298 | 0.9989 | 1.0202 | 1.0441 | 1.0065 | 1.0031 |
| do-count-null | 1.0137 | 1.0075 | 1.0250 | 1.0163 | 1.0135 | 0.9798 | 1.0093 | 0.9857 | 1.0149 |
| do-iterator-null | 0.9868 | 0.9796 | 0.9711 | 0.9745 | 0.9801 | 1.0256 | 0.9780 | 0.9823 | 0.9961 |
| table-count-null | 1.0192 | 1.0085 | 1.0095 | 0.9832 | 0.9970 | 1.0000 | 1.0065 | 1.0020 | 1.0088 |
| table-iterator-null | 1.0204 | 1.0038 | 1.0071 | 0.9887 | 1.0080 | 0.9972 | 1.0265 | 1.0095 | 0.9978 |
| table-iterator-symbol | 1.0727 | 1.0065 | 0.9978 | 0.9332 | 0.9915 | 1.0277 | 0.9713 | 0.9988 | 0.9805 |
| literal-integer | 0.9986 | 1.0057 | 1.0084 | 1.0187 | 1.0205 | 1.0273 | 1.0403 | 1.0002 | 0.9966 |
| literal-string | 1.0202 | 1.0657 | 1.0659 | 1.0322 | 0.9948 | 0.9854 | 1.0366 | 0.9925 | 1.0156 |
| symbol-ownvalue-read | 0.9828 | 1.0070 | 1.0322 | 1.0400 | 1.0149 | 0.9908 | 1.0316 | 1.0032 | 0.9998 |
| symbol-ownvalue-guarded-read | 0.9817 | 1.0026 | 1.0153 | 1.0136 | 0.9942 | 0.9895 | 1.0130 | 1.0139 | 1.0044 |
| set-ownvalue | 0.9951 | 0.9950 | 1.0009 | 1.0030 | 1.0033 | 1.0029 | 1.0076 | 0.9961 | 1.0021 |
| list-literal-small | 0.9935 | 1.0067 | 1.0168 | 1.0185 | 1.0152 | 1.0116 | 1.0253 | 1.0027 | 1.0125 |
| head-list | 0.9972 | 1.0022 | 1.0008 | 1.0036 | 1.0011 | 1.0048 | 1.0039 | 1.0038 | 1.0039 |
| length-list | 0.9970 | 0.9999 | 1.0001 | 1.0026 | 1.0010 | 1.0044 | 1.0064 | 1.0010 | 1.0083 |
| part-list | 0.9978 | 0.9986 | 0.9960 | 1.0003 | 0.9965 | 1.0050 | 0.9992 | 1.0025 | 0.9998 |
| sameq-integer | 1.0026 | 1.0007 | 0.9949 | 1.0000 | 0.9961 | 1.0038 | 0.9957 | 0.9927 | 0.9979 |
| sameq-list | 1.0026 | 1.0022 | 0.9988 | 1.0023 | 0.9995 | 0.9994 | 1.0022 | 1.0054 | 1.0046 |
| if-true | 0.9956 | 0.9987 | 1.0042 | 1.0094 | 1.0032 | 0.9995 | 1.0019 | 0.9983 | 1.0062 |
| module-local | 0.9975 | 1.0006 | 0.9990 | 1.0017 | 0.9989 | 0.9968 | 1.0106 | 0.9987 | 1.0033 |
| module-chain-escape-small | 1.0025 | 0.9853 | 0.9811 | 0.9653 | 0.9979 | 0.9848 | 1.0098 | 0.9892 | 1.0041 |
| module-chain-escape-large | 1.0047 | 0.9965 | 0.9916 | 0.9885 | 0.9953 | 0.9862 | 1.0048 | 0.9940 | 0.9933 |
| block-local | 0.9976 | 1.0002 | 1.0043 | 1.0024 | 1.0008 | 0.9992 | 1.0002 | 0.9969 | 1.0065 |
| block-symbol-only-repeat | 0.9966 | 1.0070 | 1.0089 | 1.0118 | 1.0028 | 1.0156 | 1.0205 | 0.9824 | 1.0086 |
| block-init-null-repeat | 1.0090 | 0.9979 | 1.0061 | 0.9975 | 0.9971 | 1.0055 | 0.9910 | 0.9975 | 1.0031 |
| block-init-read-repeat | 1.0010 | 0.9970 | 1.0031 | 0.9996 | 1.0017 | 1.0038 | 0.9969 | 1.0006 | 1.0088 |
| with-local | 0.9985 | 1.0011 | 1.0011 | 1.0001 | 0.9981 | 0.9996 | 0.9973 | 0.9984 | 1.0052 |
| table-small | 1.0072 | 1.0036 | 1.0002 | 0.9931 | 0.9896 | 0.9876 | 0.9990 | 0.9900 | 1.0113 |
| table-iterator-plus | 1.0117 | 0.9894 | 0.9804 | 0.9680 | 0.9752 | 0.9990 | 0.9812 | 1.0154 | 0.9862 |
| table-iterator-pair-list | 0.9962 | 0.9936 | 0.9929 | 1.0090 | 0.9993 | 0.9993 | 1.0076 | 0.9967 | 0.9896 |
| do-small | 0.9954 | 0.9985 | 0.9902 | 0.9936 | 0.9937 | 0.9927 | 0.9996 | 0.9951 | 1.0017 |
| downvalue-exact-hit | 0.9949 | 0.9949 | 1.0021 | 0.9997 | 1.0052 | 0.9947 | 0.9942 | 1.0010 | 1.0014 |
| downvalue-exact-miss | 0.9931 | 0.9918 | 0.9969 | 1.0052 | 0.9977 | 1.0051 | 1.0051 | 0.9988 | 1.0078 |
| downvalue-pattern-hit | 1.0039 | 0.9970 | 0.9935 | 0.9863 | 0.9943 | 1.0030 | 0.9908 | 0.9975 | 1.0034 |
| downvalue-pattern-miss | 0.9986 | 0.9978 | 0.9964 | 1.0069 | 1.0055 | 0.9972 | 1.0077 | 1.0126 | 1.0062 |
| downvalue-exact-miss-repeat | 1.0001 | 1.0084 | 1.0098 | 1.0080 | 1.0102 | 0.9991 | 1.0079 | 1.0017 | 1.0114 |
| downvalue-pattern-miss-repeat | 0.9982 | 1.0028 | 1.0163 | 1.0167 | 1.0109 | 1.0091 | 1.0013 | 0.9986 | 1.0301 |
| downvalue-guarded-hit | 0.9997 | 1.0021 | 0.9987 | 0.9976 | 0.9974 | 0.9973 | 0.9964 | 0.9949 | 0.9987 |
| downvalue-guarded-decline-fallback | 0.9993 | 0.9993 | 0.9961 | 0.9961 | 0.9988 | 1.0001 | 0.9950 | 0.9950 | 1.0011 |
| upvalue-guarded-hit | 1.0021 | 1.0000 | 1.0048 | 1.0027 | 1.0047 | 1.0041 | 0.9966 | 0.9990 | 0.9979 |
| subvalue-unguarded-hit | 1.0016 | 1.0024 | 1.0006 | 0.9982 | 0.9995 | 0.9963 | 1.0002 | 0.9988 | 0.9978 |
| subvalue-unguarded-miss | 0.9992 | 0.9968 | 1.0011 | 1.0037 | 1.0078 | 0.9961 | 1.0016 | 1.0010 | 0.9986 |
| subvalue-guarded-hit | 1.0019 | 1.0016 | 1.0026 | 1.0008 | 0.9986 | 0.9985 | 0.9996 | 1.0009 | 0.9948 |
| default-unguarded-hit | 0.9984 | 0.9977 | 0.9968 | 0.9981 | 0.9976 | 0.9977 | 1.0057 | 0.9998 | 0.9985 |
| default-unguarded-miss | 0.9978 | 0.9980 | 0.9983 | 1.0003 | 0.9998 | 1.0037 | 1.0060 | 0.9982 | 0.9982 |
| default-guarded-hit | 1.0037 | 0.9988 | 0.9945 | 0.9988 | 0.9979 | 1.0034 | 1.0001 | 0.9957 | 1.0048 |
| matchq-integer-success | 0.9833 | 0.9963 | 0.9950 | 1.0009 | 0.9947 | 1.0151 | 0.9975 | 0.9980 | 0.9946 |
| matchq-integer-fail | 0.9913 | 0.9968 | 0.9972 | 0.9997 | 0.9994 | 1.0171 | 1.0003 | 0.9921 | 0.9983 |
| matcher-single-tiny | 0.9985 | 1.0003 | 0.9971 | 0.9981 | 0.9995 | 1.0011 | 1.0002 | 0.9998 | 0.9935 |
| matcher-single-wide-small | 1.0033 | 0.9992 | 1.0116 | 1.0088 | 1.0045 | 0.9767 | 1.0094 | 0.9964 | 0.9989 |
| matcher-single-wide-large | 1.0053 | 1.0031 | 1.0121 | 1.0074 | 1.0062 | 0.9794 | 1.0019 | 1.0010 | 0.9970 |
| replaceall-simple | 0.9928 | 0.9952 | 0.9891 | 0.9963 | 0.9953 | 1.0018 | 0.9898 | 0.9936 | 1.0014 |
| rewrite-guarded-hit | 0.9983 | 0.9973 | 1.0039 | 1.0053 | 1.0029 | 0.9940 | 0.9950 | 1.0003 | 0.9919 |
| rewrite-guarded-decline-fallback | 0.9745 | 0.9943 | 0.9764 | 1.0025 | 0.9812 | 0.9966 | 1.0017 | 0.9962 | 0.9943 |
| cases-wide-no-match | 1.0021 | 0.9963 | 0.9963 | 0.9992 | 1.0030 | 0.9742 | 1.0352 | 0.9966 | 0.9916 |
| count-wide-no-match | 1.0020 | 1.0052 | 0.9976 | 0.9994 | 1.0005 | 1.0062 | 1.0094 | 0.9959 | 1.0016 |
| position-wide-no-match | 0.9987 | 0.9986 | 1.0031 | 0.9988 | 0.9988 | 0.9951 | 0.9963 | 1.0025 | 1.0006 |
| deletecases-wide-no-match | 1.0030 | 0.9978 | 0.9952 | 0.9898 | 0.9971 | 1.0075 | 0.9880 | 0.9989 | 1.0028 |
| deletecases-wide-association-no-match | 1.0210 | 1.0280 | 1.0394 | 1.0076 | 0.9858 | 1.0055 | 1.0056 | 0.9850 | 0.9914 |
| replace-wide-no-match | 1.0023 | 0.9670 | 0.9651 | 0.9609 | 1.0006 | 1.0080 | 1.0031 | 0.9997 | 1.0066 |
| replacepart-wide-rule-list | 0.9993 | 1.0028 | 1.0053 | 1.0114 | 1.0065 | 1.0092 | 1.0010 | 1.0035 | 1.0026 |
| replacepart-guarded-hit | 0.9987 | 1.0006 | 1.0038 | 1.0026 | 1.0007 | 0.9986 | 1.0081 | 1.0077 | 1.0081 |
| replacepart-guarded-decline-fallback | 1.0013 | 1.0044 | 1.0019 | 0.9986 | 0.9948 | 1.0052 | 1.0041 | 0.9982 | 1.0028 |
| string-repeat-small | 0.9918 | 1.0013 | 1.0087 | 1.0132 | 1.0045 | 0.9993 | 1.0123 | 0.9928 | 1.0090 |
| character-range-small | 0.9973 | 1.0061 | 0.9971 | 0.9988 | 0.9965 | 1.0099 | 0.9887 | 0.9948 | 1.0134 |
| stringtrim-literal-list-4000 | 1.0029 | 0.9988 | 1.0006 | 0.9958 | 1.0001 | 0.9990 | 1.0023 | 0.9990 | 1.0070 |
| stringtrim-literal-list-32000 | 0.9985 | 1.0012 | 0.9985 | 0.9994 | 0.9970 | 1.0002 | 0.9961 | 1.0013 | 1.0061 |
| stringtrim-single-literal-4000 | 1.0029 | 1.0012 | 1.0004 | 1.0015 | 1.0007 | 0.9952 | 1.0038 | 0.9966 | 0.9987 |
| stringtrim-single-literal-32000 | 0.9998 | 1.0002 | 1.0005 | 0.9998 | 0.9998 | 1.0008 | 0.9995 | 0.9978 | 0.9997 |
| stringtrim-whitespace-4000 | 1.0014 | 0.9980 | 1.0037 | 0.9980 | 1.0006 | 1.0003 | 0.9938 | 1.0012 | 1.0004 |
| stringtrim-whitespace-32000 | 1.0013 | 1.0002 | 0.9988 | 0.9981 | 0.9990 | 1.0010 | 1.0018 | 1.0005 | 1.0007 |
| stringreplace-single-literal-unguarded | 0.9981 | 0.9943 | 1.0076 | 1.0079 | 1.0001 | 0.9906 | 1.0082 | 1.0055 | 0.9974 |
| stringreplace-pattern-unguarded | 0.9987 | 0.9952 | 0.9982 | 1.0021 | 1.0035 | 0.9960 | 0.9981 | 0.9966 | 1.0009 |
| stringreplace-guarded-hit | 1.0002 | 1.0021 | 1.0025 | 1.0014 | 1.0003 | 0.9917 | 1.0010 | 0.9974 | 1.0007 |
| stringreplace-guarded-decline-fallback | 0.9984 | 0.9984 | 1.0007 | 0.9965 | 1.0011 | 1.0014 | 0.9930 | 0.9993 | 1.0045 |
| appendto-list-incremental-small | 1.0004 | 1.0018 | 1.0206 | 0.9985 | 1.0019 | 0.9916 | 0.9993 | 0.9948 | 1.0074 |
| appendto-list-incremental-large | 0.9952 | 1.0004 | 0.9459 | 1.0020 | 0.9370 | 0.9955 | 1.0006 | 0.8738 | 1.0026 |
| appendto-part-outer1000-small | 0.9984 | 1.0074 | 0.9941 | 0.9993 | 0.9885 | 1.0012 | 0.9987 | 0.9856 | 0.9960 |
| appendto-part-outer1000-large | 0.9984 | 0.9995 | 0.9982 | 0.9994 | 0.9973 | 0.9986 | 0.9995 | 0.9969 | 0.9984 |
| association-set-key-incremental-small | 0.9994 | 1.0023 | 1.0018 | 1.0022 | 1.0003 | 0.9993 | 1.0012 | 0.9995 | 1.0011 |
| association-set-key-incremental-large | 0.9993 | 0.9986 | 1.0008 | 1.0013 | 1.0014 | 1.0005 | 1.0014 | 1.0013 | 0.9985 |
| association-small-literal-repeat | 1.0042 | 0.9984 | 1.0021 | 0.9905 | 1.0007 | 0.9930 | 0.9987 | 1.0074 | 1.0029 |
| set-part-fixed100-len100 | 1.0013 | 1.0066 | 1.0017 | 1.0043 | 1.0013 | 0.9989 | 1.0052 | 1.0025 | 1.0170 |
| set-part-fixed100-len800 | 0.9957 | 1.0002 | 0.9984 | 0.9978 | 0.9967 | 1.0022 | 0.9987 | 0.9954 | 1.0201 |
| set-part-fill-small | 0.9963 | 0.9873 | 1.0010 | 1.0057 | 1.0123 | 0.9901 | 1.0032 | 1.0021 | 1.0156 |
| set-part-fill-large | 0.9916 | 0.9987 | 0.9945 | 1.0046 | 0.9982 | 1.0064 | 1.0012 | 1.0069 | 0.9985 |
| upvalue-candidates-tiny | 1.0035 | 1.0001 | 1.0076 | 1.0080 | 1.0044 | 0.9916 | 1.0073 | 0.9945 | 1.0018 |
| upvalue-candidates-wide-small | 0.9889 | 0.9904 | 0.9764 | 0.9870 | 0.9806 | 1.0167 | 0.9974 | 0.9935 | 0.9904 |
| upvalue-candidates-wide-large | 0.9898 | 0.9895 | 0.9811 | 0.9863 | 0.9896 | 1.0174 | 0.9950 | 0.9983 | 0.9847 |
| sort-callback-small | 1.0023 | 0.9985 | 0.9997 | 1.0044 | 1.0047 | 1.0097 | 0.9964 | 1.0049 | 1.0094 |
| sort-callback-large | 1.0015 | 0.9962 | 0.9951 | 0.9982 | 1.0084 | 1.0053 | 1.0023 | 0.9883 | 1.0102 |
| sortby-callback-small | 1.0119 | 1.0018 | 0.9946 | 0.9890 | 0.9931 | 0.9946 | 1.0127 | 0.9935 | 1.0004 |
| sortby-callback-large | 1.0013 | 1.0023 | 0.9966 | 0.9992 | 0.9992 | 0.9994 | 1.0154 | 0.9961 | 1.0004 |
| reversesort-callback-small | 0.9976 | 0.9994 | 0.9985 | 1.0020 | 1.0092 | 0.9966 | 1.0109 | 0.9987 | 1.0090 |
| reversesort-callback-large | 0.9992 | 0.9987 | 1.0012 | 1.0002 | 1.0015 | 1.0014 | 1.0058 | 1.0020 | 1.0050 |
| keysort-callback-small | 1.0021 | 1.0090 | 0.9990 | 0.9925 | 0.9925 | 1.0240 | 1.0022 | 1.0016 | 1.0102 |
| keysort-callback-large | 0.9934 | 1.0072 | 0.9957 | 0.9996 | 0.9922 | 1.0074 | 0.9960 | 1.0010 | 1.0170 |
| keysortby-callback-small | 0.9978 | 0.9972 | 0.9989 | 1.0011 | 0.9970 | 0.9980 | 0.9974 | 0.9932 | 1.0088 |
| keysortby-callback-large | 0.9967 | 1.0007 | 0.9982 | 0.9986 | 1.0000 | 1.0064 | 0.9947 | 1.0071 | 1.0017 |
| sort-default-wide | 0.9997 | 0.9986 | 1.0000 | 0.9998 | 0.9968 | 0.9972 | 0.9975 | 1.0006 | 1.0029 |
| sort-symbol-default-width4000 | 0.9739 | 0.9895 | 0.9789 | 1.0008 | 1.0038 | 0.9981 | 0.9928 | 0.9986 | 1.0001 |
| sort-symbol-default-width16000 | 0.9973 | 0.9951 | 0.9970 | 1.0046 | 1.0073 | 0.9959 | 0.9965 | 1.0070 | 0.9962 |
| sortby-default-wide | 1.0047 | 1.0040 | 1.0000 | 0.9954 | 1.0024 | 1.0092 | 0.9988 | 0.9970 | 1.0001 |
| reversesort-default-wide | 0.9987 | 1.0008 | 0.9991 | 1.0003 | 1.0029 | 0.9971 | 0.9969 | 0.9957 | 1.0008 |
| keysort-default-wide | 0.9925 | 1.0039 | 1.0117 | 1.0035 | 0.9996 | 0.9870 | 0.9991 | 0.9823 | 0.9946 |
| keysortby-default-wide | 0.9895 | 1.0008 | 0.9845 | 0.9943 | 0.9965 | 0.9970 | 1.0049 | 0.9946 | 1.0007 |
| valueq-undefined-symbol-definitions-2000 | 0.9968 | 0.9997 | 0.9954 | 1.0007 | 1.0016 | 0.9938 | 0.9968 | 0.9983 | 0.9952 |
| valueq-undefined-symbol-definitions-8000 | 0.9959 | 0.9942 | 0.9972 | 1.0027 | 0.9999 | 0.9990 | 0.9959 | 1.0049 | 0.9954 |
| module-retained-gensym-gc-batch | 1.0075 | 1.0150 | 0.9872 | 0.9655 | 0.9865 | 0.9955 | 0.9562 | 1.0073 | 0.9921 |

## focus

| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| focus-increment-1024 | 0.8792 | 1.0017 | 0.8758 | 0.9967 | 0.8764 | 0.9924 | 0.9986 | 1.0090 | 0.9943 |
| focus-increment-16000 | 0.8821 | 1.0022 | 0.8773 | 0.9981 | 0.8822 | 0.9879 | 1.0080 | 1.0042 | 0.9931 |
| focus-increment-reversed-1024 | 0.8854 | 1.0001 | 0.8786 | 0.9977 | 0.8870 | 0.9981 | 1.0084 | 1.0060 | 0.9956 |
| focus-increment-reversed-16000 | 0.8816 | 1.0000 | 0.8752 | 1.0002 | 0.8770 | 0.9939 | 1.0035 | 1.0038 | 0.9912 |
| focus-identity-1024 | 0.6213 | 0.9984 | 0.6164 | 0.9824 | 0.6137 | 0.9944 | 1.0040 | 1.0010 | 0.9870 |
| focus-identity-16000 | 0.6017 | 0.9955 | 0.6026 | 0.9823 | 0.5993 | 0.9990 | 1.0075 | 0.9994 | 0.9798 |
| focus-miss-1024 | 1.0024 | 0.9725 | 0.9730 | 0.9707 | 1.0006 | 1.0060 | 0.9993 | 0.9959 | 1.0024 |
| focus-miss-16000 | 1.0030 | 0.9668 | 0.9728 | 0.9708 | 1.0032 | 1.0010 | 0.9985 | 1.0035 | 0.9962 |
| focus-opaque-body-1024 | 0.6492 | 0.9650 | 0.6479 | 1.0075 | 0.6684 | 1.0113 | 0.9923 | 0.9853 | 0.9898 |
| focus-opaque-body-16000 | 0.7173 | 0.9980 | 0.7191 | 1.0036 | 0.7192 | 0.9990 | 1.0029 | 0.9972 | 0.9942 |
| focus-flat-nested-1024 | 0.9156 | 0.9925 | 0.9175 | 0.9993 | 0.9314 | 0.9956 | 0.9990 | 1.0015 | 0.9967 |
| focus-flat-nested-16000 | 0.9150 | 0.9959 | 0.9097 | 0.9947 | 0.9277 | 0.9886 | 0.9986 | 1.0027 | 1.0013 |

## pipeline

| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| synthetic-rewrite-heavy-hit-4000 | 0.8835 | 1.0060 | 0.8796 | 1.0031 | 0.8816 | 0.9921 | 0.9968 | 0.9905 | 0.9856 |
| synthetic-rewrite-heavy-hit-16000 | 0.8755 | 0.9922 | 0.8793 | 1.0062 | 0.8842 | 0.9936 | 0.9992 | 0.9610 | 0.9887 |
| synthetic-mixed-hit-4000 | 0.8896 | 1.0021 | 0.8955 | 1.0037 | 0.8916 | 0.9907 | 1.0018 | 0.9880 | 0.9921 |
| synthetic-mixed-hit-16000 | 0.8732 | 1.0081 | 0.8806 | 1.0022 | 0.8716 | 0.9881 | 0.9981 | 0.9990 | 0.9814 |
| synthetic-mixed-control-4000 | 0.9853 | 0.9914 | 0.9804 | 1.0087 | 0.9814 | 0.9716 | 0.9891 | 1.0121 | 1.0048 |
| synthetic-mixed-control-16000 | 0.7158 | 1.0011 | 0.6345 | 0.8515 | 0.6327 | 0.9945 | 0.9962 | 1.0002 | 0.9966 |

## shape

| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| shape-chain-16-positive | 0.9806 | 0.9644 | 0.9562 | 0.9764 | 1.0066 | 1.0071 | 1.0011 | 0.9835 | 0.9911 |
| shape-chain-16-negative | 0.9946 | 0.9908 | 0.9993 | 0.9991 | 0.9918 | 0.9925 | 1.0004 | 0.9898 | 0.9969 |
| shape-chain-32-positive | 0.9995 | 0.9297 | 0.9184 | 0.9225 | 0.9908 | 0.9896 | 0.9863 | 0.9943 | 1.0034 |
| shape-chain-32-negative | 0.9968 | 0.9972 | 0.9931 | 0.9946 | 0.9967 | 0.9927 | 0.9967 | 1.0009 | 0.9983 |
| shape-chain-64-positive | 0.9988 | 0.8420 | 0.8343 | 0.8361 | 0.9971 | 0.9991 | 0.9955 | 1.0015 | 1.0057 |
| shape-chain-64-negative | 1.0012 | 0.9933 | 1.0004 | 0.9996 | 1.0074 | 1.0002 | 1.0037 | 1.0035 | 0.9965 |
| shape-binary-5-positive | 1.0026 | 0.9585 | 0.9665 | 0.9672 | 1.0026 | 0.9925 | 1.0142 | 1.0046 | 0.9941 |
| shape-binary-5-negative | 1.0003 | 0.9993 | 0.9976 | 1.0025 | 0.9949 | 0.9950 | 0.9978 | 1.0075 | 1.0053 |
| shape-binary-7-positive | 0.9912 | 0.9257 | 0.9386 | 0.9406 | 1.0011 | 0.9848 | 1.0077 | 1.0126 | 1.0087 |
| shape-binary-7-negative | 0.9781 | 0.9899 | 0.9952 | 1.0077 | 1.0084 | 0.9905 | 0.9988 | 1.0166 | 1.0159 |
| shape-binary-9-positive | 1.0014 | 0.9237 | 0.9234 | 0.9256 | 1.0038 | 0.9823 | 0.9983 | 1.0020 | 0.9855 |
| shape-binary-9-negative | 0.9915 | 0.9888 | 0.9925 | 1.0050 | 1.0022 | 0.9958 | 0.9978 | 1.0125 | 0.9888 |

All slower original-suite medians (including very small differences):

{
  "matcher/base": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width32000",
    "do-count-null",
    "table-count-null",
    "table-iterator-null",
    "table-iterator-symbol",
    "literal-string",
    "sameq-integer",
    "sameq-list",
    "module-chain-escape-small",
    "module-chain-escape-large",
    "block-init-null-repeat",
    "block-init-read-repeat",
    "table-small",
    "table-iterator-plus",
    "downvalue-pattern-hit",
    "downvalue-exact-miss-repeat",
    "upvalue-guarded-hit",
    "subvalue-unguarded-hit",
    "subvalue-guarded-hit",
    "default-guarded-hit",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "cases-wide-no-match",
    "count-wide-no-match",
    "deletecases-wide-no-match",
    "deletecases-wide-association-no-match",
    "replace-wide-no-match",
    "replacepart-guarded-decline-fallback",
    "stringtrim-literal-list-4000",
    "stringtrim-single-literal-4000",
    "stringtrim-whitespace-4000",
    "stringtrim-whitespace-32000",
    "stringreplace-guarded-hit",
    "appendto-list-incremental-small",
    "association-small-literal-repeat",
    "set-part-fixed100-len100",
    "upvalue-candidates-tiny",
    "sort-callback-small",
    "sort-callback-large",
    "sortby-callback-small",
    "sortby-callback-large",
    "keysort-callback-small",
    "sortby-default-wide",
    "module-retained-gensym-gc-batch"
  ],
  "depth/base": [
    "association-part-key-read-width4000",
    "association-part-key-read-width32000",
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
    "head-list",
    "sameq-integer",
    "sameq-list",
    "module-local",
    "block-local",
    "block-symbol-only-repeat",
    "with-local",
    "table-small",
    "downvalue-exact-miss-repeat",
    "downvalue-pattern-miss-repeat",
    "downvalue-guarded-hit",
    "subvalue-unguarded-hit",
    "subvalue-guarded-hit",
    "matcher-single-tiny",
    "matcher-single-wide-large",
    "count-wide-no-match",
    "deletecases-wide-association-no-match",
    "replacepart-wide-rule-list",
    "replacepart-guarded-hit",
    "replacepart-guarded-decline-fallback",
    "string-repeat-small",
    "character-range-small",
    "stringtrim-literal-list-32000",
    "stringtrim-single-literal-4000",
    "stringtrim-single-literal-32000",
    "stringtrim-whitespace-32000",
    "stringreplace-guarded-hit",
    "appendto-list-incremental-small",
    "appendto-list-incremental-large",
    "appendto-part-outer1000-small",
    "association-set-key-incremental-small",
    "set-part-fixed100-len100",
    "set-part-fixed100-len800",
    "upvalue-candidates-tiny",
    "sortby-callback-small",
    "sortby-callback-large",
    "keysort-callback-small",
    "keysort-callback-large",
    "keysortby-callback-large",
    "sortby-default-wide",
    "reversesort-default-wide",
    "keysort-default-wide",
    "keysortby-default-wide",
    "module-retained-gensym-gc-batch"
  ],
  "combined/base": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width4000",
    "loop-empty",
    "do-count-null",
    "table-count-null",
    "table-iterator-null",
    "literal-integer",
    "literal-string",
    "symbol-ownvalue-read",
    "symbol-ownvalue-guarded-read",
    "set-ownvalue",
    "list-literal-small",
    "head-list",
    "length-list",
    "if-true",
    "block-local",
    "block-symbol-only-repeat",
    "block-init-null-repeat",
    "block-init-read-repeat",
    "with-local",
    "table-small",
    "downvalue-exact-hit",
    "downvalue-exact-miss-repeat",
    "downvalue-pattern-miss-repeat",
    "upvalue-guarded-hit",
    "subvalue-unguarded-hit",
    "subvalue-unguarded-miss",
    "subvalue-guarded-hit",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "rewrite-guarded-hit",
    "position-wide-no-match",
    "deletecases-wide-association-no-match",
    "replacepart-wide-rule-list",
    "replacepart-guarded-hit",
    "replacepart-guarded-decline-fallback",
    "string-repeat-small",
    "stringtrim-literal-list-4000",
    "stringtrim-single-literal-4000",
    "stringtrim-single-literal-32000",
    "stringtrim-whitespace-4000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "appendto-list-incremental-small",
    "association-set-key-incremental-small",
    "association-set-key-incremental-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len100",
    "set-part-fill-small",
    "upvalue-candidates-tiny",
    "reversesort-callback-large",
    "sort-default-wide",
    "sortby-default-wide",
    "keysort-default-wide"
  ],
  "combined/matcher": [
    "association-part-key-read-width4000",
    "loop-empty",
    "do-count-null",
    "literal-integer",
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
    "module-local",
    "block-local",
    "block-symbol-only-repeat",
    "with-local",
    "table-iterator-pair-list",
    "downvalue-exact-miss",
    "downvalue-pattern-miss",
    "downvalue-exact-miss-repeat",
    "downvalue-pattern-miss-repeat",
    "upvalue-guarded-hit",
    "subvalue-unguarded-miss",
    "subvalue-guarded-hit",
    "default-unguarded-miss",
    "matchq-integer-success",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "rewrite-guarded-hit",
    "rewrite-guarded-decline-fallback",
    "deletecases-wide-association-no-match",
    "replacepart-wide-rule-list",
    "replacepart-guarded-hit",
    "string-repeat-small",
    "stringtrim-single-literal-4000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-pattern-unguarded",
    "stringreplace-guarded-hit",
    "appendto-list-incremental-large",
    "association-set-key-incremental-small",
    "association-set-key-incremental-large",
    "set-part-fixed100-len100",
    "set-part-fill-small",
    "set-part-fill-large",
    "upvalue-candidates-tiny",
    "sort-callback-small",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysortby-callback-small",
    "sort-symbol-default-width4000",
    "sort-symbol-default-width16000",
    "reversesort-default-wide",
    "keysort-default-wide",
    "valueq-undefined-symbol-definitions-2000",
    "valueq-undefined-symbol-definitions-8000"
  ],
  "combined/depth": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width32000",
    "do-count-null",
    "table-iterator-null",
    "literal-integer",
    "symbol-ownvalue-read",
    "set-ownvalue",
    "list-literal-small",
    "head-list",
    "length-list",
    "if-true",
    "block-local",
    "block-symbol-only-repeat",
    "block-init-read-repeat",
    "downvalue-exact-hit",
    "downvalue-pattern-miss",
    "downvalue-exact-miss-repeat",
    "downvalue-pattern-miss-repeat",
    "upvalue-guarded-hit",
    "subvalue-unguarded-miss",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "rewrite-guarded-hit",
    "cases-wide-no-match",
    "count-wide-no-match",
    "replace-wide-no-match",
    "replacepart-wide-rule-list",
    "replacepart-guarded-hit",
    "string-repeat-small",
    "stringtrim-literal-list-4000",
    "stringtrim-single-literal-4000",
    "stringtrim-whitespace-4000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-pattern-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "appendto-list-incremental-small",
    "association-set-key-incremental-small",
    "association-set-key-incremental-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len100",
    "set-part-fill-small",
    "upvalue-candidates-tiny",
    "sort-callback-small",
    "sort-callback-large",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysortby-callback-large",
    "sort-symbol-default-width4000",
    "sort-symbol-default-width16000",
    "sortby-default-wide",
    "reversesort-default-wide",
    "valueq-undefined-symbol-definitions-2000"
  ]
}

One nominally idle desktop and one shared executable; all modes pay common cached-switch overhead.
Eight paired process blocks; retained in-process repetitions are not independent experiment blocks.
Every slower original-suite median is retained; A/A ratios are controls and are never subtracted.
Same bytes hold relative code offsets fixed; absolute addresses still vary under ASLR. This is not an untouched upstream executable gate.
Synthetic focus/pipeline programs do not establish a real application distribution or whole-runtime speedup.
Value, alias, continuation, trace and selected resource-limit checks are finite evidence; this is not full contextual equivalence.
