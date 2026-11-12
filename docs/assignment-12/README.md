# Assignment 12 automated testing

Branch: `assignment-12-tdd`, based on `151102da35` (`main`).
The assignment calls its deliverables A7, so the commits and wiki page use that prefix.

## Baseline

The first full Python SDK run used the existing repository `.venv` (Python 3.11.15) with coverage.py 7.17.2 added.
No production code was changed for characterization.

```sh
.venv/bin/python -m coverage run --branch --source=rerun_py/rerun_sdk/rerun --data-file=/tmp/a12-before.coverage -m pytest rerun_py/tests
.venv/bin/python -m coverage report --data-file=/tmp/a12-before.coverage
.venv/bin/python -m coverage json --data-file=/tmp/a12-before.coverage -o docs/assignment-12/before-coverage.json
```

The initial full run recorded 1,207 passed, 152 failed, 194 errors, and 14 skipped.
The sandbox blocked local sockets, and many Git LFS fixtures were pointer files rather than data.
See `before-tests.txt` for the complete output rather than treating this as a passing baseline.

SDK-wide Python line coverage was 71.7895% (15,200 / 21,173 statements), branch coverage was 44.9945% (1,645 / 3,656 branches), and coverage.py's combined figure was 67.8441%.
This is not overall Rust/C++/Python project coverage, and failed setup paths make it incomplete even for Python.
Coverage measures execution, not assertion quality.

## Characterization targets

- `LoggingHandler`: real `LogRecord` formatting, module-to-entity-path conversion, level mapping, empty messages, custom levels, and formatting failure without forwarding.
- `script_setup`: real argument parsing, each sink route, headless operation, default spawning, flag precedence, and propagated connection failure.
- `default_strict_mode`: all eight documented environment values, case handling, absent values, and invalid-value diagnostics with preservation of the current mode.

These had no dedicated behavioral tests in `rerun_py/tests` before this assignment.
They control log ingestion, recording destinations, and SDK-wide error handling.
Additional tests cover `build_column_args`; that function already had substantial indirect coverage, so it is not counted as one of the three previously untested targets.
The characterization command is:

```sh
.venv/bin/python -m pytest rerun_py/tests/unit/test_logging_boundaries.py rerun_py/tests/unit/test_strict_mode_environment.py -q
```

The initial characterization run passed all 31 tests.
The tests use real Arrow arrays and Rerun data objects, while replacing external recording operations at the side-effect boundary.
The invalid-environment characterization tests run after import and preserve existing behavior; the later regression tests exercise fresh interpreter startup, where the real defect occurs.

## Final results and review

The primary comparison is `matched-before-coverage.*` against `matched-after-coverage.*`.
Both use the restored fixtures, the repository Viewer, and the full Python SDK suite with local servers permitted.
The matched baseline used the exact `main` version of `error_utils.py` with the two new test files excluded.
The final run used the refactored fix and all tests.
The original source was restored automatically after measuring the baseline.

| Metric | Matched baseline | Final |
| --- | ---: | ---: |
| Python SDK line coverage | 74.5431% | 74.7379% |
| Python SDK branch coverage | 50.2462% | 50.8206% |
| coverage.py combined coverage | 70.9654% | 71.2163% |
| Full Python SDK suite passed | 1,549 | 1,583 |
| Full Python SDK suite failed | 2 | 2 |
| Full Python SDK suite skipped | 16 | 16 |

Both failures are the pre-existing headless Viewer lifecycle cases `test_viewer_dies_on_client_close` and `test_viewer_can_restart_on_same_port`.
The Viewer panics loading `Global.Typography.Body`; process teardown also reports permission errors.
These occur in the matched baseline before any assignment changes.
The full unit suite is green with 1,031 passed and 4 skipped; the full broader SDK suite is not green.
A matching, working Viewer and process-control permissions are needed to finish verifying those lifecycle integrations.
Do not claim that the assignment repaired them or that CI passed.

Final coverage command (use `before` instead of `after` for the original parser and exclude both new test files):

```sh
PATH="$PWD/.venv/bin:$PATH" .venv/bin/python -m coverage run --branch --source=rerun_py/rerun_sdk/rerun --data-file=/tmp/a12-matched-after.coverage -m pytest rerun_py/tests
.venv/bin/python -m coverage report --data-file=/tmp/a12-matched-after.coverage
.venv/bin/python -m coverage json --data-file=/tmp/a12-matched-after.coverage -o docs/assignment-12/matched-after-coverage.json
```

The initial `before-*` reports preserve the inherited sandbox baseline.
The plain `after-*` reports record a sandbox run while fixtures were being restored and are diagnostic rather than the primary comparison.
`verified-full-tests.txt` records an additional run with one Viewer failure; the stable matched comparison reports both lifecycle failures on both sides.
This variation is another reason not to infer correctness from a coverage percentage.
Coverage excludes native Rust/C++ code and subprocess execution.

The tests have not changed since the Red commit.
Green initializes the fallback before environment parsing; Refactor flattens the parser into early returns and documents that initialization order.
Ruff lint and format checks pass for all three changed Python files, `pixi run rs-fmt` succeeds, and `git diff --check` passes.
No Rust source changes were needed.

Review `A7-Automated-Testing.md` for the wiki text, `AI-Use-Log.md` for the AI disclosure, and `Demo.md` for the two-minute walkthrough.
The wiki's AI assessment and component-risk judgment need the student's review.
The branch and wiki are prepared locally for review; publishing, the in-class demo, and OAKS submission remain.

## Part 2 revision after codebase-wide review

The authoritative Part 2 selection is now documented in [Part2-Codebase-Audit.md](Part2-Codebase-Audit.md).
The audit spans native Rust storage, SDK, Viewer, ingestion, utilities and code generation, the C++ SDK, Python/notebook packages, and JavaScript/TypeScript/browser code.
The three primary targets are `LogSender::flush_blocking`, `LogReceiver::recv_timeout`, and `LogReceiverSet::remove_by_uri`.
Twelve new same-file Rust characterization cases cover normal, boundary, and failure behavior.
The complete `re_log_channel` suite passes **16 tests**, compared with **4** before the additions.
The Python targets described earlier in this README are retained as additional historical work rather than the final three Part 2 selections.

The original four A7 commits remain intact; a supplemental A7-Tests commit adds the Rust characterization and audit.
The Python Red/Green/Refactor evidence is unchanged.
The earlier coverage percentages remain explicitly Python-only; no Rust or whole-project percentage is inferred from the additional test count.
The full multi-language workspace was not rerun.
