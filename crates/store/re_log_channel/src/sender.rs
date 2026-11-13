use std::sync::Arc;

use re_byte_size::SizeBytes as _;

use crate::{
    Channel, DataSourceMessage, LogSource, SendError, SmartMessage, SmartMessagePayload,
    TrySendError,
};

#[derive(Clone)]
pub struct LogSender {
    tx: re_quota_channel::Sender<SmartMessage>,
    source: Arc<LogSource>,
    channel: Arc<Channel>,
}

impl LogSender {
    pub(crate) fn new(
        tx: re_quota_channel::Sender<SmartMessage>,
        source: Arc<LogSource>,
        channel: Arc<Channel>,
    ) -> Self {
        Self {
            tx,
            source,
            channel,
        }
    }

    fn wake_receiver(&self) {
        if let Some(waker) = self.channel.waker.read().as_ref() {
            (waker)();
        }
    }

    /// Send a message to the receiver
    pub fn send(&self, msg: DataSourceMessage) -> Result<(), SendError<Box<DataSourceMessage>>> {
        let source = Arc::clone(&self.source);

        let payload = SmartMessagePayload::Msg(msg);

        self.tx
            .send(SmartMessage { source, payload })
            .map_err(|SendError(msg)| match msg.payload {
                SmartMessagePayload::Msg(msg) => SendError(Box::new(msg)),
                SmartMessagePayload::Flush { .. } | SmartMessagePayload::Quit(_) => unreachable!(),
            })?;
        self.wake_receiver();

        Ok(())
    }

    /// Tries to send a message to the receiver without blocking.
    ///
    /// Returns the message if the channel is full or disconnected.
    pub fn try_send(
        &self,
        msg: DataSourceMessage,
    ) -> Result<(), TrySendError<Box<DataSourceMessage>>> {
        let smart_msg = SmartMessage {
            source: Arc::clone(&self.source),
            payload: SmartMessagePayload::Msg(msg),
        };
        let size_bytes = smart_msg.total_size_bytes();

        self.tx.try_send(smart_msg, size_bytes).map_err(|err| {
            let map_msg = |msg: SmartMessage| match msg.payload {
                SmartMessagePayload::Msg(msg) => Box::new(msg),
                SmartMessagePayload::Flush { .. } | SmartMessagePayload::Quit(_) => unreachable!(),
            };

            match err {
                TrySendError::Full(msg) => TrySendError::Full(map_msg(msg)),
                TrySendError::Disconnected(msg) => TrySendError::Disconnected(map_msg(msg)),
            }
        })?;

        self.wake_receiver();

        Ok(())
    }

    /// Blocks until all previously sent messages have been received.
    ///
    /// Note: This is only implemented for non-wasm targets since we cannot make
    /// blocking calls on web.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn flush_blocking(&self, timeout: std::time::Duration) -> Result<(), crate::FlushError> {
        use crate::FlushError;

        let (tx, rx) = std::sync::mpsc::sync_channel(0); // oneshot
        self.tx
            .send(SmartMessage {
                source: Arc::clone(&self.source),
                payload: SmartMessagePayload::Flush {
                    on_flush_done: Box::new(move || {
                        tx.send(()).ok();
                    }),
                },
            })
            .map_err(|_ignored| FlushError::Closed)?;

        // A receiver waiting for this notification cannot acknowledge until it wakes.
        self.wake_receiver();

        rx.recv_timeout(timeout).map_err(|err| match err {
            std::sync::mpsc::RecvTimeoutError::Timeout => FlushError::Timeout,
            std::sync::mpsc::RecvTimeoutError::Disconnected => FlushError::Closed,
        })?;

