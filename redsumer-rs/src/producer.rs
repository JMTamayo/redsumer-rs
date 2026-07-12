//! Resources to produce messages in a stream.

use crate::domain::connection::ConnectionConfig;
use crate::domain::message::FieldValue;
use crate::domain::result::RedsumerResult;
use crate::infrastructure::redis::adapter::RedisStreamsAdapter;

pub use crate::application::ports::MessageProducer;
pub use crate::application::producer::{ProduceMessageReply, ProducerConfig, ProducerService};

/// Ready-to-use producer that connects to the message broker and publishes messages in a stream.
///
/// Create an instance with [`Self::new`] and call [`Self::produce_from_map`] or [`Self::produce_from_items`].
#[derive(Debug, Clone)]
pub struct Producer {
    inner: ProducerService<RedisStreamsAdapter>,
}

impl Producer {
    /// Build a new [`Producer`] instance.
    ///
    /// Before creating a new producer, the following validations are performed:
    ///
    /// - If the connection settings are invalid, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    /// - If the connection to the message broker cannot be established, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    ///
    /// # Arguments:
    /// - **connection**: Connection settings to reach the message broker.
    /// - **config**: Producer configuration parameters.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the new [`Producer`] instance. Otherwise, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn new(connection: &ConnectionConfig, config: &ProducerConfig) -> RedsumerResult<Producer> {
        let adapter: RedisStreamsAdapter = RedisStreamsAdapter::new(connection)?;
        Ok(Producer {
            inner: ProducerService::from_port(adapter, config)?,
        })
    }

    /// Get *config*.
    pub fn get_config(&self) -> &ProducerConfig {
        self.inner.get_config()
    }

    /// Produce a new message in the stream from a map.
    pub fn produce_from_map<K, V, M>(&mut self, map: M) -> RedsumerResult<ProduceMessageReply>
    where
        K: Into<String>,
        V: Into<FieldValue>,
        M: IntoIterator<Item = (K, V)>,
    {
        self.inner.produce_from_map(map)
    }

    /// Produce a new message in the stream from a list of items.
    pub fn produce_from_items<K, V>(
        &mut self,
        items: Vec<(K, V)>,
    ) -> RedsumerResult<ProduceMessageReply>
    where
        K: Into<String>,
        V: Into<FieldValue>,
    {
        self.inner.produce_from_items(items)
    }
}
