use log::{debug, warn};
use redis::{
    streams::{
        StreamAutoClaimOptions, StreamAutoClaimReply, StreamId, StreamReadOptions, StreamReadReply,
    },
    Commands, RedisResult, ToRedisArgs,
};

use crate::streams::consumer::models::*;
#[allow(unused_imports)]
use crate::types::{Id, RedsumerError, RedsumerResult};

fn create_consumer_group<C, K, G, ID>(
    conn: &mut C,
    key: K,
    group: G,
    since_id: ID,
) -> RedisResult<bool>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    ID: ToRedisArgs,
{
    match conn.xgroup_create::<_, _, _, String>(key, group, since_id) {
        Ok(_) => {
            debug!("The consumer group was successfully created");
            Ok(true)
        }
        Err(error) => {
            if error.to_string().contains("BUSYGROUP") {
                debug!("The consumer group already exists");
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

trait UnwrapStreamReadReply<K> {
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

fn read_new_messages<C, K, G, N>(
    conn: &mut C,
    key: &K,
    group: &G,
    consumer: &N,
    count: usize,
    block: usize,
) -> RedisResult<NewMessagesReply>
where
    C: Commands,
    K: ToRedisArgs + ToString,
    G: ToRedisArgs,
    N: ToRedisArgs,
{
    let new_messages: Vec<StreamId> = conn
        .xread_options::<_, _, StreamReadReply>(
            &[key],
            &[">"],
            &StreamReadOptions::default()
                .group(group, consumer)
                .count(count)
                .block(block),
        )?
        .unwrap_by_key(key);

    Ok(NewMessagesReply::build(new_messages))
}

fn read_pending_messages<C, K, G, N, ID>(
    conn: &mut C,
    key: &K,
    group: &G,
    consumer: &N,
    latest_pending_message_id: ID,
    count: usize,
) -> RedisResult<PendingMessagesReply>
where
    C: Commands,
    K: ToRedisArgs + ToString,
    G: ToRedisArgs,
    N: ToRedisArgs,
    ID: ToRedisArgs,
{
    let pending_messages: Vec<StreamId> = conn
        .xread_options::<_, _, StreamReadReply>(
            &[key],
            &[latest_pending_message_id],
            &StreamReadOptions::default()
                .group(group, consumer)
                .count(count),
        )?
        .unwrap_by_key(key);

    let latest_pending_message_id: Option<Id> = pending_messages.last().map(|s| s.id.to_owned());

    Ok(PendingMessagesReply::build(
        pending_messages,
        latest_pending_message_id,
    ))
}

fn claim_pending_messages<C, K, G, N, ID>(
    conn: &mut C,
    key: &K,
    group: &G,
    consumer: &N,
    min_idle_time: usize,
    next_id_to_claim: ID,
    count: usize,
) -> RedisResult<ClaimedMessagesReply>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    N: ToRedisArgs,
    ID: ToRedisArgs,
{
    let reply: StreamAutoClaimReply = conn
        .xautoclaim_options::<_, _, _, _, _, StreamAutoClaimReply>(
            key,
            group,
            consumer,
            min_idle_time,
            next_id_to_claim,
            StreamAutoClaimOptions::default().count(count),
        )?;

    let claimed_messages: Vec<StreamId> = reply.claimed.to_owned();
    let next_id_to_claim: Option<String> = Some(reply.next_stream_id.to_owned());

    Ok(ClaimedMessagesReply::build(
        claimed_messages,
        next_id_to_claim,
    ))
}

fn consume<C, K, G, N, ID>(
    c: &mut C,
    key: K,
    group: G,
    consumer: N,
    read_new_messages_options: ReadNewMessagesOptions,
    read_pending_messages_options: ReadPendingMessagesOptions<ID>,
    claim_messages_options: ClaimMessagesOptions<ID>,
) -> RedisResult<ConsumeReply>
where
    C: Commands,
    K: ToRedisArgs + ToString,
    G: ToRedisArgs,
    N: ToRedisArgs,
    ID: ToRedisArgs,
{
    let new_messages: NewMessagesReply = match read_new_messages_options.get_count().gt(&0) {
        true => read_new_messages(
            c,
            &key,
            &group,
            &consumer,
            read_new_messages_options.get_count(),
            read_new_messages_options.get_block(),
        )?,
        false => NewMessagesReply::empty(),
    };
    if !new_messages.is_empty() {
        return Ok(ConsumeReply::from(ConsumeReplyRepr::New(new_messages)));
    }

    let pending_messages: PendingMessagesReply =
        match read_pending_messages_options.get_count().gt(&0) {
            true => read_pending_messages(
                c,
                &key,
                &group,
                &consumer,
                read_pending_messages_options.get_latest_pending_message_id(),
                read_new_messages_options.get_count(),
            )?,
            false => PendingMessagesReply::empty(),
        };
    if !pending_messages.is_empty() {
        return Ok(ConsumeReply::from(ConsumeReplyRepr::Pending(
            pending_messages,
        )));
    }

    let claimed_messages: ClaimedMessagesReply = match claim_messages_options.get_count().gt(&0) {
        true => claim_pending_messages(
            c,
            &key,
            &group,
            &consumer,
            claim_messages_options.get_min_idle_time(),
            claim_messages_options.get_next_id_to_claim(),
            claim_messages_options.get_count(),
        )?,
        false => ClaimedMessagesReply::empty(),
    };
    if !claimed_messages.is_empty() {
        return Ok(ConsumeReply::from(ConsumeReplyRepr::Claimed(
            claimed_messages,
        )));
    }

    Ok(ConsumeReply::from(ConsumeReplyRepr::Empty))
}

/// A trait that bundles methods for consuming messages from a Redis stream
pub trait ConsumerCommands<K, G>
where
    K: ToRedisArgs,
    G: ToRedisArgs,
{
    /// Create a consumer group in a Redis stream.
    ///
    /// # Arguments:
    /// - **key**: A stream key, which must implement the `ToRedisArgs` trait.
    /// - **group**: A consumers group, which must implement the `ToRedisArgs` trait.
    /// - **since_id**: The ID of the message to start consuming, which must implement the `ToRedisArgs` trait.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with [`bool`] if the consumer group was created successfully. Otherwise, a [`RedsumerError`] is returned.
    /// - If the consumer group is created, the method returns `true`.
    /// - If the consumer group already exists, the method returns `false`.
    fn create_consumer_group<ID>(&mut self, key: K, group: G, since_id: ID) -> RedsumerResult<bool>
    where
        ID: ToRedisArgs;

    /// Consume messages from stream according to the following steps:
    ///
    /// 1. Try to retrieve new messages. If new messages are found, they are returned as a result.
    /// 2. If no new messages are found, an attempt is made to retrieve pending messages. If pending messages are found, they are returned as a result.
    /// 3. If no pending messages are found, an attempt is made to claim pending messages. If claimed messages are found, they are returned as a result.
    /// 4. If no new, pending, or claimed messages are found, an empty list is returned as a result.
    ///
    /// # Arguments:
    /// - **key**: A stream key, which must implement the `ToRedisArgs` and `ToString` traits.
    /// - **group**: A consumers group, which must implement the `ToRedisArgs` trait.
    /// - **consumer**: A consumer name, which must implement the `ToRedisArgs` trait.
    /// - **new_messages_count**: The number of new messages to retrieve.
    /// - **pending_messages_count**: The number of pending messages to retrieve.
    /// - **claimed_messages_count**: The number of claimed messages to retrieve.
    /// - **min_idle_time**: The minimum idle time [milliseconds] for a message to be claimed.
    /// - **latest_pending_message_id**: The ID of the latest pending message, which must implement the `ToRedisArgs` trait.
    /// - **next_id_to_claim**: The ID of the next message to claim, which must implement the `ToRedisArgs` trait.
    /// - **block**: The time to block [seconds] while waiting for new messages.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with a [`ConsumeReply`] as a result. Otherwise, a [`RedsumerError`] is returned.
    fn consume<N, ID>(
        &mut self,
        key: K,
        group: G,
        consumer: N,
        read_new_messages_options: ReadNewMessagesOptions,
        read_pending_messages_options: ReadPendingMessagesOptions<ID>,
        claim_messages_options: ClaimMessagesOptions<ID>,
    ) -> RedsumerResult<ConsumeReply>
    where
        N: ToRedisArgs,
        ID: ToRedisArgs;
}

impl<C, K, G> ConsumerCommands<K, G> for C
where
    C: Commands,
    K: ToRedisArgs + ToString,
    G: ToRedisArgs,
{
    fn create_consumer_group<ID>(&mut self, key: K, group: G, since_id: ID) -> RedsumerResult<bool>
    where
        ID: ToRedisArgs,
    {
        create_consumer_group(self, key, group, since_id)
    }

    fn consume<N, ID>(
        &mut self,
        key: K,
        group: G,
        consumer: N,
        read_new_messages_options: ReadNewMessagesOptions,
        read_pending_messages_options: ReadPendingMessagesOptions<ID>,
        claim_messages_options: ClaimMessagesOptions<ID>,
    ) -> RedsumerResult<ConsumeReply>
    where
        N: ToRedisArgs,
        ID: ToRedisArgs,
    {
        consume(
            self,
            key,
            group,
            consumer,
            read_new_messages_options,
            read_pending_messages_options,
            claim_messages_options,
        )
    }
}

#[cfg(test)]
mod test_create_consumer_group {
    use redis::{cmd, ErrorKind, RedisError};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_create_non_existent_consumer_group() {
        // Define the key, group, and since_id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, &str>(
                cmd("XGROUP")
                    .arg("CREATE")
                    .arg(key)
                    .arg(group)
                    .arg(since_id),
                Ok("Ok"),
            )]);

        // Create the consumer group:
        let result: RedsumerResult<bool> = conn.create_consumer_group(key, group, since_id);

        // Verify the result:
        assert!(result.is_ok());
        assert!(result.unwrap())
    }

    #[test]
    fn test_create_existent_consumer_group() {
        // Define the key, group, and since_id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, &str>(
                cmd("XGROUP")
                    .arg("CREATE")
                    .arg(key)
                    .arg(group)
                    .arg(since_id),
                Err(RedisError::from((
                    ErrorKind::ResponseError,
                    "BUSYGROUP Consumer Group name already exists",
                ))),
            )]);

        // Create the consumer group:
        let result: RedsumerResult<bool> = conn.create_consumer_group(key, group, since_id);

        // Verify the result:
        assert!(result.is_ok());
        assert!(!result.unwrap())
    }

    #[test]
    fn test_create_consumer_group_error() {
        // Define the key, group, and since_id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, &str>(
                cmd("XGROUP")
                    .arg("CREATE")
                    .arg(key)
                    .arg(group)
                    .arg(since_id),
                Err(RedisError::from((ErrorKind::ResponseError, "XGROUP Error"))),
            )]);

        // Create the consumer group:
        let result: RedsumerResult<bool> = conn.create_consumer_group(key, group, since_id);

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_consume {
    use redis::{cmd, ErrorKind, RedisError, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_consume_messages_empty() {
        // Define the key, group, and consumer:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let consumer: &str = "my-consumer";

        // Define the read options:
        let read_new_messages_options: ReadNewMessagesOptions = ReadNewMessagesOptions::new(0, 0);

        // Define the read pending messages options:
        let read_pending_messages_options: ReadPendingMessagesOptions<&str> =
            ReadPendingMessagesOptions::new(0, "0");

        // Define the claim messages options:
        let claim_messages_options: ClaimMessagesOptions<&str> =
            ClaimMessagesOptions::new(0, 0, "0");

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![]);

        // Consume messages:
        let result: RedsumerResult<ConsumeReply> = conn.consume(
            key,
            group,
            consumer,
            read_new_messages_options,
            read_pending_messages_options,
            claim_messages_options,
        );

        // Verify the result:
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_consume_new_messages_ok() {
        // Define the key, group, and consumer:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let consumer: &str = "my-consumer";

        // Define the read options:
        let read_new_messages_options: ReadNewMessagesOptions = ReadNewMessagesOptions::new(2, 0);

        // Define the read pending messages options:
        let read_pending_messages_options: ReadPendingMessagesOptions<&str> =
            ReadPendingMessagesOptions::new(0, "0");

        // Define the claim messages options:
        let claim_messages_options: ClaimMessagesOptions<&str> =
            ClaimMessagesOptions::new(0, 0, "0");

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XREADGROUP")
                    .arg(
                        &StreamReadOptions::default()
                            .group(group, consumer)
                            .count(read_new_messages_options.get_count())
                            .block(read_new_messages_options.get_block()),
                    )
                    .arg("STREAMS")
                    .arg(&[key])
                    .arg(&[">"]),
                Ok(Value::Array(vec![Value::Map(vec![
                    (
                        Value::SimpleString("my-key".to_string()),
                        Value::Array(vec![Value::Map(vec![
                            (
                                Value::SimpleString("0-0".to_string()),
                                Value::Array(vec![
                                    Value::SimpleString("code".to_string()),
                                    Value::Int(0),
                                ]),
                            ),
                            (
                                Value::SimpleString("1-0".to_string()),
                                Value::Array(vec![
                                    Value::SimpleString("code".to_string()),
                                    Value::Int(1),
                                ]),
                            ),
                        ])]),
                    ),
                    (
                        Value::SimpleString("fake-key".to_string()),
                        Value::Array(vec![Value::Map(vec![(
                            Value::SimpleString("666-0".to_string()),
                            Value::Array(vec![
                                Value::SimpleString("code".to_string()),
                                Value::Int(666),
                            ]),
                        )])]),
                    ),
                ])])),
            )]);

        // Consume messages:
        let result: RedsumerResult<ConsumeReply> = conn.consume(
            key,
            group,
            consumer,
            read_new_messages_options,
            read_pending_messages_options,
            claim_messages_options,
        );

        // Verify the result:
        assert!(result.is_ok());

        let consume_reply: ConsumeReply = result.unwrap();
        assert!(consume_reply.contains_new_messages());
        assert!(consume_reply.get_latest_pending_message_id().is_none());
        assert!(consume_reply.get_next_id_to_claim().is_none());

        // Verify the messages:
        let messages: Vec<StreamId> = consume_reply.get_messages();
        assert!(messages.len().eq(&2));

        assert!(messages[0].id.eq("0-0"));
        assert!(messages[0].map.contains_key("code"));
        assert!(messages[0].get::<usize>("code").eq(&Some(0)));
    }

    #[test]
    fn test_consume_new_messages_error() {
        // Define the key, group, and consumer:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let consumer: &str = "my-consumer";

        // Define the read options:
        let read_new_messages_options: ReadNewMessagesOptions = ReadNewMessagesOptions::new(2, 0);

        // Define the read pending messages options:
        let read_pending_messages_options: ReadPendingMessagesOptions<&str> =
            ReadPendingMessagesOptions::new(0, "0");

        // Define the claim messages options:
        let claim_messages_options: ClaimMessagesOptions<&str> =
            ClaimMessagesOptions::new(0, 0, "0");

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XREADGROUP")
                    .arg(
                        &StreamReadOptions::default()
                            .group(group, consumer)
                            .count(read_new_messages_options.get_count())
                            .block(read_new_messages_options.get_block()),
                    )
                    .arg("STREAMS")
                    .arg(&[key])
                    .arg(&[">"]),
                Err(RedisError::from((
                    ErrorKind::ResponseError,
                    "XREADGROUP Error",
                ))),
            )]);

        // Consume messages:
        let result: RedsumerResult<ConsumeReply> = conn.consume(
            key,
            group,
            consumer,
            read_new_messages_options,
            read_pending_messages_options,
            claim_messages_options,
        );

        // Verify the result:
        assert!(result.is_err());
    }
}
