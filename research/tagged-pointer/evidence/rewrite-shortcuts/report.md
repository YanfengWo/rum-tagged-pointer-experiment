# Safe rewrite shortcut validation

Elapsed ratios below one are faster. Every original label and all A/A controls are retained.

## Original suite

| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| association-part-key-read-width4000 | 0.9974 | 1.0041 | 1.0041 | 1.0068 | 1.0032 | 1.0049 | 0.9937 | 0.9972 | 0.9960 |
| association-part-key-read-width32000 | 0.9968 | 0.9980 | 0.9972 | 1.0013 | 0.9995 | 1.0001 | 0.9990 | 0.9995 | 0.9964 |
| association-part-positional-read-width4000 | 0.9982 | 0.9908 | 0.9987 | 0.9994 | 1.0083 | 0.9862 | 1.0020 | 1.0012 | 1.0174 |
| association-part-positional-read-width32000 | 1.0003 | 0.9930 | 0.9976 | 1.0047 | 1.0039 | 0.9812 | 1.0028 | 0.9906 | 1.0319 |
| loop-empty | 0.9983 | 1.0005 | 0.9959 | 0.9853 | 1.0002 | 1.0241 | 1.0202 | 1.0052 | 0.9673 |
| do-count-null | 1.0018 | 0.9876 | 0.9881 | 0.9890 | 1.0022 | 0.9953 | 1.0388 | 1.0010 | 0.9913 |
| do-iterator-null | 0.9909 | 0.9962 | 0.9939 | 1.0042 | 0.9984 | 0.9421 | 1.0015 | 0.9954 | 1.0314 |
| table-count-null | 0.9961 | 0.9850 | 0.9921 | 1.0045 | 1.0076 | 1.0006 | 0.9687 | 1.0020 | 1.0050 |
| table-iterator-null | 1.0128 | 1.0128 | 0.9983 | 0.9870 | 0.9932 | 0.9967 | 1.0026 | 1.0231 | 0.9879 |
| table-iterator-symbol | 1.0039 | 0.9869 | 0.9983 | 0.9657 | 1.0069 | 0.9947 | 0.9938 | 1.0156 | 1.0022 |
| literal-integer | 0.9985 | 1.0010 | 0.9847 | 0.9856 | 0.9977 | 1.0146 | 0.9990 | 1.0116 | 0.9702 |
| literal-string | 0.9680 | 0.9918 | 0.9501 | 1.0141 | 0.9988 | 1.0408 | 1.0365 | 1.0621 | 0.9826 |
| symbol-ownvalue-read | 1.0093 | 1.0018 | 0.9944 | 0.9819 | 0.9990 | 1.0411 | 1.0145 | 0.9939 | 0.9648 |
| symbol-ownvalue-guarded-read | 1.0136 | 0.9982 | 1.0084 | 0.9921 | 1.0146 | 1.0330 | 0.9929 | 0.9967 | 0.9685 |
| set-ownvalue | 0.9692 | 0.9895 | 0.9722 | 0.9949 | 0.9794 | 1.0038 | 1.0014 | 1.0034 | 0.9930 |
| list-literal-small | 0.9889 | 0.9965 | 0.9871 | 0.9921 | 0.9929 | 1.0205 | 1.0030 | 1.0066 | 0.9812 |
| head-list | 0.9881 | 0.9963 | 0.9925 | 1.0075 | 0.9993 | 1.0078 | 1.0018 | 1.0118 | 1.0105 |
| length-list | 0.9908 | 0.9986 | 0.9949 | 1.0049 | 0.9971 | 1.0107 | 0.9958 | 1.0113 | 1.0105 |
| part-list | 0.9950 | 0.9994 | 0.9975 | 1.0012 | 0.9975 | 0.9983 | 1.0081 | 1.0047 | 1.0188 |
| sameq-integer | 0.9882 | 0.9939 | 0.9901 | 0.9896 | 0.9933 | 1.0063 | 1.0019 | 1.0236 | 1.0100 |
| sameq-list | 0.9911 | 0.9978 | 0.9975 | 1.0087 | 1.0058 | 1.0039 | 1.0023 | 1.0086 | 1.0078 |
| if-true | 0.9901 | 0.9872 | 0.9945 | 1.0084 | 1.0072 | 0.9921 | 1.0154 | 1.0075 | 1.0234 |
| module-local | 1.0057 | 0.9924 | 1.0108 | 1.0067 | 1.0181 | 1.0106 | 0.9944 | 0.9984 | 1.0297 |
| module-chain-escape-small | 1.0062 | 0.9822 | 1.0214 | 1.0150 | 1.0423 | 0.9482 | 0.9867 | 0.9873 | 1.0504 |
| module-chain-escape-large | 1.0099 | 1.0017 | 1.0132 | 0.9963 | 0.9993 | 1.0030 | 0.9980 | 1.0088 | 1.0055 |
| block-local | 0.9957 | 0.9976 | 0.9992 | 1.0044 | 1.0001 | 1.0103 | 1.0162 | 1.0028 | 1.0400 |
| block-symbol-only-repeat | 0.9863 | 0.9933 | 0.9948 | 1.0121 | 1.0030 | 1.0034 | 1.0035 | 1.0050 | 1.0312 |
| block-init-null-repeat | 0.9916 | 0.9836 | 1.0020 | 1.0092 | 1.0144 | 1.0097 | 1.0096 | 1.0143 | 1.0214 |
| block-init-read-repeat | 0.9795 | 0.9914 | 0.9859 | 1.0128 | 0.9926 | 0.9987 | 1.0041 | 1.0056 | 1.0070 |
| with-local | 0.9870 | 0.9947 | 1.0010 | 1.0132 | 1.0052 | 1.0031 | 1.0004 | 0.9943 | 1.0105 |
| table-small | 1.0099 | 1.0279 | 1.0067 | 1.0069 | 0.9819 | 0.9800 | 0.9978 | 0.9934 | 1.0173 |
| table-iterator-plus | 1.0203 | 1.0157 | 1.0134 | 0.9881 | 1.0087 | 0.9946 | 1.0065 | 0.9667 | 1.0352 |
| table-iterator-pair-list | 1.0080 | 1.0029 | 0.9984 | 0.9881 | 0.9868 | 0.9980 | 1.0132 | 1.0043 | 1.0097 |
| do-small | 0.9993 | 1.0017 | 1.0095 | 1.0070 | 1.0084 | 0.9969 | 0.9960 | 0.9957 | 1.0125 |
| downvalue-exact-hit | 1.0084 | 0.9992 | 1.0111 | 1.0023 | 1.0170 | 1.0157 | 1.0022 | 1.0050 | 1.0199 |
| downvalue-exact-miss | 1.0004 | 0.9986 | 0.9954 | 0.9973 | 1.0005 | 1.0183 | 0.9929 | 1.0192 | 1.0096 |
| downvalue-pattern-hit | 0.9926 | 0.9966 | 1.0062 | 1.0131 | 1.0083 | 1.0090 | 0.9921 | 0.9891 | 1.0440 |
| downvalue-pattern-miss | 0.9970 | 0.9831 | 0.9982 | 0.9977 | 1.0138 | 1.0139 | 0.9972 | 1.0280 | 1.0125 |
| downvalue-exact-miss-repeat | 0.9965 | 0.9839 | 0.9991 | 1.0109 | 1.0088 | 1.0196 | 1.0015 | 1.0018 | 1.0035 |
| downvalue-pattern-miss-repeat | 0.9922 | 0.9897 | 0.9990 | 1.0157 | 1.0138 | 1.0160 | 0.9931 | 1.0036 | 1.0091 |
| downvalue-guarded-hit | 0.9802 | 0.9988 | 1.0006 | 1.0185 | 1.0011 | 1.0090 | 0.9854 | 0.9983 | 1.0304 |
| downvalue-guarded-decline-fallback | 0.9908 | 0.9984 | 0.9932 | 1.0028 | 0.9942 | 1.0117 | 0.9880 | 0.9792 | 1.0156 |
| upvalue-guarded-hit | 0.9872 | 0.9958 | 1.0001 | 1.0111 | 1.0044 | 0.9915 | 0.9902 | 0.9936 | 1.0372 |
| subvalue-unguarded-hit | 0.9950 | 0.9952 | 0.9987 | 1.0020 | 1.0076 | 1.0131 | 0.9900 | 1.0004 | 1.0337 |
| subvalue-unguarded-miss | 0.9890 | 0.9826 | 0.9867 | 0.9968 | 1.0073 | 1.0327 | 1.0000 | 1.0096 | 1.0333 |
| subvalue-guarded-hit | 0.9876 | 0.9950 | 0.9994 | 1.0103 | 1.0010 | 1.0065 | 0.9781 | 0.9994 | 1.0230 |
| default-unguarded-hit | 0.9873 | 0.9895 | 0.9910 | 1.0047 | 0.9997 | 0.9909 | 0.9857 | 0.9989 | 1.0131 |
| default-unguarded-miss | 0.9829 | 0.9949 | 0.9884 | 1.0070 | 0.9994 | 1.0033 | 0.9959 | 0.9999 | 1.0110 |
| default-guarded-hit | 0.9938 | 0.9819 | 0.9982 | 1.0038 | 1.0142 | 0.9917 | 0.9765 | 0.9980 | 1.0137 |
| matchq-integer-success | 0.9982 | 0.9886 | 1.0072 | 1.0108 | 1.0182 | 1.0172 | 1.0043 | 1.0084 | 1.0265 |
| matchq-integer-fail | 0.9939 | 0.9885 | 1.0010 | 1.0075 | 1.0119 | 1.0188 | 1.0002 | 1.0045 | 1.0208 |
| matcher-single-tiny | 0.9870 | 0.9924 | 0.9855 | 0.9968 | 0.9935 | 1.0175 | 0.9883 | 1.0063 | 1.0082 |
| matcher-single-wide-small | 1.0016 | 1.0036 | 1.0082 | 1.0018 | 0.9983 | 0.9914 | 1.0337 | 0.9766 | 1.0131 |
| matcher-single-wide-large | 1.0032 | 1.0055 | 1.0063 | 1.0040 | 1.0037 | 0.9950 | 1.0367 | 0.9759 | 1.0101 |
| replaceall-simple | 0.9942 | 0.9992 | 0.9897 | 0.9967 | 0.9932 | 1.0119 | 1.0019 | 1.0341 | 1.0052 |
| rewrite-guarded-hit | 0.9948 | 0.9975 | 1.0059 | 1.0095 | 1.0072 | 0.9961 | 0.9988 | 1.0031 | 1.0216 |
| rewrite-guarded-decline-fallback | 0.9747 | 0.9996 | 0.9795 | 1.0054 | 0.9799 | 1.0109 | 0.9907 | 0.9978 | 1.0133 |
| cases-wide-no-match | 1.0138 | 1.0081 | 1.0090 | 1.0059 | 1.0067 | 1.0097 | 0.9926 | 0.9839 | 0.9904 |
| count-wide-no-match | 0.9894 | 0.9921 | 0.9885 | 1.0003 | 0.9972 | 1.0017 | 1.0011 | 1.0145 | 1.0009 |
| position-wide-no-match | 1.0027 | 0.9993 | 0.9988 | 0.9961 | 1.0041 | 0.9935 | 1.0008 | 0.9963 | 1.0051 |
| deletecases-wide-no-match | 0.9966 | 1.0039 | 1.0011 | 1.0017 | 0.9959 | 1.0018 | 0.9970 | 0.9993 | 1.0073 |
| deletecases-wide-association-no-match | 0.9957 | 1.0019 | 1.0004 | 1.0023 | 0.9932 | 1.0132 | 0.9930 | 0.9957 | 1.0039 |
| replace-wide-no-match | 0.9983 | 0.9680 | 0.9673 | 0.9644 | 0.9991 | 1.0001 | 0.9995 | 0.9980 | 0.9993 |
| replacepart-wide-rule-list | 1.0098 | 0.9992 | 0.9933 | 0.9920 | 1.0021 | 0.9908 | 1.0171 | 1.0252 | 0.9955 |
| replacepart-guarded-hit | 1.0038 | 0.9981 | 1.0087 | 1.0041 | 1.0089 | 1.0292 | 0.9976 | 1.0044 | 1.0207 |
| replacepart-guarded-decline-fallback | 1.0092 | 1.0068 | 1.0051 | 0.9941 | 0.9997 | 1.0136 | 1.0096 | 1.0056 | 1.0213 |
| string-repeat-small | 0.9782 | 0.9797 | 0.9861 | 1.0065 | 1.0151 | 0.9852 | 1.0028 | 1.0209 | 1.0247 |
| character-range-small | 0.9975 | 0.9913 | 1.0030 | 1.0019 | 1.0090 | 0.9961 | 1.0183 | 1.0079 | 1.0292 |
| stringtrim-literal-list-4000 | 0.9843 | 0.9928 | 0.9759 | 0.9958 | 0.9855 | 0.9951 | 0.9954 | 0.9944 | 1.0028 |
| stringtrim-literal-list-32000 | 0.9915 | 1.0006 | 0.9901 | 0.9944 | 0.9919 | 0.9975 | 0.9934 | 1.0044 | 0.9970 |
| stringtrim-single-literal-4000 | 0.9902 | 0.9934 | 0.9905 | 1.0055 | 1.0008 | 0.9985 | 0.9902 | 1.0008 | 1.0045 |
| stringtrim-single-literal-32000 | 0.9864 | 0.9987 | 0.9849 | 0.9992 | 0.9843 | 1.0025 | 0.9964 | 0.9956 | 0.9944 |
| stringtrim-whitespace-4000 | 0.9932 | 0.9966 | 0.9907 | 1.0001 | 0.9952 | 1.0083 | 1.0006 | 1.0124 | 1.0013 |
| stringtrim-whitespace-32000 | 0.9880 | 0.9987 | 0.9866 | 0.9976 | 0.9860 | 1.0020 | 0.9990 | 0.9959 | 0.9962 |
| stringreplace-single-literal-unguarded | 0.9900 | 0.9949 | 0.9969 | 1.0079 | 1.0044 | 1.0092 | 0.9989 | 1.0026 | 1.0338 |
| stringreplace-pattern-unguarded | 1.0079 | 0.9992 | 1.0069 | 0.9979 | 1.0020 | 1.0046 | 0.9895 | 0.9911 | 1.0075 |
| stringreplace-guarded-hit | 0.9933 | 0.9898 | 1.0055 | 1.0111 | 1.0144 | 1.0146 | 0.9976 | 0.9971 | 1.0266 |
| stringreplace-guarded-decline-fallback | 0.9941 | 0.9894 | 1.0160 | 1.0149 | 1.0244 | 1.0110 | 0.9883 | 0.9955 | 1.0323 |
| appendto-list-incremental-small | 0.9988 | 0.9995 | 1.0051 | 1.0114 | 1.0058 | 0.9881 | 1.0001 | 0.9912 | 1.0027 |
| appendto-list-incremental-large | 1.0004 | 0.9986 | 1.0021 | 1.0020 | 1.0015 | 0.9991 | 0.9979 | 0.9969 | 1.0043 |
| appendto-part-outer1000-small | 1.0010 | 1.0009 | 1.0015 | 0.9999 | 1.0003 | 0.9996 | 0.9952 | 1.0005 | 1.0004 |
| appendto-part-outer1000-large | 1.0025 | 1.0018 | 1.0002 | 1.0004 | 0.9998 | 0.9995 | 0.9959 | 0.9987 | 1.0008 |
| association-set-key-incremental-small | 0.9930 | 0.9916 | 0.9913 | 0.9969 | 0.9994 | 1.0089 | 1.0015 | 1.0020 | 0.9991 |
| association-set-key-incremental-large | 0.9939 | 0.9915 | 0.9931 | 0.9968 | 0.9975 | 1.0102 | 0.9985 | 1.0013 | 0.9939 |
| association-small-literal-repeat | 1.0095 | 1.0016 | 1.0140 | 1.0084 | 1.0124 | 0.9992 | 1.0047 | 0.9919 | 1.0271 |
| set-part-fixed100-len100 | 0.9940 | 0.9992 | 0.9942 | 0.9992 | 1.0020 | 0.9974 | 1.0153 | 0.9832 | 1.0150 |
| set-part-fixed100-len800 | 1.0009 | 1.0008 | 1.0040 | 1.0026 | 0.9993 | 0.9968 | 1.0391 | 0.9976 | 1.0383 |
| set-part-fill-small | 1.0016 | 1.0004 | 0.9933 | 0.9910 | 0.9890 | 0.9965 | 1.0208 | 0.9867 | 1.0272 |
| set-part-fill-large | 0.9989 | 0.9993 | 0.9986 | 0.9995 | 0.9986 | 0.9964 | 1.0052 | 1.0017 | 1.0095 |
| upvalue-candidates-tiny | 1.0060 | 1.0013 | 1.0039 | 1.0080 | 1.0034 | 0.9984 | 0.9885 | 0.9852 | 1.0366 |
| upvalue-candidates-wide-small | 0.9950 | 1.0014 | 1.0002 | 1.0020 | 0.9997 | 0.9933 | 0.9854 | 0.9909 | 1.0109 |
| upvalue-candidates-wide-large | 1.0001 | 1.0051 | 1.0028 | 1.0001 | 0.9970 | 0.9977 | 0.9857 | 0.9954 | 1.0046 |
| sort-callback-small | 1.0018 | 0.9961 | 0.9906 | 0.9969 | 0.9939 | 1.0286 | 0.9831 | 1.0013 | 1.0225 |
| sort-callback-large | 0.9995 | 0.9952 | 0.9891 | 0.9898 | 0.9915 | 1.0304 | 0.9868 | 1.0019 | 1.0258 |
| sortby-callback-small | 0.9971 | 0.9983 | 0.9843 | 0.9886 | 0.9881 | 1.0108 | 0.9872 | 1.0026 | 1.0183 |
| sortby-callback-large | 0.9966 | 0.9945 | 0.9874 | 0.9921 | 0.9849 | 1.0157 | 0.9899 | 1.0057 | 1.0159 |
| reversesort-callback-small | 0.9951 | 0.9872 | 0.9864 | 1.0050 | 1.0016 | 1.0105 | 1.0056 | 0.9869 | 1.0178 |
| reversesort-callback-large | 0.9892 | 0.9822 | 0.9879 | 1.0047 | 1.0034 | 1.0085 | 1.0047 | 0.9935 | 1.0152 |
| keysort-callback-small | 0.9739 | 0.9702 | 0.9748 | 0.9933 | 1.0062 | 1.0071 | 1.0002 | 1.0013 | 1.0202 |
| keysort-callback-large | 0.9672 | 0.9688 | 0.9737 | 1.0077 | 1.0010 | 1.0043 | 1.0040 | 1.0074 | 1.0192 |
| keysortby-callback-small | 0.9851 | 0.9874 | 0.9900 | 1.0021 | 1.0031 | 1.0014 | 0.9973 | 1.0012 | 1.0171 |
| keysortby-callback-large | 0.9853 | 0.9855 | 0.9828 | 1.0002 | 0.9934 | 1.0092 | 0.9955 | 0.9924 | 1.0246 |
| sort-default-wide | 1.0008 | 0.9983 | 1.0032 | 1.0026 | 1.0042 | 0.9973 | 0.9986 | 0.9979 | 1.0019 |
| sort-symbol-default-width4000 | 1.0255 | 1.0146 | 0.9798 | 0.9644 | 0.9879 | 1.0033 | 1.0022 | 1.0211 | 0.9835 |
| sort-symbol-default-width16000 | 1.0130 | 0.9990 | 1.0057 | 0.9960 | 1.0000 | 0.9979 | 0.9990 | 1.0087 | 0.9829 |
| sortby-default-wide | 1.0018 | 1.0035 | 0.9938 | 0.9846 | 0.9912 | 0.9930 | 1.0084 | 1.0049 | 1.0083 |
| reversesort-default-wide | 1.0000 | 1.0007 | 1.0020 | 1.0014 | 1.0010 | 0.9996 | 1.0004 | 0.9996 | 0.9994 |
| keysort-default-wide | 0.9850 | 0.9746 | 0.9832 | 0.9875 | 1.0083 | 1.0233 | 0.9870 | 1.0186 | 1.0028 |
| keysortby-default-wide | 1.0013 | 0.9919 | 0.9931 | 1.0012 | 0.9963 | 0.9887 | 1.0016 | 1.0092 | 1.0055 |
| valueq-undefined-symbol-definitions-2000 | 0.9948 | 0.9892 | 0.9949 | 1.0010 | 1.0019 | 1.0064 | 0.9983 | 1.0123 | 1.0204 |
| valueq-undefined-symbol-definitions-8000 | 0.9905 | 0.9918 | 0.9972 | 1.0086 | 1.0037 | 1.0066 | 1.0026 | 1.0036 | 1.0141 |
| module-retained-gensym-gc-batch | 1.0352 | 1.0074 | 1.0271 | 1.0083 | 1.0226 | 1.0100 | 1.0369 | 0.9678 | 1.0112 |

