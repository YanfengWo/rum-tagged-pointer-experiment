# Untimed shape-preflight correction

All result and input checks passed, but the generic alias test tried Part
assignment on an ordinary function call. Rum permits that assignment only on
plain Lists. Stderr rejected this preflight before any timing plan or samples
were created. The corrected shape wrapper boxes the result in a List before
its post-window alias mutation; timed Replace bodies are unchanged. The
original generator and all preflight outputs are retained here.
