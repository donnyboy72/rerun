# Part 2 codebase audit and revised characterization targets

Part 2 was reassessed across the repository rather than limited to the Python SDK.
The audit used the original assignment baseline `151102da35` and the unchanged Rust implementation at `9b6e787973`.
The original Python characterization tests remain useful additional tests; the three primary Part 2 targets are now native Rust functions in `re_log_channel`.
This is a static test-location and candidate audit, followed by execution of the affected crate's entire test suite.
It is not a claim that every function was manually inspected or that the entire multi-language workspace was run under coverage.

## Search method

The search included production files, inline `#[test]` functions and `#[cfg(test)]` modules, separate source test modules, crate `tests/` directories, and the shared Rust integration suite.
For Python, it included SDK unit/integration tests and interactive test programs.
For C++, it included the Catch2 suite and its recursive CMake discovery.
For JavaScript/TypeScript, it included wrapper source, package scripts, and the shared browser integration harness; absence of a colocated `.test.ts` file was not treated as proof that the web viewer was untested.
Generated type files were excluded as edit targets.

Useful reproducible searches from the repository root:

```sh
rg --files crates rerun_py rerun_cpp rerun_js rerun_notebook tests
git grep -n -E '#\[cfg\(test\)\]|#\[test\]|mod .*test' 151102da35 -- crates
git grep -n -E 'LogReceiver|LogSender|remove_by_uri' 151102da35 -- '*test*.rs' '**/tests/**'
git grep -n -E 'flush_blocking|recv_timeout|remove_by_uri' 151102da35 -- crates/store/re_log_channel
git grep -n 'RrdManifest::merge' 151102da35 -- crates tests
```

Search matches were inspected in context: generic names such as `flush_blocking` appear on several unrelated types.
The selected crate already had four tests, so the claim is that these three specific functions lacked dedicated behavioral assertions, not that the whole crate was untested.
Its existing receiver-set test calls the set's `recv_timeout`, which uses underlying quota-channel receivers rather than calling `LogReceiver::recv_timeout`.

## Repository areas reviewed

| Area | Existing test locations or concrete evidence | Selection decision |
| --- | --- | --- |
| Rust storage and indexing | `re_chunk/tests/integration`, inline chunk-batcher tests, `re_chunk_optimizer/tests/integration`, and `re_log_encoding/tests/integration/footers_and_manifests.rs`. | Rejected manifest merging: it already has direct integration assertions. Rejected `InMemoryChunkProvider` as an untested component: optimizer integration tests use it. |
| Rust SDK and ingestion | Inline tests in `crates/top/re_sdk/src/recording_stream.rs`; importer integration tests under `crates/data_flow/re_importer/tests/integration`; client write tests in `re_grpc_client/src/write.rs`. | Did not call SDK flushing or importers untested merely because tests are not beside every method. |
| Rust channel infrastructure | `re_log_channel/src/lib.rs` contains three URL-display tests; `receiver_set.rs` contains a receive-set test. `sender.rs` and `receiver.rs` had no inline tests. | Selected the three function-level gaps below. They govern delivery acknowledgments, connection state, and source disconnection. |
| Rust Viewer and blueprints | Shared UI tests in `tests/rust/re_integration_test/tests/integration`; separate `test_view_class` module in `re_viewport_blueprint`. The earlier duplication branch also has `duplicate_tests.rs`. | Existing test modules must be checked in addition to the production file and crate `tests/` directory. No change to the earlier duplication work. |
| Rust utility and build code | Inline tests in quota-channel implementations and `re_types_builder/src/attributes.rs` and `codegen/definitions/mod.rs`; backoff doctest examples and unit tests. | Did not choose generated output or trivial wrappers as the three targets. |
| C++ SDK | `rerun_cpp/tests/CMakeLists.txt` recursively includes `.cpp`/`.hpp` files; Catch2 scenarios in `recording_stream.cpp` cover creation, logging, files, and gRPC sinks. | Reviewed `entity_path.cpp` and `time_column.cpp` as potential gaps. Their C FFI and Arrow dependencies make meaningful failure-path checks more involved; no claim that either has zero indirect coverage. |
| Python SDK and notebook | `rerun_py/tests/unit`, `integration`, `e2e_redap_tests`, and `rerun_notebook/tests`; interactive programs in `tests/python`. | Retained the existing 34 assignment tests as additional work rather than the sole basis for Part 2 selection. |
| JavaScript/TypeScript and browser | `rerun_js/web-viewer/index.ts`, React wrapper, package scripts, and `tests/rust/re_integration_test/src/inspection/connection/browser.rs`. | Download-progress and lifecycle code are possible future targets. Browser integration coverage exists; no whole-web-viewer gap is claimed from missing TypeScript test filenames. |