## focus

| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| focus-increment-1024 | 0.8620 | 0.9917 | 0.8608 | 1.0031 | 0.8720 | 1.0110 | 1.0126 | 0.9844 | 1.0074 |
| focus-increment-16000 | 0.8556 | 0.9912 | 0.8564 | 0.9975 | 0.8627 | 1.0063 | 1.0148 | 0.9810 | 1.0166 |
| focus-increment-reversed-1024 | 0.8545 | 0.9906 | 0.8525 | 1.0002 | 0.8611 | 1.0012 | 1.0109 | 0.9674 | 1.0128 |
| focus-increment-reversed-16000 | 0.8524 | 0.9903 | 0.8533 | 1.0003 | 0.8624 | 1.0055 | 1.0145 | 0.9841 | 1.0166 |
| focus-identity-1024 | 0.6167 | 0.9908 | 0.6072 | 0.9834 | 0.6082 | 0.9986 | 1.0038 | 1.0045 | 1.0079 |
| focus-identity-16000 | 0.6053 | 0.9974 | 0.5880 | 0.9768 | 0.5885 | 0.9971 | 0.9949 | 0.9983 | 1.0053 |
| focus-miss-1024 | 1.0016 | 0.9705 | 0.9740 | 0.9706 | 0.9977 | 1.0012 | 1.0019 | 1.0025 | 0.9936 |
| focus-miss-16000 | 1.0027 | 0.9727 | 0.9722 | 0.9691 | 0.9976 | 0.9998 | 1.0034 | 0.9935 | 1.0010 |
| focus-opaque-body-1024 | 0.6730 | 0.9785 | 0.6679 | 0.9831 | 0.6807 | 1.0061 | 0.9982 | 0.9986 | 0.9919 |
| focus-opaque-body-16000 | 0.7272 | 0.9957 | 0.7183 | 0.9862 | 0.7214 | 1.0145 | 1.0092 | 0.9946 | 1.0013 |
| focus-flat-nested-1024 | 0.9117 | 1.0021 | 0.9149 | 1.0011 | 0.9176 | 1.0120 | 1.0017 | 0.9893 | 1.0157 |
| focus-flat-nested-16000 | 0.9137 | 1.0049 | 0.9180 | 0.9988 | 0.9160 | 1.0154 | 1.0076 | 0.9928 | 1.0084 |

