# AI Use Log

Date: November 11, 2026 (America/New_York).
Assistant: OpenAI Codex.

The student asked Codex to read Assignment 12, explain whether a PR was required, create a branch, and implement the assignment for review.
Codex read the assignment and repository instructions, created `assignment-12-tdd` from `main`, inspected existing tests, installed coverage.py in the existing repository Python environment, and ran the Python SDK suites.
It authored characterization and regression tests, reproduced an invalid-environment startup crash, saved the genuinely failing output, and made separate A7 Tests, Red, Green, and Refactor commits.
It also drafted the wiki content and demo notes.
No upstream issue or PR was requested or opened.

## Where the assistant helped

It separated imported-state tests from fresh-process startup tests.
An invalid `RERUN_STRICT` value after a successful import returns the current mode, but the same value during import referenced `_strict_mode` before assignment.
The regression tests capture the child interpreter's exit code, diagnostic, and default mode rather than simply checking that a function ran.
The assistant also traced unrelated suite failures to sandbox socket restrictions, Git LFS pointer files, and the system Viewer executable.

## Where the assistant was wrong or needed correction

The assistant initially considered `build_column_args` a previously untested target because no test named it directly.
The baseline report showed substantial indirect coverage of `_send_columns.py`, so it retained those useful additional assertions but did not count that function among the three previously untested targets.
This demonstrates why searching test names alone does not establish a coverage gap.

Its first test invocation also used the restricted sandbox and inherited the system `rerun` on PATH.
That produced many failures unrelated to the fix.
It subsequently restored required fixtures, allowed local test servers, and selected the repository Viewer for a separate validation run.
The raw failing runs remain available so these limitations are visible.

## Student review still needed

The student should read the tests and smallest fix, run the demonstration, and revise the wiki's personal assessment to reflect their own judgment.
The assistant's interpretation is not evidence that the student has independently reviewed the code.
The student must perform the in-class demo and submit links on OAKS.
