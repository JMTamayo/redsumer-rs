use tracing::{debug, info};

use crate::application::ports::MessageProducer;
use crate::domain::message::{FieldValue, MessageId};
use crate::domain::result::RedsumerResult;
use crate::domain::stream::StreamName;

/// Define the configuration parameters to create a producer instance.
#[derive(Debug, Clone)]
pub struct ProducerConfig {
    /// Stream name where messages will be produced.
    stream_name: String,
}

impl ProducerConfig {
    /// Get **stream name**.
    pub fn get_stream_name(&self) -> &str {
        &self.stream_name
    }

    /// Create a new [`ProducerConfig`] instance.
    ///
    /// # Arguments:
    /// - **stream_name**: The name of the stream where messages will be produced.
    ///
    /// # Returns:
    /// A new [`ProducerConfig`] instance.
    pub fn new(stream_name: &str) -> Self {
        ProducerConfig {
            stream_name: stream_name.to_owned(),
        }
    }
}

/// Reply of a produced message in a stream.
#[derive(Debug, Clone)]
pub struct ProduceMessageReply {
    /// *ID* of the produced message.
    id: MessageId,
}

impl ProduceMessageReply {
    /// Get *ID* of the produced message.
    pub fn get_id(&self) -> &MessageId {
        &self.id
    }
}

/// Convert a [`MessageId`] to a [`ProduceMessageReply`] instance.
impl From<MessageId> for ProduceMessageReply {
    fn from(id: MessageId) -> Self {
        ProduceMessageReply { id }
    }
}

/// A producer of stream messages. It produces messages through any [`MessageProducer`] implementation, received in the constructor.
#[derive(Debug, Clone)]
pub struct ProducerService<P>
where
    P: MessageProducer,
{
    /// Implementation used to produce messages.
    port: P,

    /// Stream name where messages will be produced.
    stream: StreamName,

    /// Producer configuration parameters.
    config: ProducerConfig,
}

impl<P> ProducerService<P>
where
    P: MessageProducer,
{
    /// Get *config*.
    pub fn get_config(&self) -> &ProducerConfig {
        &self.config
    }

    /// Build a new [`ProducerService`] instance from a [`MessageProducer`] implementation.
    ///
    /// The stream name from the configuration is validated during construction.
    ///
    /// # Arguments:
    /// - **port**: The [`MessageProducer`] implementation used to produce messages.
    /// - **config**: Producer configuration parameters.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the new [`ProducerService`] instance. Otherwise, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn from_port(port: P, config: &ProducerConfig) -> RedsumerResult<ProducerService<P>> {
        debug!("Creating a new producer instance by: {:?}", config);

        let stream: StreamName = StreamName::new(config.get_stream_name())?;

        info!("Producer instance created successfully and it is ready to be used");

        Ok(ProducerService {
            port,
            stream,
            config: config.to_owned(),
        })
    }

    /// Produce a new message in the stream from a map.
    ///
    /// This method produces a new message in the stream. The message broker generates the message *ID* automatically with the current timestamp. If the stream does not exist, it will be created.
    ///
    /// # Arguments:
    /// - **map**: A map with the message fields. Field names must convert into [`String`] and values into [`FieldValue`].
    ///
    /// # Returns:
    /// - A [`RedsumerResult`] with a [`ProduceMessageReply`] instance. Otherwise, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn produce_from_map<K, V, M>(&mut self, map: M) -> RedsumerResult<ProduceMessageReply>
    where
        K: Into<String>,
        V: Into<FieldValue>,
        M: IntoIterator<Item = (K, V)>,
    {
        let fields: Vec<(String, FieldValue)> =
            map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();

        self.port
            .produce(&self.stream, &fields)
            .map(ProduceMessageReply::from)
    }

    /// Produce a new message in the stream from a list of items.
    ///
    /// This method produces a new message in the stream. The message broker generates the message *ID* automatically with the current timestamp. If the stream does not exist, it will be created.
    ///
    /// # Arguments:
    /// - **items**: A list of items with the message fields. Each item is a tuple with the field name and the value. Field names must convert into [`String`] and values into [`FieldValue`].
    ///
    /// # Returns:
    /// - A [`RedsumerResult`] with a [`ProduceMessageReply`] instance. Otherwise, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn produce_from_items<K, V>(
        &mut self,
        items: Vec<(K, V)>,
    ) -> RedsumerResult<ProduceMessageReply>
    where
        K: Into<String>,
        V: Into<FieldValue>,
    {
        self.produce_from_map(items)
    }
}