## pipeline

| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| synthetic-rewrite-heavy-hit-4000 | 0.8663 | 0.9922 | 0.8564 | 0.9905 | 0.8614 | 1.0122 | 0.9992 | 0.9738 | 1.0219 |
| synthetic-rewrite-heavy-hit-16000 | 0.8621 | 0.9942 | 0.8552 | 0.9920 | 0.8568 | 1.0076 | 0.9949 | 0.9761 | 1.0208 |
| synthetic-mixed-hit-4000 | 0.8773 | 0.9933 | 0.8713 | 0.9930 | 0.8750 | 1.0171 | 1.0052 | 0.9804 | 1.0234 |
| synthetic-mixed-hit-16000 | 0.8725 | 0.9937 | 0.8665 | 0.9940 | 0.8699 | 1.0106 | 0.9963 | 0.9787 | 1.0207 |
| synthetic-mixed-control-4000 | 1.0007 | 1.0107 | 0.9974 | 1.0006 | 0.9925 | 0.9951 | 0.9870 | 0.9992 | 1.0145 |
| synthetic-mixed-control-16000 | 1.0083 | 1.0071 | 1.0068 | 0.9961 | 0.9957 | 0.9993 | 1.0016 | 1.0073 | 1.0055 |

## shape

| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| shape-chain-16-positive | 0.8995 | 0.9698 | 0.9596 | 1.2419 | 1.0069 | 0.9964 | 1.3441 | 0.9931 | 0.9890 |
| shape-chain-16-negative | 0.9840 | 0.9871 | 0.9876 | 1.0141 | 0.9966 | 0.9914 | 1.0016 | 0.9939 | 0.9863 |
| shape-chain-32-positive | 0.9850 | 0.9095 | 0.9047 | 0.9173 | 1.0008 | 0.9950 | 1.0012 | 0.9919 | 0.9813 |
| shape-chain-32-negative | 0.9979 | 0.9963 | 1.0002 | 1.0074 | 1.0044 | 0.9981 | 1.0003 | 0.9957 | 0.9943 |
| shape-chain-64-positive | 0.9936 | 0.8245 | 0.8288 | 0.8379 | 1.0057 | 0.9905 | 0.9962 | 0.9862 | 0.9872 |
| shape-chain-64-negative | 0.9975 | 0.9958 | 0.9932 | 1.0011 | 1.0036 | 0.9947 | 1.0011 | 1.0019 | 0.9892 |
| shape-binary-5-positive | 0.9939 | 0.9489 | 0.9626 | 0.9702 | 1.0004 | 1.0041 | 0.9945 | 0.9967 | 0.9843 |
| shape-binary-5-negative | 0.9931 | 0.9939 | 1.0052 | 1.0100 | 1.0078 | 1.0059 | 0.9986 | 0.9944 | 0.9908 |
| shape-binary-7-positive | 0.9824 | 0.9232 | 0.9105 | 0.9239 | 1.0009 | 1.0178 | 1.0023 | 0.9839 | 0.9708 |
| shape-binary-7-negative | 0.9903 | 0.9877 | 0.9708 | 0.9835 | 0.9907 | 1.0348 | 1.0062 | 0.9671 | 0.9729 |
| shape-binary-9-positive | 0.9832 | 0.9099 | 0.8995 | 0.9177 | 0.9979 | 0.9993 | 0.9960 | 0.9800 | 0.9643 |
| shape-binary-9-negative | 0.9976 | 0.9971 | 0.9900 | 0.9933 | 0.9974 | 1.0109 | 0.9832 | 0.9879 | 1.0032 |

