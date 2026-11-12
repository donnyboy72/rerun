# A7: Automated Testing

Testing Rerun's Python SDK showed a useful separation between Arrow data conversion and operations that start viewers, choose recordings, or read process-wide configuration.
The latter boundaries needed explicit isolation even when their functions were short.
This assignment adds 31 characterization cases and three fresh-process regression cases, without adding features.

## What testing revealed

Three previously untested behavioral targets were characterized: `LoggingHandler`, `script_setup`, and `default_strict_mode`.
The handler tests assert formatted message text, entity path, severity, empty/custom-level behavior, and formatting failures.
Startup tests cover all sink routes, headless/default operation, conflicting flags, and a connection error that must propagate without launching a fallback viewer.
Configuration tests cover valid values, absent settings, and invalid values that retain an already initialized mode.
Five additional column-validation cases assert Arrow values and mismatch diagnostics; those are not counted as a previously untested component because existing tests already exercised that code indirectly.

`LoggingHandler.emit` imports its logging destination internally, coupling conversion to the SDK's recording machinery.
This weakens **Low Coupling** and **Protected Variations**: tests patch the destination, while retaining real `LogRecord` and `TextLog` objects.
An injected log callable would make that boundary explicit.
`script_setup` initializes global recording state and can touch the network, filesystem, and GUI.
This also weakens **Low Coupling**; an injected recording factory or a separate sink-selection policy would separate decisions from effects.
Tests currently replace those operations and check the selected calls and arguments.
`default_strict_mode` reads the environment and module state during import, coupling configuration policy to initialization order.
An **Information Expert** with a pure value parser and an explicit fallback would keep that responsibility together and protect callers from process-state variation.
No production seams were added for characterization.

## Defect and Red Green Refactor

Reading the configuration code exposed an initialization-order defect.
With `RERUN_STRICT=invalid`, importing `rerun` printed a diagnostic and then raised `NameError`: the fallback `_strict_mode` had not been assigned yet.
An in-process test would miss this because importing the test module already initializes the SDK.
Three subprocess tests reproduced the crash for an invalid word, an empty value, and a whitespace-padded value.
The Red commit contains their actual failure output.
Green adds one initialization, `_strict_mode = False`, before environment parsing.
This preserves existing diagnostics and the behavior of subsequent invalid-value checks.
The full Python unit suite then passed: **1,031 passed, 4 skipped**.
Refactor replaces nested conditionals with early returns and explains why the fallback must exist before parsing; tests remain unchanged.
This is worth reporting upstream because a configuration typo prevents the entire SDK from importing, and the fix is small and independently reproducible.
No issue or PR has been opened.

## Testing Evolution

See the matched before/after coverage reports and complete test output in `docs/assignment-12`.
Coverage is measured across Python SDK source with both line and branch instrumentation; it does not include Rust, C++, native bindings, or child-interpreter execution.
The initial sandbox baseline measured **71.79% line coverage and 44.99% branch coverage** but had many environment failures.
Restoring Git LFS fixtures and choosing the repository Viewer substantially improved execution, so that recovery must not be credited to the new assertions.
The matched comparison below uses the same repaired environment on both sides.

| Python SDK metric | Before | After |
| --- | ---: | ---: |
| Line coverage | 74.54% | 74.74% |
| Branch coverage | 50.25% | 50.82% |
| coverage.py combined | 70.97% | 71.22% |

The matched baseline had **1,549 passed, 2 failed, 16 skipped**; the final run had **1,583 passed, the same 2 failed, 16 skipped**.
Both failures are in headless Viewer lifecycle tests: the executable panics loading `Global.Typography.Body`, and process teardown reports permission errors.
They reproduce without the assignment changes.
The full Python unit suite passes, but the broader suite is not fully green.

High coverage alone is unconvincing: a call can execute without checking its result, and mocks cannot prove native transport works.
These tests check returned values, Arrow contents, exceptions, routing arguments, and observable subprocess output.
The complete broader suite was also run; its remaining Viewer failure and exact totals are recorded with the matched results.
The entire multi-language workspace was not run.

## AI assessment and the riskiest component to change

Codex helped identify initialization order as the reason fresh-process tests were necessary and preserved separate failing and passing commits.
It initially mistook the lack of directly named tests for a gap in `build_column_args`; coverage showed existing indirect exercise, so that target was replaced rather than counted toward the three.
It also initially invoked tests under socket restrictions and with an older system Viewer, producing misleading environment failures.
The AI Use Log records those corrections.
The component least comfortable to change without tests is `script_setup`: a small precedence change can send recordings to a different destination, create a file, or launch a viewer instead of respecting headless operation.
The tests pin those choices down, while the integration-suite limitation prevents claiming that every external operation is verified.
