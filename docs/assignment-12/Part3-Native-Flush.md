# Part 3 native log-channel flush defect

The final Part 3 now matches Part 2: it fixes `LogSender::flush_blocking`, one of the three characterized native Rust methods.
The earlier Python startup fix is retained as additional historical work.
Upstream issue #9211 was researched but is unrelated to the Part 2 channel methods and was not reproduced or fixed here.
It is not the submitted native cycle.

## How the defect was found

Part 2 checked successful flush acknowledgment using a receiver that polled independently of notifications.
Reviewing that test against the channel's registered-waker API exposed a missing case: a receiver that only polls after its wake callback runs.
`Channel::waker` documents that senders should notify whenever a message is sent.
`LogReceiver::set_waker` exists for receivers that need notification before polling; the Viewer uses it to request repaint.

The old flush implementation queued a `SmartMessagePayload::Flush`, waited for its acknowledgment, and only then invoked the waker.
A receiver relying on that notification cannot consume and acknowledge the marker while the sender waits.
With a finite timeout, the method returns `FlushError::Timeout`; with an effectively unlimited timeout it can wait indefinitely.
No claim is made that every current channel consumer relies on wake-driven polling.

A targeted public upstream search for open issues containing `flush` and `waker` returned no matches during investigation.
That is not an exhaustive guarantee that no related issue exists.
The assignment permits a defect discovered by extending characterization, so this cycle uses that option rather than relabeling an unrelated open issue.
No upstream issue, comment, or PR was created.

## Red

Commit: `e91844fba1`, `A7-Red: Failing test for flush waiting before waking receiver`.

The same-file regression is `flush_when_receiver_waits_for_wake_notifies_before_waiting_for_ack` in `crates/store/re_log_channel/src/sender.rs`.
It installs a real wake callback, uses a bounded notification channel, and starts a scoped consumer that does not poll for data before receiving the notification.
The consumer acknowledges the real flush marker when it wakes.
The receiver remains alive throughout the check, so the original failure is a timeout rather than a test-induced disconnection.
The test uses no sleeps; five-second bounds prevent indefinite hanging.

```sh
PATH="$PWD/.pixi/envs/default/bin:$PATH" cargo nextest run --all-features --no-fail-fast -p re_log_channel flush_when_receiver_waits_for_wake
```

Captured result in `part3-native-red-tests.txt`:

```text
flush must wake its receiver before awaiting acknowledgment; got Err(Timeout), receiver woke: false
Summary: 1 test run: 0 passed, 1 failed, 16 skipped
```

The sixteen other tests were filtered out for this targeted Red run.
The Red commit contains no production fix.
An initial test-harness lint warning about an unbounded notification channel was corrected to `sync_channel(1)` before the final Red commit; the failure was rerun and captured with that final test.

## Green

Commit: `ea5ae0246b`, `A7-Green: Fix flush waking receiver after acknowledgment wait`.

The least change moves the existing wake block to immediately after queuing the flush marker and before `recv_timeout` waits for its callback.
No new API, dependency, thread, or acknowledgment mechanism was introduced.
The regression test was unchanged.

```sh
PATH="$PWD/.pixi/envs/default/bin:$PATH" cargo nextest run --all-features --no-fail-fast -p re_log_channel
```

Complete affected-crate result in `part3-native-green-tests.txt`:

```text
Summary: 17 tests run: 17 passed, 0 skipped
```

## Refactor

The four identical sender wake blocks were extracted into one private `wake_receiver` helper.
`send`, `try_send`, `flush_blocking`, and `quit` retain their Green behavior.
The flush call keeps the new ordering, with a comment explaining the dependency on waking before acknowledgment.
This reduces repetition without changing the locking policy or callback behavior.

The entire affected crate suite again reports **17 passed, 0 skipped** in `part3-native-refactor-tests.txt`.
Clippy with all features and targets and `-D warnings` passes; see `part3-native-clippy.txt`.
`pixi run rs-fmt` and `git diff --check` pass.
The complete sender test module was compared byte-for-byte against the Red commit; `part3-native-unchanged-tests.json` records its SHA-256 and successful comparison.
No other test source changed during Green or Refactor.

## Scope and remaining limits

The test result covers the whole affected `re_log_channel` crate, not only the regression.
It does not claim that the entire multi-language Rerun workspace or all downstream SDK/Viewer integrations were rerun.
Earlier Python coverage reports remain Python-only and describe the earlier cycle; no new native coverage percentage is inferred from passing tests.
The prior two headless Viewer failures remain historical integration limitations, separate from this completely passing native crate suite.

The defect is worth reporting upstream because waiting before notifying can make flush falsely time out even with a live consumer.
The reproduction and fix are small, and the test verifies the actual channel protocol rather than mocking success.
Reporting remains a separate user decision; this work is published only in the student's fork.