        Ok(())
    }

    /// Used to indicate that a sender has left.
    ///
    /// This sends a message down the channel allowing the receiving end to know whether one of the
    /// sender has left, and if so why (if applicable).
    ///
    /// Using a [`LogSender`] after calling `quit` is undefined behavior: the receiving end is free
    /// to silently drop those messages (or worse).
    ///
    /// This is an _explicit_ quit.
    /// Dropping all senders will also close the channel and notify the receiver,
    /// but calling `quit` has the benefit of the receiver knowing
    /// that we reached the end (or there was an error).
    pub fn quit(
        &self,
        err: Option<Box<dyn std::error::Error + Send>>,
    ) -> Result<(), SendError<Box<SmartMessage>>> {
        self.tx
            .send(SmartMessage {
                source: Arc::clone(&self.source),
                payload: SmartMessagePayload::Quit(err),
            })
            .map_err(|SendError(msg)| SendError(Box::new(msg)))?;

        self.wake_receiver();

        Ok(())
    }

    /// Is the channel currently empty of messages?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tx.is_empty()
    }

    /// Number of messages in the channel right now.
    #[inline]
    pub fn len(&self) -> usize {
        self.tx.len()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use crate::{DataSourceMessage, FlushError, LogSource, SmartMessagePayload, log_channel};

    #[test]
    fn flush_when_receiver_waits_for_wake_notifies_before_waiting_for_ack() {
        let (sender, receiver) = log_channel(LogSource::Sdk);
        let receiver = Arc::new(receiver);
        let (wake_sender, wake_receiver) = std::sync::mpsc::sync_channel(1);
        receiver.set_waker(move || {
            wake_sender.send(()).ok();
        });

        std::thread::scope(|scope| {
            let consumer_receiver = Arc::clone(&receiver);
            let consumer = scope.spawn(move || {
                // Do not poll the data channel until its registered waker runs.
                if wake_receiver.recv_timeout(Duration::from_secs(5)).is_err() {
                    return false;
                }
                let marker = consumer_receiver.try_recv().unwrap();
                let SmartMessagePayload::Flush { on_flush_done } = marker.payload else {
                    panic!("expected a flush marker after waking");
                };
                on_flush_done();
                true
            });

            let result = sender.flush_blocking(Duration::from_secs(5));
            let received_wake = consumer.join().unwrap();
            assert!(
                result.is_ok(),
                "flush must wake its receiver before awaiting acknowledgment; got {result:?}, receiver woke: {received_wake}"
            );
            assert!(received_wake);
        });
        assert!(receiver.is_empty());
    }

    #[test]
    fn flush_when_acknowledged_delivers_prior_data_before_returning() {
        let (sender, receiver) = log_channel(LogSource::Sdk);
        let store_id = re_log_types::StoreId::empty_recording();
        sender
            .send(DataSourceMessage::RrdManifestComplete(store_id.clone()))
            .unwrap();

        std::thread::scope(|scope| {
            let consumer = scope.spawn(|| {
                let data = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
                assert!(matches!(
                    data.payload,
                    SmartMessagePayload::Msg(DataSourceMessage::RrdManifestComplete(id)) if id == store_id
                ));
                let flush = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
                let SmartMessagePayload::Flush { on_flush_done } = flush.payload else {
                    panic!("expected a flush after the data");
                };
                on_flush_done();
            });
            sender.flush_blocking(Duration::from_secs(5)).unwrap();
            consumer.join().unwrap();
        });
        assert!(receiver.is_empty());
    }

    #[test]
    fn flush_when_zero_timeout_and_no_ack_returns_timeout_but_queues_marker() {
        let (sender, receiver) = log_channel(LogSource::Sdk);
        assert!(matches!(
            sender.flush_blocking(Duration::ZERO),
            Err(FlushError::Timeout)
        ));
        let marker = receiver.try_recv().unwrap();
        assert!(matches!(marker.payload, SmartMessagePayload::Flush { .. }));
        assert!(receiver.is_empty());
    }

    #[test]
    fn flush_when_receiver_is_gone_returns_closed() {
        let (sender, receiver) = log_channel(LogSource::Sdk);
        drop(receiver);
        assert!(matches!(
            sender.flush_blocking(Duration::from_secs(5)),
            Err(FlushError::Closed)
        ));
    }

    #[test]
    fn flush_when_callback_is_dropped_without_ack_returns_closed() {
        let (sender, receiver) = log_channel(LogSource::Sdk);
        std::thread::scope(|scope| {
            let consumer = scope.spawn(|| {
                let marker = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
                assert!(matches!(marker.payload, SmartMessagePayload::Flush { .. }));
                drop(marker);
            });
            assert!(matches!(
                sender.flush_blocking(Duration::from_secs(5)),
                Err(FlushError::Closed)
            ));
            consumer.join().unwrap();
        });
    }
}
