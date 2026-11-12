# Two minute demo

Use GitHub's commit views and the wiki once the reviewed branch and page are published.
The commit hashes are listed in `Submission.md`.

1. **0:00–0:35:** Open the A7-Red commit and `docs/assignment-12/red-tests.txt`.
   Show the failing `NameError` and explain why a fresh interpreter matters: calling the parser after import already has a fallback value.
2. **0:35–1:00:** Open A7-Green.
   Show the single `_strict_mode = False` initialization and `green-unit-tests.txt`: 1,031 passed, 4 skipped.
   Explain that invalid input now emits its existing diagnostic and defaults to false during startup.
3. **1:00–1:25:** Show `test_script_setup_when_connection_fails_propagates_error_without_spawning` in `test_logging_boundaries.py`.
   Explain that real argument parsing is retained while recording operations are replaced to avoid network, files, or GUI effects.
4. **1:25–1:45:** Show A7-Refactor and explain the early returns.
   The tests have not changed since Red.
5. **1:45–2:00:** Show the wiki's Testing Evolution and AI sections.
   Explain the assistant's mistaken assumption that no directly named test meant no existing coverage, and the distinction between line execution and assertions.

For a local replay of Red without changing this checkout, inspect the committed log with:

```sh
git show cb7c42c742:docs/assignment-12/red-tests.txt
git show 94d2888f6e -- rerun_py/rerun_sdk/rerun/error_utils.py
```
