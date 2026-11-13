# Two minute native testing demo

Use the four current submission links in the wiki: native A7-Tests, A7-Red, A7-Green, and A7-Refactor.
The earlier Python cycle and issue #9211 research are not this demonstration.

1. **0:00–0:35:** Show native Red `e91844fba1` and `part3-native-red-tests.txt`.
   Explain the real failure: `Err(Timeout), receiver woke: false`.
   The consumer waits for the registered notification, so the flush marker cannot be acknowledged before that notification runs.
2. **0:35–1:00:** Show Green `ea5ae0246b`.
   The only production change moves the existing wake call before the acknowledgment wait.
   Show `part3-native-green-tests.txt`: the complete affected crate has 17 passed, 0 skipped.
3. **1:00–1:25:** Show Part 2's `recv_timeout_when_empty_with_live_sender_keeps_connection` in its same-file Rust test module.
   Explain that silence and disconnection are different results and state transitions.
4. **1:25–1:45:** Show the final native Refactor.
   `wake_receiver` removes repetition from four sender paths; the test module is unchanged from Red.
   Show `part3-native-refactor-tests.txt` and `part3-native-unchanged-tests.json`.
5. **1:45–2:00:** Explain the assistant's initial Python-only selection and the unbounded test-notification channel corrected before final Red.
   State that 17 passing tests covers the whole affected native crate, not the entire multi-language workspace.
