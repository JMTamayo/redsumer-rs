use redis::{Client, ToRedisArgs};

use super::client::{ClientArgs, RedisClientBuilder};
use super::core::{ProducerCommands, VerifyConnection};

#[allow(unused_imports)]
use super::types::{Id, RedsumerError, RedsumerResult};

/// Define the configuration parameters to create a [`Producer`] instance.
#[derive(Debug, Clone)]
pub struct ProducerConfig<'d> {
    stream_name: &'d str,
}

impl<'d> ProducerConfig<'d> {
    /// Get **stream name**.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The stream name.
    fn get_stream_name(&self) -> &str {
        self.stream_name
    }

    /// Create a new [`ProducerConfig`] instance.
    ///
    /// # Arguments:
    /// - **stream_name**: The name of the stream where messages will be produced.
    ///
    /// # Returns:
    /// A new [`ProducerConfig`] instance.
    ///
    /// # Example:
    /// ```rust,no_run
    /// use redsumer::ProducerConfig;
    ///
    /// let config: ProducerConfig = ProducerConfig::new("my_stream");
    /// ```
    pub fn new(stream_name: &'d str) -> Self {
        ProducerConfig { stream_name }
    }
}

/// A producer implementation of Redis Streams.
///
/// This struct is responsible for producing messages in a specific stream.
#[derive(Debug, Clone)]
pub struct Producer<'p> {
    client: Client,
    config: ProducerConfig<'p>,
}

impl<'p> Producer<'p> {
    /// Get [`Client`].
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The [`Client`] parameters.
    fn get_client(&self) -> &Client {
        &self.client
    }

    /// Get *config*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// [`ProducerConfig`] parameters.
    fn get_config(&self) -> &ProducerConfig {
        &self.config
    }

    /// Build a new [`Producer`] instance.
    ///
    /// Before creating a new producer, the following validations are performed:
    ///
    /// - If client parameters are invalid, a [`RedsumerError`] is returned.
    /// - If connection to Redis server can not be established, a [`RedsumerError`] is returned.
    ///
    /// # Arguments:
    /// - **args**: A [`ClientArgs`] instance with the connection parameters to Redis server.
    /// - **config**: A [`ProducerConfig`] instance with the producer configuration parameters.
    ///
    /// # Returns:
    ///  A [`RedsumerResult`] with the new [`Producer`] instance. Otherwise, a [`RedsumerError`] is returned.
    ///
    /// # Example:
    /// ```rust,no_run
    /// use redsumer::{ClientArgs, Producer, ProducerConfig};
    ///
    /// let args: ClientArgs = ClientArgs::new(None, "localhost", 6379, 0);
    /// let config: ProducerConfig = ProducerConfig::new("my_stream");
    ///
    /// let producer: Producer = Producer::new(
    ///     args,
    ///     config,
    /// ).unwrap_or_else(|err| {
    ///    panic!("Error creating producer: {:?}", err);
    /// });
    /// ```
    pub fn new(args: ClientArgs, config: ProducerConfig<'p>) -> RedsumerResult<Producer<'p>> {
        let client: Client = args.build()?;
        client.get_connection()?.ping()?;

        Ok(Producer { client, config })
    }

    /// Produce a new message in stream.
    ///
    /// This method produces a new message in the stream setting the *Id* as "*", which means that Redis will generate a new *Id* for the message automatically with the current timestamp.
    ///
    /// If stream does not exist, it will be created.
    ///
    /// # Arguments:
    /// - **message**: Message to produce in stream. It must implement [`ToRedisArgs`].
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the [`Id`] of the produced message. Otherwise, a [`RedsumerError`] is returned.
    ///
    /// # Example:
    /// ```rust,no_run
    /// use std::collections::BTreeMap;
    /// use redsumer::{ClientArgs, Id, Producer, ProducerConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let args: ClientArgs = ClientArgs::new(None, "127.0.0.1", 80, 0);
    ///     let config: ProducerConfig = ProducerConfig::new("my_stream");
    ///
    ///     let producer: Producer = Producer::new(
    ///         args,
    ///         config,
    ///     ).unwrap_or_else(|err| {
    ///         panic!("Error creating producer: {:?}", err);
    ///     });
    ///
    ///     let mut message: BTreeMap<&str, String> = BTreeMap::new();
    ///     message.insert("name", String::from("Juan"));
    ///     message.insert("last_name", String::from("Pérez"));
    ///
    ///     let id: Id = producer.produce(message).await.unwrap_or_else(|err| {
    ///         panic!("Error producing message: {:?}", err);
    ///     });
    /// }
    /// ```
    pub async fn produce<M>(&self, message: M) -> RedsumerResult<Id>
    where
        M: ToRedisArgs,
    {
        self.get_client()
            .get_connection()?
            .produce_from_map(self.get_config().get_stream_name(), message)
    }
}

#[cfg(test)]
mod test_producer {
    use redis::ConnectionAddr;

    use super::*;

    #[test]
    fn test_producer_config() {
        // Create a new ProducerConfig instance.
        let config: ProducerConfig = ProducerConfig::new("my_stream");

        // Verify if the stream name is correct.
        assert_eq!(config.get_stream_name(), "my_stream");
    }

    #[test]
    fn test_producer_getters() {
        // Define the host, port and db to connect to Redis server.
        let host: &str = "localhost";
        let port: u16 = 6377;
        let db: i64 = 16;

        // Create a new ClientArgs instance.
        let args: ClientArgs = ClientArgs::new(None, host, port, db);

        // Build a new Client instance.
        let client: Client = args.build().unwrap();

        // Define the stream name.
        let stream_name: &str = "my_stream";

        // Create a new ProducerConfig instance.
        let config: ProducerConfig = ProducerConfig::new(stream_name);

        // Create a new Producer instance.
        let producer: Producer = Producer { client, config };

        // Verify if the client is correct.
        assert_eq!(
            producer.get_client().get_connection_info().addr,
            ConnectionAddr::Tcp(host.to_string(), port)
        );
        assert_eq!(producer.get_client().get_connection_info().redis.db, db);
        assert_eq!(
            producer.get_client().get_connection_info().redis.username,
            None
        );
        assert_eq!(
            producer.get_client().get_connection_info().redis.password,
            None
        );

        // Verify if the config is correct.
        assert_eq!(producer.get_config().get_stream_name(), stream_name);
    }

    #[test]
    fn test_new_producer_error() {
        // Create a new ProducerConfig instance.
        let config: ProducerConfig = ProducerConfig::new("my_stream");

        // Create a new ClientArgs instance.
        let args: ClientArgs = ClientArgs::new(None, "invalid_host", 6377, 16);

        // Create a new Producer instance.
        let producer_result: RedsumerResult<Producer> = Producer::new(args, config);

        // Verify if the error is correct.
        assert!(producer_result.is_err());
        assert_eq!(
            producer_result.unwrap_err().to_string(),
            "Connection Verification Error - ClientError: The connection to the Redis server could not be verified. Please verify the client configuration or server availability"
        );
    }
}
