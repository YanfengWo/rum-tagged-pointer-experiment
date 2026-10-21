# Initial baseline expectation correction

This untimed attempt failed one newly authored regression on the unchanged
baseline, before any shortcut patch was applied. The fixture does not install
the standard library and an unqualified Association resolves to an ordinary
Global call; its integer keys are therefore ordinary traversed arguments.
The corrected shared test source distinguishes this case from an explicitly
qualified System Association. The original failing source, hashes and logs
are retained here. This is not a candidate runtime regression or timing run.