#[cfg(test)]
mod test_producer_config {
    use super::*;

    #[test]
    fn test_producer_config_new() {
        // Define the stream name.
        let stream_name: &str = "stream_name";

        // Create a new producer configuration.
        let config: ProducerConfig = ProducerConfig::new(stream_name);

        // Verify the result.
        assert_eq!(config.get_stream_name(), stream_name);
    }
}

#[cfg(test)]
mod test_produce_messages_reply {
    use super::*;

    #[test]
    fn test_produce_message_reply_from() {
        // Define the message ID.
        let id: MessageId = MessageId::new("1234567890-0");

        // Create a new produce message reply.
        let reply: ProduceMessageReply = ProduceMessageReply::from(id.to_owned());

        // Verify the result.
        assert_eq!(reply.get_id(), &id);
    }
}

#[cfg(test)]
mod test_producer_service {
    use std::collections::BTreeMap;

    use crate::domain::error::RedsumerError;

    use super::*;

    /// In-memory fake implementation of the [`MessageProducer`] port.
    struct FakeProducerPort {
        /// Messages received by the port, as (stream, fields) pairs.
        produced: Vec<(String, Vec<(String, FieldValue)>)>,

        /// If true, the port fails on produce.
        fail: bool,
    }

    impl FakeProducerPort {
        fn new(fail: bool) -> Self {
            FakeProducerPort {
                produced: Vec::new(),
                fail,
            }
        }
    }

    impl MessageProducer for FakeProducerPort {
        fn produce(
            &mut self,
            stream: &StreamName,
            fields: &[(String, FieldValue)],
        ) -> RedsumerResult<MessageId> {
            if self.fail {
                return Err(RedsumerError::ProduceError("produce failed".to_string()));
            }

            self.produced
                .push((stream.as_str().to_string(), fields.to_vec()));

            Ok(MessageId::new("1-0"))
        }
    }

    #[test]
    fn test_from_port_with_invalid_stream_name() {
        let port: FakeProducerPort = FakeProducerPort::new(false);
        let config: ProducerConfig = ProducerConfig::new("");

        let result: RedsumerResult<ProducerService<FakeProducerPort>> =
            ProducerService::from_port(port, &config);

        assert!(matches!(result, Err(RedsumerError::ConfigurationError(_))));
    }

    #[test]
    fn test_produce_from_map_ok() {
        let port: FakeProducerPort = FakeProducerPort::new(false);
        let config: ProducerConfig = ProducerConfig::new("my-stream");

        let mut producer: ProducerService<FakeProducerPort> =
            ProducerService::from_port(port, &config).unwrap();

        let mut map: BTreeMap<&str, FieldValue> = BTreeMap::new();
        map.insert("code", FieldValue::from(1));
        map.insert("name", FieldValue::from("test"));

        let reply: ProduceMessageReply = producer.produce_from_map(map).unwrap();

        assert_eq!(reply.get_id().as_str(), "1-0");
        assert_eq!(producer.port.produced.len(), 1);
        assert_eq!(producer.port.produced[0].0, "my-stream");
        assert_eq!(producer.port.produced[0].1.len(), 2);
    }

    #[test]
    fn test_produce_from_items_ok() {
        let port: FakeProducerPort = FakeProducerPort::new(false);
        let config: ProducerConfig = ProducerConfig::new("my-stream");

        let mut producer: ProducerService<FakeProducerPort> =
            ProducerService::from_port(port, &config).unwrap();

        let items: Vec<(&str, &str)> = vec![("field", "value")];
        let reply: ProduceMessageReply = producer.produce_from_items(items).unwrap();

        assert_eq!(reply.get_id().as_str(), "1-0");
        assert_eq!(producer.get_config().get_stream_name(), "my-stream");
    }

    #[test]
    fn test_produce_error_is_propagated() {
        let port: FakeProducerPort = FakeProducerPort::new(true);
        let config: ProducerConfig = ProducerConfig::new("my-stream");

        let mut producer: ProducerService<FakeProducerPort> =
            ProducerService::from_port(port, &config).unwrap();

        let items: Vec<(&str, &str)> = vec![("field", "value")];
        let result: RedsumerResult<ProduceMessageReply> = producer.produce_from_items(items);

        assert!(matches!(result, Err(RedsumerError::ProduceError(_))));
    }
}
