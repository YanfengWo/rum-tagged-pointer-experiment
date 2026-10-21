# Initial preparation attempt

The patch application was refused before compilation because its Windows line
endings did not match the Linux source archive. The patch was normalized to LF
and git apply --check passed before a new isolated build. The original inputs
and preparation failure are retained here; no timing or candidate runtime
execution occurred in this attempt.