All slower original-suite medians (including very small differences):

{
  "matcher/base": [
    "association-part-positional-read-width32000",
    "do-count-null",
    "table-iterator-null",
    "table-iterator-symbol",
    "symbol-ownvalue-read",
    "symbol-ownvalue-guarded-read",
    "module-local",
    "module-chain-escape-small",
    "module-chain-escape-large",
    "table-small",
    "table-iterator-plus",
    "table-iterator-pair-list",
    "downvalue-exact-hit",
    "downvalue-exact-miss",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "cases-wide-no-match",
    "position-wide-no-match",
    "replacepart-wide-rule-list",
    "replacepart-guarded-hit",
    "replacepart-guarded-decline-fallback",
    "stringreplace-pattern-unguarded",
    "appendto-list-incremental-large",
    "appendto-part-outer1000-small",
    "appendto-part-outer1000-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len800",
    "set-part-fill-small",
    "upvalue-candidates-tiny",
    "upvalue-candidates-wide-large",
    "sort-callback-small",
    "sort-default-wide",
    "sort-symbol-default-width4000",
    "sort-symbol-default-width16000",
    "sortby-default-wide",
    "keysortby-default-wide",
    "module-retained-gensym-gc-batch"
  ],
  "depth/base": [
    "association-part-key-read-width4000",
    "loop-empty",
    "table-iterator-null",
    "literal-integer",
    "symbol-ownvalue-read",
    "module-chain-escape-large",
    "table-small",
    "table-iterator-plus",
    "table-iterator-pair-list",
    "do-small",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "cases-wide-no-match",
    "deletecases-wide-no-match",
    "deletecases-wide-association-no-match",
    "replacepart-guarded-decline-fallback",
    "stringtrim-literal-list-32000",
    "appendto-part-outer1000-small",
    "appendto-part-outer1000-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len800",
    "set-part-fill-small",
    "upvalue-candidates-tiny",
    "upvalue-candidates-wide-small",
    "upvalue-candidates-wide-large",
    "sort-symbol-default-width4000",
    "sortby-default-wide",
    "reversesort-default-wide",
    "module-retained-gensym-gc-batch"
  ],
  "combined/base": [
    "association-part-key-read-width4000",
    "symbol-ownvalue-guarded-read",
    "module-local",
    "module-chain-escape-small",
    "module-chain-escape-large",
    "block-init-null-repeat",
    "with-local",
    "table-small",
    "table-iterator-plus",
    "do-small",
    "downvalue-exact-hit",
    "downvalue-pattern-hit",
    "downvalue-guarded-hit",
    "upvalue-guarded-hit",
    "matchq-integer-success",
    "matchq-integer-fail",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "rewrite-guarded-hit",
    "cases-wide-no-match",
    "deletecases-wide-no-match",
    "deletecases-wide-association-no-match",
    "replacepart-guarded-hit",
    "replacepart-guarded-decline-fallback",
    "character-range-small",
    "stringreplace-pattern-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "appendto-list-incremental-small",
    "appendto-list-incremental-large",
    "appendto-part-outer1000-small",
    "appendto-part-outer1000-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len800",
    "upvalue-candidates-tiny",
    "upvalue-candidates-wide-small",
    "upvalue-candidates-wide-large",
    "sort-default-wide",
    "sort-symbol-default-width16000",
    "reversesort-default-wide",
    "module-retained-gensym-gc-batch"
  ],
  "combined/matcher": [
    "association-part-key-read-width4000",
    "association-part-key-read-width32000",
    "association-part-positional-read-width32000",
    "do-iterator-null",
    "table-count-null",
    "literal-string",
    "head-list",
    "length-list",
    "part-list",
    "sameq-list",
    "if-true",
    "module-local",
    "module-chain-escape-small",
    "block-local",
    "block-symbol-only-repeat",
    "block-init-null-repeat",
    "block-init-read-repeat",
    "with-local",
    "table-small",
    "do-small",
    "downvalue-exact-hit",
    "downvalue-pattern-hit",
    "downvalue-exact-miss-repeat",
    "downvalue-pattern-miss-repeat",
    "downvalue-guarded-hit",
    "downvalue-guarded-decline-fallback",
    "upvalue-guarded-hit",
    "subvalue-unguarded-hit",
    "subvalue-guarded-hit",
    "default-unguarded-hit",
    "default-unguarded-miss",
    "default-guarded-hit",
    "matchq-integer-success",
    "matchq-integer-fail",
    "matcher-single-wide-small",
    "matcher-single-wide-large",
    "rewrite-guarded-hit",
    "rewrite-guarded-decline-fallback",
    "cases-wide-no-match",
    "count-wide-no-match",
    "deletecases-wide-no-match",
    "deletecases-wide-association-no-match",
    "replacepart-guarded-hit",
    "string-repeat-small",
    "character-range-small",
    "stringtrim-single-literal-4000",
    "stringtrim-whitespace-4000",
    "stringreplace-single-literal-unguarded",
    "stringreplace-guarded-hit",
    "stringreplace-guarded-decline-fallback",
    "appendto-list-incremental-small",
    "appendto-list-incremental-large",
    "appendto-part-outer1000-large",
    "association-small-literal-repeat",
    "set-part-fixed100-len800",
    "upvalue-candidates-tiny",
    "upvalue-candidates-wide-small",
    "upvalue-candidates-wide-large",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysort-callback-large",
    "keysortby-callback-small",
    "keysortby-callback-large",
    "sort-default-wide",
    "reversesort-default-wide",
    "keysortby-default-wide",
    "valueq-undefined-symbol-definitions-2000",
    "valueq-undefined-symbol-definitions-8000",
    "module-retained-gensym-gc-batch"
  ],
  "combined/depth": [
    "association-part-key-read-width4000",
    "association-part-positional-read-width4000",
    "association-part-positional-read-width32000",
    "loop-empty",
    "do-count-null",
    "table-count-null",
    "table-iterator-symbol",
    "symbol-ownvalue-guarded-read",
    "sameq-list",
    "if-true",
    "module-local",
    "module-chain-escape-small",
    "block-local",
    "block-symbol-only-repeat",
    "block-init-null-repeat",
    "with-local",
    "table-iterator-plus",
    "do-small",
    "downvalue-exact-hit",
    "downvalue-exact-miss",
    "downvalue-pattern-hit",
    "downvalue-pattern-miss",
    "downvalue-exact-miss-repeat",
    "downvalue-pattern-miss-repeat",
    "downvalue-guarded-hit",
    "upvalue-guarded-hit",
    "subvalue-unguarded-hit",
    "subvalue-unguarded-miss",
    "subvalue-guarded-hit",
    "default-guarded-hit",
    "matchq-integer-success",
    "matchq-integer-fail",
    "matcher-single-wide-large",
    "rewrite-guarded-hit",
    "cases-wide-no-match",
    "position-wide-no-match",
    "replacepart-wide-rule-list",
    "replacepart-guarded-hit",
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
    "association-small-literal-repeat",
    "set-part-fixed100-len100",
    "upvalue-candidates-tiny",
    "reversesort-callback-small",
    "reversesort-callback-large",
    "keysort-callback-small",
    "keysort-callback-large",
    "keysortby-callback-small",
    "sort-default-wide",
    "reversesort-default-wide",
    "keysort-default-wide",
    "valueq-undefined-symbol-definitions-2000",
    "valueq-undefined-symbol-definitions-8000",
    "module-retained-gensym-gc-batch"
  ]
}

One nominally idle desktop, same WSL host and toolchain; external background work is not controlled completely.
Eight paired process blocks; retained in-process repetitions are not independent experiment blocks.
Every slower original-suite median is retained; A/A ratios are controls and are never subtracted.
Same-binary aliases check timing noise, not independent compilation reproducibility.
Synthetic focus/pipeline programs do not establish a real application distribution or whole-runtime speedup.
Value, alias, continuation, trace and selected resource-limit checks are finite evidence; this is not full contextual equivalence.
