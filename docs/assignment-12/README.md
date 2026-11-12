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
