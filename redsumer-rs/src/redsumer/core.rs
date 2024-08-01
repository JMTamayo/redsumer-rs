use log::{debug, warn};
use redis::{
    streams::{
        StreamAutoClaimOptions, StreamAutoClaimReply, StreamId, StreamPendingCountReply,
        StreamReadOptions, StreamReadReply,
    },
    Client, Commands, ConnectionAddr, ConnectionInfo, ErrorKind, RedisConnectionInfo, RedisError,
    ToRedisArgs,
};

use super::client::ClientArgs;
use super::types::*;

/// To build a new instance of [`Client`].
pub trait RedisClientBuilder {
    /// Build a new instance of [`Client`].
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with a new instance of [`Client`]. Otherwise, a [`RedsumerError`] is returned.
    fn build(&self) -> RedsumerResult<Client>;
}

impl<'k, 'a> RedisClientBuilder for ClientArgs<'k, 'a> {
    fn build(&self) -> RedsumerResult<Client> {
        let addr: ConnectionAddr =
            ConnectionAddr::Tcp(String::from(self.get_host()), self.get_port());

        let username: Option<String> = self
            .get_credentials()
            .to_owned()
            .map(|c| c.get_user().to_string());

        let password: Option<String> = self
            .get_credentials()
            .to_owned()
            .map(|c| c.get_password().to_string());

        let redis: RedisConnectionInfo = RedisConnectionInfo {
            db: self.get_db(),
            username,
            password,
            protocol: self.get_protocol(),
        };

        Client::open(ConnectionInfo { addr, redis })
    }
}

/// A trait to verify the connection to the Redis server.
pub trait VerifyConnection {
    /// Verify the connection to the Redis server.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with `()` if the connection was verified successfully. Otherwise, a [`RedsumerError`] is returned.
    fn ping(&mut self) -> RedsumerResult<()>;
}

impl<C> VerifyConnection for C
where
    C: Commands,
{
    fn ping(&mut self) -> RedsumerResult<()> {
        match self.check_connection() {
            true => {
                debug!("Connection to the Redis server was verified successfully");
                Ok(())
            }
            false => {
                debug!("Connection to the Redis server could not be verified");
                Err(RedisError::from((
					ErrorKind::ClientError,
						"Connection Verification Error",
						"The connection to the Redis server could not be verified. Please verify the client configuration or server availability".to_string(),
					))
				)
            }
        }
    }
}

fn produce_from_map<C, K, M>(conn: &mut C, key: K, map: M) -> RedsumerResult<Id>
where
    C: Commands,
    K: ToRedisArgs,
    M: ToRedisArgs,
{
    conn.xadd_map::<_, _, _, Id>(key, "*", map)
}

fn create_consumers_group<C, K, G, ID>(
    conn: &mut C,
    key: K,
    group: G,
    since_id: ID,
) -> RedsumerResult<()>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    ID: ToRedisArgs,
{
    match conn.xgroup_create::<_, _, _, bool>(key, group, since_id) {
        Ok(_) => {
            debug!("Consumers group was created successfully");
            Ok(())
        }
        Err(error) => {
            if error.to_string().contains("BUSYGROUP") {
                debug!("Consumers group already exists");
                Ok(())
            } else {
                Err(error)
            }
        }
    }
}

fn read_new_messages<C, K, G, N>(
    conn: &mut C,
    key: &K,
    group: &G,
    consumer: &N,
    count: usize,
    block: usize,
) -> RedsumerResult<StreamReadReply>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    N: ToRedisArgs,
{
    conn.xread_options(
        &[key],
        &[">"],
        &StreamReadOptions::default()
            .group(group, consumer)
            .count(count)
            .block(block),
    )
}

fn read_own_pending_messages<C, K, G, N, ID>(
    conn: &mut C,
    key: &K,
    group: &G,
    consumer: &N,
    since_id: &ID,
    count: usize,
) -> RedsumerResult<StreamReadReply>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    N: ToRedisArgs,
    ID: ToRedisArgs,
{
    conn.xread_options(
        &[key],
        &[since_id],
        &StreamReadOptions::default()
            .group(group, consumer)
            .count(count),
    )
}

fn claim_pending_messages<C, K, G, N, ID>(
    conn: &mut C,
    key: &K,
    group: &G,
    consumer: &N,
    since_id: &ID,
    min_idle_time: usize,
    count: usize,
) -> RedsumerResult<StreamAutoClaimReply>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    N: ToRedisArgs,
    ID: ToRedisArgs,
{
    conn.xautoclaim_options(
        key,
        group,
        consumer,
        min_idle_time,
        since_id,
        StreamAutoClaimOptions::default().count(count),
    )
}

