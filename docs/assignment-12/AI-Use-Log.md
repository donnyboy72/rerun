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

## Follow-up codebase-wide Part 2 review

The student pointed out that Python-only selection did not consider the whole project and supplied maintainer guidance about same-file Rust tests or crate integration tests.
Codex inspected native storage/indexing, SDK/ingestion, Viewer/blueprint, utility/build code, C++ Catch2 tests, Python/notebook tests, and JavaScript/browser integration locations.
It rejected manifest merging and in-memory provider loading as untested targets after finding existing integration exercise.
It replaced the primary Part 2 selection with three Rust log-channel functions and added twelve tests inside their production files under test-only modules.
The original Python tests and defect fix remain additional work.

Verification: the affected crate's baseline had four passing tests; the final complete nextest run has sixteen passing tests with all features and no skips.
Clippy with all targets and features, the repository Rust formatter, and diff whitespace checks pass.
Only test modules were added to production source files; no runtime behavior or dependencies changed.
The audit is a repository-wide static search and focused candidate review, not proof of coverage for every function.
No new whole-project coverage percentage or passing entire-workspace result is claimed.

## November 12 native Part 3 cycle

The student asked for Part 3 to match the Part 2 native methods and follow the actual Red/Green/Refactor sequence.
Codex found that `LogSender::flush_blocking` waits for acknowledgment before waking a receiver that may depend on the wake callback.
A targeted upstream open-issue search found no matching flush/waker report; the assignment's characterization-discovered-defect option was used.
Issue #9211 was kept separate as earlier research rather than claimed as fixed.

The final Red regression failed with `Err(Timeout), receiver woke: false` and was committed in `e91844fba1`.
A harness lint warning prompted use of a bounded notification channel; the Red failure was rerun and the local Red commit finalized before Green.
Green `ea5ae0246b` only moves the existing wake block before the wait.
Refactor consolidates repeated wake blocks into one private helper.
Both complete affected-crate runs pass 17 tests with no skips.
The sender test module is byte-for-byte identical to final Red; Clippy with warnings denied, repository formatting, and diff checks pass.
No entire-workspace result or new native coverage percentage is claimed.
The code and documentation are published only in the student's fork; no upstream issue, comment, or PR was created.
