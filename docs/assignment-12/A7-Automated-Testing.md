# A7 Automated Testing

The final assignment work connects characterization and defect fixing at Rerun's native log-channel boundary.
Part 2 selects three existing Rust methods with no pre-existing test cases found in the baseline audit: `LogSender::flush_blocking`, `LogReceiver::recv_timeout`, and `LogReceiverSet::remove_by_uri`.
Their twelve same-file characterization tests cover normal delivery, timeout and disconnection boundaries, and exact source-removal behavior.
The crate already had four tests for other behavior; all sixteen passed after characterization.
The [codebase audit](Part2-Codebase-Audit.md) records the Rust, C++, Python/notebook, and JavaScript/browser areas reviewed and the test locations checked.

## What testing revealed about the design

The sender, receiver, and receiver set have separate responsibilities but share a protocol involving queue order, callbacks, timeouts, and connection lifetime.
A receiver that polls independently can hide a sender notification defect.
Using real channels and a consumer that waits for its registered wake callback exposed the missing case.
This relates to Low Coupling and Protected Variations: acknowledgment and scheduling are dependencies that tests must model explicitly.
The receiver owns its lazy connection-state transition, consistent with Information Expert.
URI matching currently branches on concrete source variants; source-owned matching would improve High Cohesion if those policies expand.
These larger design changes remain proposals.

## Real defect and smallest fix

`LogSender::flush_blocking` queued its flush marker, waited for acknowledgment, and only then woke the receiver.
A wake-driven receiver could not process the marker while the sender waited.
A new regression test genuinely failed with `Err(Timeout), receiver woke: false` and was committed failing as `e91844fba1`.
Green, `ea5ae0246b`, moves the existing wake call before the acknowledgment wait.
The complete affected crate suite passes: **17 tests, 0 skipped**.
Refactor extracts the repeated wake code into a private helper while keeping all tests unchanged.
The final full affected-crate run still passes all seventeen tests.
See [Part3-Native-Flush.md](Part3-Native-Flush.md) and the saved Red, Green, and Refactor output for evidence.

A targeted upstream issue search found no matching open flush/waker report; this uses the assignment's characterization-discovered-defect option.
The previously researched issue #9211 is unrelated to the selected channel methods and was not fixed here.
The native flush defect is worth reporting upstream, but no issue, comment, or PR was posted.

## Testing Evolution

Native crate results progressed from **4 passing tests** in the inherited baseline to **16 passing tests** after characterization, then **17 passing tests** after the regression and fix.
The full affected crate was run for Green and Refactor, not just the regression.
Clippy with all features/targets and warnings denied, Rust formatting, and diff checks pass.
The sender test module matches the Red version byte-for-byte.
These checks do not claim an entire-workspace or downstream integration run.

Earlier Python work remains additional history, including its independent strict-mode startup fix and coverage reports.
Those reports measured Python line coverage of **74.54% before and 74.74% after**, and branch coverage of **50.25% before and 50.82% after**.
They do not measure this Rust fix or whole-project coverage.
Coverage measures execution, not useful assertions; the native tests inspect actual payloads, connection state, sender-side disconnection, notification, and acknowledgment.

## AI assessment and the component most risky to change

Codex helped trace the callback-ordering defect and preserve genuinely failing and passing commits.
It initially restricted selection to Python and confused a lack of directly named tests with missing exercise of `build_column_args`.
The student's correction led to a broader test-location audit and the native method selection.
The first native regression harness also used an unbounded notification channel; Clippy prompted correction to a bounded channel before the final Red commit was recorded.
The student should review this AI assessment in their own voice.

The component least comfortable to change without tests is `LogSender::flush_blocking`: notification and acknowledgment order determine whether a delivery guarantee succeeds, times out, or can wait indefinitely.
The new regression demonstrates why a simple happy-path test was insufficient.