pub trait UnwrapStreamReadReply<K> {
    fn unwrap_by_key(&self, key: &K) -> Vec<StreamId>
    where
        K: ToString;
}

impl<K> UnwrapStreamReadReply<K> for StreamReadReply
where
    K: ToString,
{
    fn unwrap_by_key(&self, key: &K) -> Vec<StreamId> {
        let mut ids: Vec<StreamId> = Vec::new();
        for stream in self.keys.iter() {
            match stream.key.eq(&key.to_string()) {
                true => ids.extend(stream.ids.to_owned()),
                false => warn!("Unexpected stream name found: {}. ", stream.key),
            };
        }

        ids
    }
}

pub trait UnwrapStreamAutoClaimReply {
    fn unwrap(&self) -> (Vec<StreamId>, Option<Id>);
}

impl UnwrapStreamAutoClaimReply for StreamAutoClaimReply {
    fn unwrap(&self) -> (Vec<StreamId>, Option<Id>) {
        (
            self.claimed.to_owned(),
            Some(self.next_stream_id.to_owned()),
        )
    }
}

fn consume_messages_from_stream<C, K, G, N, ID>(
    conn: &mut C,
    key: K,
    group: G,
    consumer: N,
    since_id: ID,
    new_messages_count: usize,
    pending_messages_count: usize,
    claimed_messages_count: usize,
    min_idle_time: usize,
    block: usize,
) -> RedsumerResult<(Vec<StreamId>, Option<Id>)>
where
    C: Commands,
    K: ToRedisArgs + ToString,
    G: ToRedisArgs,
    N: ToRedisArgs,
    ID: ToRedisArgs,
{
    let new_messages: Vec<StreamId> = match new_messages_count.gt(&0) {
        true => read_new_messages(conn, &key, &group, &consumer, new_messages_count, block)?
            .unwrap_by_key(&key),
        false => Vec::new(),
    };
    if new_messages.len().gt(&0) {
        return Ok((new_messages, None));
    }

    let pending_messages: Vec<StreamId> = match pending_messages_count.gt(&0) {
        true => read_own_pending_messages(
            conn,
            &key,
            &group,
            &consumer,
            &since_id,
            claimed_messages_count,
        )?
        .unwrap_by_key(&key),
        false => Vec::new(),
    };
    if pending_messages.len().gt(&0) {
        return Ok((pending_messages, None));
    }

    let claimed_messages_reply: (Vec<StreamId>, Option<Id>) = match claimed_messages_count.gt(&0) {
        true => claim_pending_messages(
            conn,
            &key,
            &group,
            &consumer,
            &since_id,
            min_idle_time,
            claimed_messages_count,
        )?
        .unwrap(),
        false => (Vec::new(), None),
    };
    if claimed_messages_reply.0.len().gt(&0) {
        return Ok(claimed_messages_reply);
    }

    Ok((Vec::new(), None))
}

fn verify_stream_message_ownership<C, K, G, N, ID>(
    conn: &mut C,
    key: K,
    group: G,
    consumer: N,
    id: ID,
) -> RedsumerResult<bool>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    N: ToRedisArgs,
    ID: ToRedisArgs,
{
    Ok(conn
        .xpending_consumer_count::<_, _, _, _, _, _, StreamPendingCountReply>(
            key, group, &id, &id, 1, consumer,
        )?
        .ids
        .len()
        .gt(&0))
}

fn ack_stream_message<C, K, G, ID>(conn: &mut C, key: K, group: G, id: ID) -> RedsumerResult<bool>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    ID: ToRedisArgs,
{
    conn.xack(key, group, &[id])
}

/// A trait that bundles methods for producing messages in a Redis stream
pub trait ProducerCommands<K, M>
where
    K: ToRedisArgs,
    M: ToRedisArgs,
{
    /// Produce a message in a Redis stream from a map.
    ///
    /// # Arguments:
    ///	- **key**: A stream key, which must implement the `ToRedisArgs` trait.
    /// - **map**: A map with the message fields and values, which must implement the `ToRedisArgs` trait.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the message [`Id`] if the message was produced successfully. Otherwise, a [`RedsumerError`] is returned.
    fn produce_from_map(&mut self, key: K, map: M) -> RedsumerResult<Id>
    where
        K: ToRedisArgs,
        M: ToRedisArgs;
}

impl<C, K, M> ProducerCommands<K, M> for C
where
    C: Commands,
    K: ToRedisArgs,
    M: ToRedisArgs,
{
    fn produce_from_map(&mut self, key: K, map: M) -> RedsumerResult<Id> {
        produce_from_map(self, key, map)
    }
}