The final three targets share a crate because they form one important ingestion boundary, not because the audit was limited to that crate.
The assignment permits functions as targets and does not require three languages or three crates.

## Target 1 LogSender flush acknowledgment

Production: `crates/store/re_log_channel/src/sender.rs`, `LogSender::flush_blocking`.
New tests are in the same file under a native-only test module.

- Normal: previously queued recording data is received before the flush marker, the callback acknowledges completion, and flush succeeds.
- Boundary: a zero timeout without an acknowledgment returns `FlushError::Timeout` but still queues the flush marker.
- Failure: a missing receiver returns `FlushError::Closed`; dropping the callback without acknowledgment also returns Closed.

This matters because a successful flush is a delivery guarantee used by the SDK's gRPC-server sink.
The tests use real channels and a scoped consumer thread rather than mocking a successful acknowledgment.
They avoid sleeps and use bounded five-second waits only to prevent a hang if behavior regresses.

## Target 2 LogReceiver timeout and connection state

Production: `crates/store/re_log_channel/src/receiver.rs`, `LogReceiver::recv_timeout`.
New tests are in the same file under a native-only test module.

- Normal: a queued message preserves its source and recording identifier.
- Boundary: zero-timeout polling of an empty channel with a live sender returns Timeout without marking the receiver disconnected.
- Failure: polling after sender disconnection returns Disconnected and updates connection state.
- Additional boundary: queued data remains readable after the last sender is dropped; disconnection is observed only after the queue drains.

This matters because temporary silence must not be mistaken for a permanently closed source, and queued recording data must survive sender shutdown.

## Target 3 LogReceiverSet source removal

Production: `crates/store/re_log_channel/src/receiver_set.rs`, `LogReceiverSet::remove_by_uri`.
New tests are in a same-file `#[cfg(test)] mod tests` block.

- Normal: matching HTTP, proxy, and dataset sources are disconnected while an unrelated SDK source remains usable.
- Boundary: empty sets, empty input, duplicate matching sources, and local/JavaScript source names that resemble a URI preserve current behavior.
- Failed-match path: unknown or malformed URI text and differing query strings remove nothing; the method returns no error and does not parse the input.
- Disconnection is checked from the sender side with `TrySendError::Disconnected`, not only by counting entries.

This matters because Viewer source controls must disconnect the requested stream without disrupting unrelated ingestion.
The failed-match behavior is characterization, not a proposed normalization or validation feature.

## Validation and limitations

Baseline command:

```sh
PATH="$PWD/.pixi/envs/default/bin:$PATH" cargo nextest run --all-features --no-fail-fast -p re_log_channel
```

Before: **4 tests passed, 0 skipped**.
After: **16 tests passed, 0 skipped**, including all 12 new characterization cases.
See `part2-rust-before-tests.txt` and `part2-rust-after-tests.txt` for complete output.
`cargo clippy -p re_log_channel --all-features --all-targets`, `pixi run rs-fmt`, and `git diff --check` pass.
The diff adds only test modules to the three Rust files; production behavior and dependencies are unchanged.
The blocking flush and receive-timeout tests are excluded from wasm because those methods do not exist there.

The earlier Python coverage reports still measure Python only.
These Rust results must not be added to their coverage numerator or denominator, and no new Rust coverage percentage is claimed.
The pre-existing two Viewer lifecycle failures remain recorded with the earlier Python integration results; the Rust channel suite has no failures.

## Testability findings

- Flush acknowledgment couples channel delivery to a receiver-owned callback and wall-clock timeout. Scoped threads and explicit acknowledgment make the tests deterministic; a scheduler/clock seam would help if testing real elapsed-time policy became necessary. This relates to Low Coupling and Protected Variations.
- Receiver connection state is intentionally lazy and backed by an atomic flag. It must be tested together with channel lifetime, not as an isolated Boolean getter. Keeping disconnection transitions with the receiver follows Information Expert.
- URI removal branches on concrete source variants and compares display strings. The same-file tests reach the existing policy without a new public seam; a source-owned matching operation could improve High Cohesion and Protected Variations if matching rules expand.
