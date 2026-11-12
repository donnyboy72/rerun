use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

use crate::{Channel, LogSource, SmartMessage, TryRecvError};

pub struct LogReceiver {
    rx: re_quota_channel::Receiver<SmartMessage>,
    channel: Arc<Channel>,
    source: Arc<LogSource>,
    connected: AtomicBool,
}

impl std::fmt::Debug for LogReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogReceiver")
            .field("source", &self.source)
            .field("connected", &self.connected.load(Relaxed))
            .finish_non_exhaustive()
    }
}

impl LogReceiver {
    pub(crate) fn new(
        rx: re_quota_channel::Receiver<SmartMessage>,
        channel: Arc<Channel>,
        source: Arc<LogSource>,
    ) -> Self {
        Self {
            rx,
            channel,
            source,
            connected: AtomicBool::new(true),
        }
    }

    /// Call this on each sent message.
    ///
    /// Can be used to wake up the receiver thread.
    pub fn set_waker(&self, waker: impl Fn() + Send + Sync + 'static) {
        *self.channel.waker.write() = Some(Box::new(waker));
    }

    /// Are we still connected?
    ///
    /// Once false, we will never be connected again: the source has run dry.
    ///
    /// This is only updated once one of the receive methods fails.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Relaxed)
    }

    #[cfg(not(target_arch = "wasm32"))] // Cannot block on web
    pub fn recv(&self) -> Result<SmartMessage, crate::RecvError> {
        let Ok(msg) = self.rx.recv() else {
            self.connected.store(false, Relaxed);
            return Err(crate::RecvError);
        };

        Ok(msg)
    }

    pub fn try_recv(&self) -> Result<SmartMessage, TryRecvError> {
        let msg = match self.rx.try_recv() {
            Ok(x) => x,
            Err(err) => {
                if err == TryRecvError::Disconnected {
                    self.connected.store(false, Relaxed);
                }
                return Err(err);
            }
        };

        Ok(msg)
    }

    #[cfg(not(target_arch = "wasm32"))] // Cannot block on web
    pub fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<SmartMessage, crate::RecvTimeoutError> {
        let msg = match self.rx.recv_timeout(timeout) {
            Ok(x) => x,
            Err(err) => {
                if err == crate::RecvTimeoutError::Disconnected {
                    self.connected.store(false, Relaxed);
                }
                return Err(err);
            }
        };

        Ok(msg)
    }

    pub(crate) fn inner(&self) -> &re_quota_channel::Receiver<SmartMessage> {
        &self.rx
    }

    /// Where is the data coming from?
    #[inline]
    pub fn source(&self) -> &LogSource {
        &self.source
    }

    /// Where is the data coming from?
    #[inline]
    pub fn source_arc(&self) -> Arc<LogSource> {
        self.source.clone()
    }

    /// Is the channel currently empty of messages?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }

    /// Number of messages in the channel right now.
    #[inline]
    pub fn len(&self) -> usize {
        self.rx.len()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::time::Duration;

    use crate::{DataSourceMessage, LogSource, RecvTimeoutError, SmartMessagePayload, log_channel};

    #[test]
    fn recv_timeout_when_data_is_queued_preserves_source_and_payload() {
        let source = LogSource::File {
            path: "recording.rrd".into(),
        };
        let (sender, receiver) = log_channel(source.clone());
        let store_id = re_log_types::StoreId::empty_recording();
        sender
            .send(DataSourceMessage::RrdManifestComplete(store_id.clone()))
            .unwrap();
        let message = receiver.recv_timeout(Duration::ZERO).unwrap();
        assert_eq!(*message.source, source);
        assert!(matches!(message.payload,
            SmartMessagePayload::Msg(DataSourceMessage::RrdManifestComplete(id)) if id == store_id));
        assert!(receiver.is_connected());
        assert!(receiver.is_empty());
    }

    #[test]
    fn recv_timeout_when_empty_with_live_sender_keeps_connection() {
        let (_sender, receiver) = log_channel(LogSource::Sdk);
        assert!(matches!(
            receiver.recv_timeout(Duration::ZERO),
            Err(RecvTimeoutError::Timeout)
        ));
        assert!(receiver.is_connected());
    }

    #[test]
    fn recv_timeout_when_sender_is_gone_marks_disconnected() {
        let (sender, receiver) = log_channel(LogSource::Sdk);
        drop(sender);
        assert!(receiver.is_connected());
        assert!(matches!(
            receiver.recv_timeout(Duration::ZERO),
            Err(RecvTimeoutError::Disconnected)
        ));
        assert!(!receiver.is_connected());
    }

    #[test]
    fn recv_timeout_when_sender_drops_drains_queued_data_before_disconnect() {
        let (sender, receiver) = log_channel(LogSource::Sdk);
        sender
            .send(DataSourceMessage::RrdManifestComplete(
                re_log_types::StoreId::empty_recording(),
            ))
            .unwrap();
        drop(sender);
        assert!(receiver.recv_timeout(Duration::ZERO).is_ok());
        assert!(receiver.is_connected());
        assert!(matches!(
            receiver.recv_timeout(Duration::ZERO),
            Err(RecvTimeoutError::Disconnected)
        ));
        assert!(!receiver.is_connected());
    }
}