/// A trait that bundles methods for consuming messages from a Redis stream
pub trait ConsumerCommands<K, G, ID>
where
    K: ToRedisArgs + ToString,
    G: ToRedisArgs,
    ID: ToRedisArgs,
{
    /// Create a consumers group in a Redis stream.
    ///
    /// # Arguments:
    /// - **key**: A stream key, which must implement the `ToRedisArgs` trait.
    /// - **group**: A consumers group, which must implement the `ToRedisArgs` trait.
    /// - **since_id**: The ID of the message to start consuming, which must implement the `ToRedisArgs` trait.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with `()` if the consumers group was created successfully. Otherwise, a [`RedsumerError`] is returned.
    fn create_consumers_group(&mut self, key: K, group: G, since_id: ID) -> RedsumerResult<()>;

    /// Consume messages from a Redis stream.
    ///
    /// # Arguments:
    /// - **key**: A stream key, which must implement the `ToRedisArgs` trait.
    /// - **group**: A consumers group, which must implement the `ToRedisArgs` trait.
    /// - **consumer**: A consumer name, which must implement the `ToRedisArgs` trait.
    /// - **since_id**: The ID of the message to start consuming, which must implement the `ToRedisArgs` trait.
    /// - **new_messages_count**: The number of new messages to read, which must be a positive integer.
    /// - **pending_messages_count**: The number of pending messages to read, which must be a positive integer.
    /// - **claimed_messages_count**: The number of claimed messages to read, which must be a positive integer.
    /// - **min_idle_time**: The minimum idle time in milliseconds, which must be a positive integer.
    /// - **block**: The block time in seconds, which must be a positive integer.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with a tuple containing the following elements:
    /// - **0**: A vector of [`StreamId`] with the consumed messages.
    /// - **1**: An optional [`Id`] with the next stream id to use as the start argument for the next xautoclaim.
    fn consume_messages_from_stream<N>(
        &mut self,
        key: K,
        group: G,
        consumer: N,
        since_id: ID,
        new_messages_count: usize,
        pending_messages_count: usize,
        claimed_messages_count: usize,
        min_idle_time: usize,
        block: usize,
    ) -> RedsumerResult<(Vec<StreamId>, Option<Id>)>
    where
        N: ToRedisArgs;

    /// Verify if a specific message by *id* is still in consumer pending list.
    ///
    /// # Arguments:
    /// - **id**: Stream message id.
    ///
    ///  # Returns:
    ///  - A [`RedsumerResult`] containing a boolean value. If the message is still in consumer pending list, `true` is returned. Otherwise, `false` is returned. If an error occurs, a [`RedsumerError`] is returned.
    fn verify_stream_message_ownership<N>(
        &mut self,
        key: K,
        group: G,
        consumer: N,
        id: ID,
    ) -> RedsumerResult<bool>
    where
        N: ToRedisArgs;

    /// Ack a stream message.
    ///
    /// # Arguments:
    /// - **key**: A stream key, which must implement the `ToRedisArgs` trait.
    /// - **group**: A consumers group, which must implement the `ToRedisArgs` trait.
    /// - **id**: Stream message id.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with a boolean value. If the message was acked successfully, `true` is returned. Otherwise, `false` is returned. If an error occurs, a [`RedsumerError`] is returned.
    fn ack_stream_message(&mut self, key: K, group: G, id: ID) -> RedsumerResult<bool>;
}

impl<C, K, G, ID> ConsumerCommands<K, G, ID> for C
where
    C: Commands,
    K: ToRedisArgs + ToString,
    G: ToRedisArgs,
    ID: ToRedisArgs,
{
    fn create_consumers_group(&mut self, key: K, group: G, since_id: ID) -> RedsumerResult<()> {
        create_consumers_group(self, key, group, since_id)
    }

    fn consume_messages_from_stream<N>(
        &mut self,
        key: K,
        group: G,
        consumer: N,
        since_id: ID,
        new_messages_count: usize,
        pending_messages_count: usize,
        claimed_messages_count: usize,
        min_idle_time: usize,
        block: usize,
    ) -> RedsumerResult<(Vec<StreamId>, Option<Id>)>
    where
        N: ToRedisArgs,
    {
        consume_messages_from_stream(
            self,
            key,
            group,
            consumer,
            since_id,
            new_messages_count,
            pending_messages_count,
            claimed_messages_count,
            min_idle_time,
            block,
        )
    }

    fn verify_stream_message_ownership<N>(
        &mut self,
        key: K,
        group: G,
        consumer: N,
        id: ID,
    ) -> RedsumerResult<bool>
    where
        N: ToRedisArgs,
    {
        verify_stream_message_ownership(self, &key, &group, &consumer, &id)
    }

    fn ack_stream_message(&mut self, key: K, group: G, id: ID) -> RedsumerResult<bool> {
        ack_stream_message(self, key, group, id)
    }
}

