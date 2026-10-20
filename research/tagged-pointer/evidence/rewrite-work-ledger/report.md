# Exact synchronous rewrite work counts

No performance timing comparison was run. Capacity is in Expr slots, not allocator bytes.

| Mode | Case | N | depth helper entries | matcher entries | RHS instantiations | Plus calls | wide copies | two-slot copies | fresh Flat Vec outputs | reused Flat Vec outputs |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| base | increment | 8 | 41 | 16 | 8 | 8 | 1 | 7 | 7 | 0 |
| base | increment | 64 | 321 | 128 | 64 | 64 | 1 | 63 | 63 | 0 |
| base | increment | 256 | 1281 | 512 | 256 | 256 | 1 | 255 | 255 | 0 |
| base | increment | 1024 | 5121 | 2048 | 1024 | 1024 | 1 | 1023 | 1023 | 0 |
| base | increment-reversed | 8 | 41 | 16 | 8 | 8 | 1 | 0 | 0 | 0 |
| base | increment-reversed | 64 | 321 | 128 | 64 | 64 | 1 | 0 | 0 | 0 |
| base | increment-reversed | 256 | 1281 | 512 | 256 | 256 | 1 | 0 | 0 | 0 |
| base | increment-reversed | 1024 | 5121 | 2048 | 1024 | 1024 | 1 | 0 | 0 | 0 |
| base | identity | 8 | 25 | 16 | 8 | 0 | 1 | 0 | 0 | 0 |
| base | identity | 64 | 193 | 128 | 64 | 0 | 1 | 0 | 0 | 0 |
| base | identity | 256 | 769 | 512 | 256 | 0 | 1 | 0 | 0 | 0 |
| base | identity | 1024 | 3073 | 2048 | 1024 | 0 | 1 | 0 | 0 | 0 |
| base | miss | 8 | 17 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| base | miss | 64 | 129 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| base | miss | 256 | 513 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| base | miss | 1024 | 2049 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| base | opaque-body | 8 | 33 | 16 | 8 | 0 | 1 | 0 | 0 | 0 |
| base | opaque-body | 64 | 257 | 128 | 64 | 0 | 1 | 0 | 0 | 0 |
| base | opaque-body | 256 | 1025 | 512 | 256 | 0 | 1 | 0 | 0 | 0 |
| base | opaque-body | 1024 | 4097 | 2048 | 1024 | 0 | 1 | 0 | 0 | 0 |
| base | flat-nested | 8 | 49 | 16 | 8 | 0 | 1 | 8 | 8 | 0 |
| base | flat-nested | 64 | 385 | 128 | 64 | 0 | 1 | 64 | 64 | 0 |
| base | flat-nested | 256 | 1537 | 512 | 256 | 0 | 1 | 256 | 256 | 0 |
| base | flat-nested | 1024 | 6145 | 2048 | 1024 | 0 | 1 | 1024 | 1024 | 0 |
| wide | increment | 8 | 41 | 16 | 8 | 8 | 0 | 7 | 7 | 0 |
| wide | increment | 64 | 321 | 128 | 64 | 64 | 0 | 63 | 63 | 0 |
| wide | increment | 256 | 1281 | 512 | 256 | 256 | 0 | 255 | 255 | 0 |
| wide | increment | 1024 | 5121 | 2048 | 1024 | 1024 | 0 | 1023 | 1023 | 0 |
| wide | increment-reversed | 8 | 41 | 16 | 8 | 8 | 0 | 0 | 0 | 0 |
| wide | increment-reversed | 64 | 321 | 128 | 64 | 64 | 0 | 0 | 0 | 0 |
| wide | increment-reversed | 256 | 1281 | 512 | 256 | 256 | 0 | 0 | 0 | 0 |
| wide | increment-reversed | 1024 | 5121 | 2048 | 1024 | 1024 | 0 | 0 | 0 | 0 |
| wide | identity | 8 | 25 | 16 | 8 | 0 | 0 | 0 | 0 | 0 |
| wide | identity | 64 | 193 | 128 | 64 | 0 | 0 | 0 | 0 | 0 |
| wide | identity | 256 | 769 | 512 | 256 | 0 | 0 | 0 | 0 | 0 |
| wide | identity | 1024 | 3073 | 2048 | 1024 | 0 | 0 | 0 | 0 | 0 |
| wide | miss | 8 | 17 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide | miss | 64 | 129 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide | miss | 256 | 513 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide | miss | 1024 | 2049 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide | opaque-body | 8 | 33 | 16 | 8 | 0 | 0 | 0 | 0 | 0 |
| wide | opaque-body | 64 | 257 | 128 | 64 | 0 | 0 | 0 | 0 | 0 |
| wide | opaque-body | 256 | 1025 | 512 | 256 | 0 | 0 | 0 | 0 | 0 |
| wide | opaque-body | 1024 | 4097 | 2048 | 1024 | 0 | 0 | 0 | 0 | 0 |
| wide | flat-nested | 8 | 49 | 16 | 8 | 0 | 0 | 8 | 8 | 0 |
| wide | flat-nested | 64 | 385 | 128 | 64 | 0 | 0 | 64 | 64 | 0 |
| wide | flat-nested | 256 | 1537 | 512 | 256 | 0 | 0 | 256 | 256 | 0 |
| wide | flat-nested | 1024 | 6145 | 2048 | 1024 | 0 | 0 | 1024 | 1024 | 0 |
| wide-flat | increment | 8 | 41 | 16 | 8 | 8 | 0 | 7 | 0 | 7 |
| wide-flat | increment | 64 | 321 | 128 | 64 | 64 | 0 | 63 | 0 | 63 |
| wide-flat | increment | 256 | 1281 | 512 | 256 | 256 | 0 | 255 | 0 | 255 |
| wide-flat | increment | 1024 | 5121 | 2048 | 1024 | 1024 | 0 | 1023 | 0 | 1023 |
| wide-flat | increment-reversed | 8 | 41 | 16 | 8 | 8 | 0 | 0 | 0 | 0 |
| wide-flat | increment-reversed | 64 | 321 | 128 | 64 | 64 | 0 | 0 | 0 | 0 |
| wide-flat | increment-reversed | 256 | 1281 | 512 | 256 | 256 | 0 | 0 | 0 | 0 |
| wide-flat | increment-reversed | 1024 | 5121 | 2048 | 1024 | 1024 | 0 | 0 | 0 | 0 |
| wide-flat | identity | 8 | 25 | 16 | 8 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | identity | 64 | 193 | 128 | 64 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | identity | 256 | 769 | 512 | 256 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | identity | 1024 | 3073 | 2048 | 1024 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | miss | 8 | 17 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | miss | 64 | 129 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | miss | 256 | 513 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | miss | 1024 | 2049 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | opaque-body | 8 | 33 | 16 | 8 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | opaque-body | 64 | 257 | 128 | 64 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | opaque-body | 256 | 1025 | 512 | 256 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | opaque-body | 1024 | 4097 | 2048 | 1024 | 0 | 0 | 0 | 0 | 0 |
| wide-flat | flat-nested | 8 | 49 | 16 | 8 | 0 | 0 | 8 | 8 | 0 |
| wide-flat | flat-nested | 64 | 385 | 128 | 64 | 0 | 0 | 64 | 64 | 0 |
| wide-flat | flat-nested | 256 | 1537 | 512 | 256 | 0 | 0 | 256 | 256 | 0 |
| wide-flat | flat-nested | 1024 | 6145 | 2048 | 1024 | 0 | 0 | 1024 | 1024 | 0 |
| flat | increment | 8 | 41 | 16 | 8 | 8 | 1 | 7 | 0 | 7 |
| flat | increment | 64 | 321 | 128 | 64 | 64 | 1 | 63 | 0 | 63 |
| flat | increment | 256 | 1281 | 512 | 256 | 256 | 1 | 255 | 0 | 255 |
| flat | increment | 1024 | 5121 | 2048 | 1024 | 1024 | 1 | 1023 | 0 | 1023 |
| flat | increment-reversed | 8 | 41 | 16 | 8 | 8 | 1 | 0 | 0 | 0 |
| flat | increment-reversed | 64 | 321 | 128 | 64 | 64 | 1 | 0 | 0 | 0 |
| flat | increment-reversed | 256 | 1281 | 512 | 256 | 256 | 1 | 0 | 0 | 0 |
| flat | increment-reversed | 1024 | 5121 | 2048 | 1024 | 1024 | 1 | 0 | 0 | 0 |
| flat | identity | 8 | 25 | 16 | 8 | 0 | 1 | 0 | 0 | 0 |
| flat | identity | 64 | 193 | 128 | 64 | 0 | 1 | 0 | 0 | 0 |
| flat | identity | 256 | 769 | 512 | 256 | 0 | 1 | 0 | 0 | 0 |
| flat | identity | 1024 | 3073 | 2048 | 1024 | 0 | 1 | 0 | 0 | 0 |
| flat | miss | 8 | 17 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| flat | miss | 64 | 129 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| flat | miss | 256 | 513 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| flat | miss | 1024 | 2049 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
| flat | opaque-body | 8 | 33 | 16 | 8 | 0 | 1 | 0 | 0 | 0 |
| flat | opaque-body | 64 | 257 | 128 | 64 | 0 | 1 | 0 | 0 | 0 |
| flat | opaque-body | 256 | 1025 | 512 | 256 | 0 | 1 | 0 | 0 | 0 |
| flat | opaque-body | 1024 | 4097 | 2048 | 1024 | 0 | 1 | 0 | 0 | 0 |
| flat | flat-nested | 8 | 49 | 16 | 8 | 0 | 1 | 8 | 8 | 0 |
| flat | flat-nested | 64 | 385 | 128 | 64 | 0 | 1 | 64 | 64 | 0 |
| flat | flat-nested | 256 | 1537 | 512 | 256 | 0 | 1 | 256 | 256 | 0 |
| flat | flat-nested | 1024 | 6145 | 2048 | 1024 | 0 | 1 | 1024 | 1024 | 0 |

Resource observations (not required to be numerically equal):

| Measurement | Base | Flat candidate |
| --- | ---: | ---: |
| memory-before | 68411 | 68411 |
| bytecount-f2 | 216 | 152 |
| memory-after-f2 | 68963 | 68899 |
| bytecount-f3 | 224 | 192 |
| memory-after-f3 | 69361 | 69265 |
| bytecount-f5 | 368 | 272 |
| memory-after-f5 | 69903 | 69711 |

Construction-history ByteCount observations (SameQ passes):

original: literal=211, Apply=147
flat: literal=147, Apply=147

Logical counts and capacity slots are not elapsed-time or total allocator-byte measurements.
Generated synchronous Replace workloads; lexical scopes do not cover later suspension resumption.
Flat early return preserves expression elements/order/identities but resource observations and tight modeled-memory thresholds may differ.
Candidate full microbenchmark gate deferred while the user uses the desktop; no acceleration or no-regression claim.
Prior timing stages recorded A/A controls but no external-application load trace; those controls do not exclude every external source of variation.
