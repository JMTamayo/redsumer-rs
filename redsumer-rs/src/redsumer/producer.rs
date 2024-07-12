use redis::{Client, Commands, ToRedisArgs};

use super::client::{ping, ClientArgs, RedisClientBuilder};

#[allow(unused_imports)]
use super::types::{Id, RedsumerError, RedsumerResult};

/// Define the configuration parameters to create a [`Producer`] instance.
#[derive(Clone)]
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
    /// A reference of the stream name.
    pub fn get_stream_name(&self) -> &str {
        self.stream_name
    }

    /// Create a new [`ProducerConfig`] instance.
    ///
    /// # Arguments:
    /// - **stream_name**: The name of the stream where messages will be produced.
    ///
    /// # Returns:
    /// A new [`ProducerConfig`] instance.
    pub fn new(stream_name: &'d str) -> Self {
        ProducerConfig { stream_name }
    }
}

/// A producer implementation of *Redis Streams*.
///
///  This struct is responsible for producing messages in a specific stream.
#[derive(Clone)]
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
    /// A reference to the [`Client`] parameters.
    fn get_client(&self) -> &Client {
        &self.client
    }

    /// Get *config*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// A reference to the [`ProducerConfig`] parameters.
    pub fn get_config(&self) -> &ProducerConfig {
        &self.config
    }

    /// Build a new [`Producer`] instance.
    ///
    /// Before creating a new producer, the following validations are performed:
    ///
    /// - If connection parameters are invalid, a [`RedsumerError`] is returned.
    /// - If connection to Redis server can not be established, a [`RedsumerError`] is returned.
    ///
    /// # Arguments:
    /// - **args**: A [`ClientArgs`] instance with the connection parameters to *Redis* server.
    /// - **config**: A [`ProducerConfig`] instance with the producer configuration parameters.
    ///
    ///  # Returns:
    ///  A [`RedsumerResult`] with the new [`Producer`] instance. Otherwise, a [`RedsumerError`] is returned.
    ///
    ///  # Example:
    ///	Create a new [`Producer`] instance.
    /// ```rust,no_run
    /// use redsumer::{ClientArgs, Producer, ProducerConfig};
    ///
    /// let args: ClientArgs = ClientArgs::new("localhost");
    /// let config: ProducerConfig = ProducerConfig::new("my_stream");
    ///
    /// let producer: Producer = Producer::new(
    ///     args,
    /// 	config,
    /// ).unwrap_or_else(|err| {
    ///    panic!("Error creating producer: {:?}", err);
    /// });
    /// ```
    pub fn new(args: ClientArgs, config: ProducerConfig<'p>) -> RedsumerResult<Producer<'p>> {
        let mut client: Client = args.build()?;
        ping(&mut client)?;

        Ok(Producer { client, config })
    }

    /// Produce a new message in stream.
    ///
    ///  This method produces a new message in the stream setting the *ID* as "*", which means that Redis will generate a new *ID* for the message automatically with the current timestamp.
    ///
    ///  If stream does not exist, it will be created.
    ///
    /// # Arguments:
    /// - **message**: Message to produce in stream. It must implement [`ToRedisArgs`].
    ///
    ///  # Returns:
    /// A [`RedsumerResult`] with the [`Id`] of the produced message. Otherwise, a [`RedsumerError`] is returned.
    pub async fn produce<M>(&self, message: M) -> RedsumerResult<Id>
    where
        M: ToRedisArgs,
    {
        self.get_client().get_connection()?.xadd_map::<_, _, _, Id>(
            self.get_config().get_stream_name(),
            "*",
            message,
        )
    }
}