#[cfg(test)]
mod test_redis_client_builder {
    use redis::ProtocolVersion;

    use super::*;

    #[test]
    fn test_redis_client_builder_ok() {
        // Create a new instance of ClientArgs with default port and db:
        let args: ClientArgs = ClientArgs::new(None, "localhost", 6377, 16, ProtocolVersion::RESP2);

        // Build a new instance of Client:
        let client_result: RedsumerResult<Client> = args.build();

        // Verify if the client is correct:
        assert!(client_result.is_ok());
    }
}

#[cfg(test)]
mod test_verify_connection {
    use redis::Client;
    use redis_test::MockRedisConnection;

    use super::*;

    #[test]
    fn test_ping_ok() {
        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![]);

        // Verify the connection to the server:
        assert!(conn.ping().is_ok());
    }

    #[test]
    fn test_ping_error() {
        // Create a client from a fake host:
        let mut client: Client = Client::open("redis://fakehost:6379/0").unwrap();

        // Ping the server:
        let ping_result: RedsumerResult<()> = client.ping();

        // Verify the connection to the server:
        assert!(ping_result.is_err());
        assert_eq!(ping_result.unwrap_err().to_string(), "Connection Verification Error - ClientError: The connection to the Redis server could not be verified. Please verify the client configuration or server availability");
    }
}

#[cfg(test)]
mod test_produce_from_map {
    use std::collections::BTreeMap;

    use redis::cmd;
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_produce_from_map_ok() {
        // Define the key and id:
        let key: &str = "my-key";
        let id: &str = "*";

        // Define the map:
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        map.insert("field", "value");

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Id>(
            cmd("XADD").arg(key).arg(id).arg(map.to_owned()),
            Ok(id.to_string()),
        )]);

        // Produce the message:
        let result: RedsumerResult<Id> = conn.produce_from_map(key, map);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[test]
    fn test_produce_from_map_error() {
        // Define the key and id:
        let key: &str = "my-key";
        let id: &str = "*";

        // Define the map:
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        map.insert("field", "value");

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Id>(
            cmd("XADD").arg(key).arg(id).arg(map.to_owned()),
            Err(RedisError::from((
                ErrorKind::ResponseError,
                "XADD Error",
                "XADD command failed".to_string(),
            ))),
        )]);

        // Produce the message:
        let result: RedsumerResult<Id> = conn.produce_from_map(key, map);

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_create_consumers_group {
    use redis::cmd;
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_create_consumers_group_ok() {
        // Define the key, group, and since id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XGROUP")
                .arg("CREATE")
                .arg(key)
                .arg(group)
                .arg(since_id),
            Ok(1),
        )]);

        // Create the consumers group:
        let result: RedsumerResult<()> = create_consumers_group(&mut conn, key, group, since_id);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_consumers_group_already_exists() {
        // Define the key, group, and since id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XGROUP")
                .arg("CREATE")
                .arg(key)
                .arg(group)
                .arg(since_id),
            Err(RedisError::from((
                ErrorKind::ResponseError,
                "BUSYGROUP Error",
                "BUSYGROUP Consumer Group already exists".to_string(),
            ))),
        )]);

        // Create the consumers group:
        let result: RedsumerResult<()> = create_consumers_group(&mut conn, key, group, since_id);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_consumers_group_error() {
        // Define the key, group, and since id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XGROUP")
                .arg("CREATE")
                .arg(key)
                .arg(group)
                .arg(since_id),
            Err(RedisError::from((
                ErrorKind::ResponseError,
                "XGROUP Error",
                "XGROUP command failed".to_string(),
            ))),
        )]);

        // Create the consumers group:
        let result: RedsumerResult<()> = create_consumers_group(&mut conn, key, group, since_id);

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_unwrap_xread_reply_by_key {
    use redis::streams::{StreamId, StreamKey};

    use super::*;

    #[test]
    fn test_unwrap_xread_response_by_key_ok() {
        // Define StreamReadReply:
        let reply: StreamReadReply = StreamReadReply {
            keys: vec![
                StreamKey {
                    key: "my-key".to_string(),
                    ids: vec![StreamId::default()],
                },
                StreamKey {
                    key: "another-key".to_string(),
                    ids: vec![StreamId::default()],
                },
                StreamKey {
                    key: "my-key".to_string(),
                    ids: vec![StreamId::default()],
                },
            ],
        };

        // Unwrap the response:
        let ids: Vec<StreamId> = reply.unwrap_by_key(&"my-key");

        // Verify the result:
        assert!(ids.len().eq(&2));
    }
}

#[cfg(test)]
mod test_read_new_messages {
    #[test]
    fn test_read_new_messages_ok() {
        //	TODO: Implement unitary tests for the read_new_messages function
    }
}
