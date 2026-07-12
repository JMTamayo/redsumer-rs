//! Resources to consume messages from a stream.

use crate::domain::connection::ConnectionConfig;
use crate::domain::message::MessageId;
use crate::domain::result::RedsumerResult;
use crate::infrastructure::redis::adapter::RedisStreamsAdapter;

pub use crate::application::consumer::{
    AckMessageReply, ClaimMessagesOptions, ConsumeMessagesReply, ConsumerConfig, ConsumerService,
    IsStillMineReply, ReadNewMessagesOptions, ReadPendingMessagesOptions,
};
pub use crate::application::ports::MessageConsumer;

/// Ready-to-use consumer that connects to the message broker and reads messages from a stream.
///
/// Create an instance with [`Self::new`] and call [`Self::consume`], [`Self::is_still_mine`] and [`Self::ack`].
#[derive(Debug, Clone)]
pub struct Consumer {
    inner: ConsumerService<RedisStreamsAdapter>,
}

impl Consumer {
    /// Build a new [`Consumer`] instance.
    ///
    /// Before creating a new consumer, the following validations are performed:
    ///
    /// - If the connection settings are invalid, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    /// - If the connection to the message broker cannot be established, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    /// - If the stream does not exist, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    /// - If the consumers group does not exist, it is created based on the stream, group and the given *initial_stream_id*.
    ///
    /// # Arguments:
    /// - **connection**: Connection settings to reach the message broker.
    /// - **config**: Consumer configuration parameters.
    /// - **initial_stream_id**: The ID of the message to start consuming from.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] containing a [`Consumer`] instance. Otherwise, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn new(
        connection: ConnectionConfig,
        config: ConsumerConfig,
        initial_stream_id: Option<String>,
    ) -> RedsumerResult<Consumer> {
        let adapter: RedisStreamsAdapter = RedisStreamsAdapter::new(&connection)?;
        Ok(Consumer {
            inner: ConsumerService::from_port(
                adapter,
                config,
                initial_stream_id.map(MessageId::from),
            )?,
        })
    }

    /// Get *config*.
    pub fn get_config(&self) -> &ConsumerConfig {
        self.inner.get_config()
    }

    /// Consume messages from stream.
    pub fn consume(&mut self) -> RedsumerResult<ConsumeMessagesReply> {
        self.inner.consume()
    }

    /// Verify if a specific message by *id* is still in consumer pending list.
    pub fn is_still_mine(&mut self, id: &MessageId) -> RedsumerResult<IsStillMineReply> {
        self.inner.is_still_mine(id)
    }

    /// Ack a message by *id*.
    pub fn ack(&mut self, id: &MessageId) -> RedsumerResult<AckMessageReply> {
        self.inner.ack(id)
    }
}
