//! Low-level Redis Streams commands used internally to produce and consume messages.

use redis::{
    streams::{
        StreamAutoClaimOptions, StreamAutoClaimReply, StreamId, StreamPendingCountReply,
        StreamReadOptions, StreamReadReply,
    },
    Commands, RedisResult, ToRedisArgs,
};
use tracing::{debug, error, warn};

/// The ID that represents the beginning of time in a Redis stream.
pub(crate) const BEGINNING_OF_TIME_ID: &str = "0-0";

/// Get StreamIds from a StreamReadReply by key.
trait UnwrapStreamReadReply {
    /// Unwrap StreamReadReply by key into a `Vec<StreamId>`.
    ///
    /// # Arguments:
    /// - **key**: A key to filter the StreamReadReply.
    ///
    /// # Returns:
    /// A `Vec<StreamId>` with the StreamIds found.
    fn unwrap_by_key(&self, key: &str) -> Vec<StreamId>;
}

impl UnwrapStreamReadReply for StreamReadReply {
    fn unwrap_by_key(&self, key: &str) -> Vec<StreamId> {
        let mut ids: Vec<StreamId> = Vec::new();

        for stream in self.keys.iter() {
            match stream.key.eq(key) {
                true => ids.extend(stream.ids.to_owned()),
                false => warn!(
                    "An unexpected stream name found while extracting the key {}: {}. ",
                    key, stream.key,
                ),
            };
        }

        ids
    }
}

/// Verify if a stream exists in Redis Stream service.
pub(crate) fn stream_exists<C>(conn: &mut C, key: &str) -> RedisResult<bool>
where
    C: Commands,
{
    match conn.exists::<_, bool>(key) {
        Ok(exists) => {
            debug!("Stream existence verified: {}", exists);
            Ok(exists)
        }
        Err(e) => {
            error!("Error verifying if stream exists: {:?}", e);
            Err(e)
        }
    }
}

/// Create a consumer group in a stream. Returns `true` if the group was created, or `false` if it already existed.
pub(crate) fn create_consumer_group<C, ID>(
    conn: &mut C,
    key: &str,
    group: &str,
    since_id: ID,
) -> RedisResult<bool>
where
    C: Commands,
    ID: ToRedisArgs,
{
    match conn.xgroup_create::<_, _, _, String>(key, group, since_id) {
        Ok(_) => {
            debug!("The consumers group was successfully created");
            Ok(true)
        }
        Err(e) => {
            if e.to_string().contains("BUSYGROUP") {
                debug!("The consumer group already exists");
                Ok(false)
            } else {
                error!("Error creating consumer group: {:?}", e);
                Err(e)
            }
        }
    }
}

/// Read new messages from a stream.
pub(crate) fn read_new_messages<C>(
    conn: &mut C,
    key: &str,
    group: &str,
    consumer: &str,
    count: usize,
    block: usize,
) -> RedisResult<Vec<StreamId>>
where
    C: Commands,
{
    Ok(match count.gt(&0) {
        true => conn
            .xread_options::<_, _, StreamReadReply>(
                &[key],
                &[">"],
                &StreamReadOptions::default()
                    .group(group, consumer)
                    .count(count)
                    .block(block),
            )?
            .unwrap_by_key(key),
        false => Vec::new(),
    })
}

/// Read pending messages from a stream. Returns the messages and the next cursor ID.
pub(crate) fn read_pending_messages<C>(
    conn: &mut C,
    key: &str,
    group: &str,
    consumer: &str,
    latest_pending_message_id: &str,
    count: usize,
) -> RedisResult<(Vec<StreamId>, String)>
where
    C: Commands,
{
    match count.gt(&0) {
        true => {
            let pending_messages: Vec<StreamId> = conn
                .xread_options::<_, _, StreamReadReply>(
                    &[key],
                    &[latest_pending_message_id],
                    &StreamReadOptions::default()
                        .group(group, consumer)
                        .count(count),
                )?
                .unwrap_by_key(key);

            let latest_pending_message_id: String = match pending_messages.last() {
                Some(s) => s.id.to_owned(),
                None => BEGINNING_OF_TIME_ID.to_owned(),
            };

            Ok((pending_messages, latest_pending_message_id))
        }
        false => Ok((Vec::new(), BEGINNING_OF_TIME_ID.to_owned())),
    }
}

/// Claim pending messages from a stream. Returns the claimed messages and the next cursor ID.
pub(crate) fn claim_pending_messages<C>(
    conn: &mut C,
    key: &str,
    group: &str,
    consumer: &str,
    min_idle_time: usize,
    next_id_to_claim: &str,
    count: usize,
) -> RedisResult<(Vec<StreamId>, String)>
where
    C: Commands,
{
    match count.gt(&0) {
        true => {
            let reply: StreamAutoClaimReply = conn
                .xautoclaim_options::<_, _, _, _, _, StreamAutoClaimReply>(
                    key,
                    group,
                    consumer,
                    min_idle_time,
                    next_id_to_claim,
                    StreamAutoClaimOptions::default().count(count),
                )?;

            Ok((reply.claimed.to_owned(), reply.next_stream_id.to_owned()))
        }
        false => Ok((Vec::new(), BEGINNING_OF_TIME_ID.to_owned())),
    }
}

/// Get the pending state of a message in the consumer pending list.
///
/// Returns `Some((last_delivered_ms, times_delivered))` if the message is still in the consumer pending list, or `None` otherwise. If more than one entry is found for a single ID, an error is returned through the `Result` as an `Err(String)` detail.
pub(crate) fn pending_message_state<C>(
    conn: &mut C,
    key: &str,
    group: &str,
    consumer: &str,
    id: &str,
) -> RedisResult<Result<Option<(usize, usize)>, String>>
where
    C: Commands,
{
    let reply: StreamPendingCountReply = conn
        .xpending_consumer_count::<_, _, _, _, _, _, StreamPendingCountReply>(
            key, group, &id, &id, 1, consumer,
        )?;

    if reply.ids.len().gt(&1) {
        error!("More than one pending message found searching for a single message Id");
        return Ok(Err(
            "More than one pending message found searching for a single message Id".to_string(),
        ));
    }

    match reply.ids.first() {
        Some(m) => {
            debug!("The message is still in the consumer pending list");
            Ok(Ok(Some((m.last_delivered_ms, m.times_delivered))))
        }
        None => {
            debug!("The message is not in the consumer pending list");
            Ok(Ok(None))
        }
    }
}

/// Ack a message in a consumer group.
pub(crate) fn ack<C>(conn: &mut C, key: &str, group: &str, id: &str) -> RedisResult<bool>
where
    C: Commands,
{
    match conn.xack::<_, _, _, bool>(key, group, &[id]) {
        Ok(true) => {
            debug!("The message was successfully acknowledged");
            Ok(true)
        }
        Ok(false) => {
            debug!("The message was not acknowledged");
            Ok(false)
        }
        Err(e) => {
            error!("Error acknowledging message: {:?}", e);
            Err(e)
        }
    }
}

/// Produce a message to a Redis stream from a list of field/value pairs. To set the ID of the message, this function uses the value "*" to indicate that Redis should generate a new ID with the current timestamp.
pub(crate) fn produce<C>(
    conn: &mut C,
    key: &str,
    items: &[(String, Vec<u8>)],
) -> RedisResult<String>
where
    C: Commands,
{
    match conn.xadd(key, "*", items) {
        Ok(id) => {
            debug!("Message produced successfully");
            Ok(id)
        }
        Err(e) => {
            error!("Error producing message: {:?}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod test_produce {
    use redis::{cmd, ErrorKind, RedisError, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_produce_ok() {
        // Define the key:
        let key: &str = "my-key";

        // Define the items:
        let items: Vec<(String, Vec<u8>)> = vec![("field".to_string(), b"value".to_vec())];

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XADD").arg(key).arg("*").arg(&items),
                Ok(Value::SimpleString("1-0".to_string())),
            )]);

        // Produce the message:
        let result: RedisResult<String> = produce(&mut conn, key, &items);

        // Verify the result:
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1-0");
    }

    #[test]
    fn test_produce_error() {
        // Define the key:
        let key: &str = "my-key";

        // Define the items:
        let items: Vec<(String, Vec<u8>)> = vec![("field".to_string(), b"value".to_vec())];

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XADD").arg(key).arg("*").arg(&items),
                Err(RedisError::from((
                    ErrorKind::ResponseError,
                    "XADD Error",
                    "XADD command failed".to_string(),
                ))),
            )]);

        // Produce the message:
        let result: RedisResult<String> = produce(&mut conn, key, &items);

        // Verify the result:
        assert!(result.is_err());
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
        let result: RedisResult<bool> = create_consumer_group(&mut conn, key, group, since_id);

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
        let result: RedisResult<bool> = create_consumer_group(&mut conn, key, group, since_id);

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
        let result: RedisResult<bool> = create_consumer_group(&mut conn, key, group, since_id);

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_stream_exists {
    use redis::{cmd, ErrorKind, RedisError};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_stream_exists_true() {
        // Define the key:
        let key: &str = "my-key";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, i64>(cmd("EXISTS").arg(key), Ok(1))]);

        // Verify if the stream exists:
        let result: RedisResult<bool> = stream_exists(&mut conn, key);

        // Verify the result:
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_stream_does_not_exist() {
        // Define the key:
        let key: &str = "my-key";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, i64>(cmd("EXISTS").arg(key), Ok(0))]);

        // Verify if the stream exists:
        let result: RedisResult<bool> = stream_exists(&mut conn, key);

        // Verify the result:
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_stream_exists_error() {
        // Define the key:
        let key: &str = "my-key";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("EXISTS").arg(key),
            Err(RedisError::from((ErrorKind::ResponseError, "EXISTS Error"))),
        )]);

        // Verify if the stream exists:
        let result: RedisResult<bool> = stream_exists(&mut conn, key);

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_read_new_messages {
    use redis::{cmd, ErrorKind, RedisError, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_read_new_messages_with_zero_count() {
        // Define the key, group, consumer, count, and block:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let consumer: &str = "my-consumer";
        let count: usize = 0;
        let block: usize = 1;

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![]);

        // Read new messages:
        let result: RedisResult<Vec<StreamId>> =
            read_new_messages(&mut conn, key, group, consumer, count, block);

        // Verify the result:
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_read_new_messages_ok() {
        // Define the key, group, and consumer:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let consumer: &str = "my-consumer";
        let count: usize = 2;
        let block: usize = 1;

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XREADGROUP")
                    .arg(
                        StreamReadOptions::default()
                            .group(group, consumer)
                            .count(count)
                            .block(block),
                    )
                    .arg("STREAMS")
                    .arg(&[key])
                    .arg(&[">"]),
                Ok(Value::Array(vec![Value::Map(vec![
                    (
                        Value::SimpleString("my-key".to_string()),
                        Value::Array(vec![Value::Map(vec![(
                            Value::SimpleString("1-0".to_string()),
                            Value::Array(vec![
                                Value::SimpleString("code".to_string()),
                                Value::Int(1),
                            ]),
                        )])]),
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
        let result: RedisResult<Vec<StreamId>> =
            read_new_messages(&mut conn, key, group, consumer, count, block);

        // Verify the result:
        assert!(result.is_ok());

        // Verify the messages:
        let messages: Vec<StreamId> = result.unwrap();
        assert!(messages.len().eq(&1));

        assert!(messages[0].id.eq("1-0"));
        assert!(messages[0].map.get("code").unwrap().eq(&Value::Int(1)));
    }

    #[test]
    fn test_read_new_messages_error() {
        // Define the key, group, and consumer:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let consumer: &str = "my-consumer";
        let count: usize = 2;
        let block: usize = 1;

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XREADGROUP")
                    .arg(
                        StreamReadOptions::default()
                            .group(group, consumer)
                            .count(count)
                            .block(block),
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
        let result: RedisResult<Vec<StreamId>> =
            read_new_messages(&mut conn, key, group, consumer, count, block);

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_read_pending_messages {
    use redis::{cmd, ErrorKind, RedisError, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_read_pending_messages_with_zero_count() {
        // Define the key, group, consumer, latest_pending_message_id, and count:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let consumer: &str = "my-consumer";
        let latest_pending_message_id: &str = "0-0";
        let count: usize = 0;

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![]);

        // Read pending messages:
        let result: RedisResult<(Vec<StreamId>, String)> = read_pending_messages(
            &mut conn,
            key,
            group,
            consumer,
            latest_pending_message_id,
            count,
        );

        // Verify the result:
        assert!(result.is_ok());

        let (messages, next_id_to_claim): (Vec<StreamId>, String) = result.unwrap();
        assert!(messages.is_empty());
        assert!(next_id_to_claim.eq(BEGINNING_OF_TIME_ID));
    }

    #[test]
    fn test_read_pending_messages_empty() {
        // Define the key, group, consumer, latest_pending_message_id, and count:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let latest_pending_message_id = "0-0";
        let count = 2;

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XREADGROUP")
                    .arg(
                        StreamReadOptions::default()
                            .group(group, consumer)
                            .count(count),
                    )
                    .arg("STREAMS")
                    .arg(&[key])
                    .arg(&[latest_pending_message_id]),
                Ok(Value::Array(vec![Value::Map(vec![])])),
            )]);

        // Read pending messages:
        let result: RedisResult<(Vec<StreamId>, String)> = read_pending_messages(
            &mut conn,
            key,
            group,
            consumer,
            latest_pending_message_id,
            count,
        );

        // Verify the result:
        assert!(result.is_ok());

        let (messages, next_id_to_claim): (Vec<StreamId>, String) = result.unwrap();
        assert!(messages.len().eq(&0));
        assert!(next_id_to_claim.eq(BEGINNING_OF_TIME_ID));
    }

    #[test]
    fn test_read_pending_messages_ok() {
        // Define the key, group, consumer, latest_pending_message_id, and count:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let latest_pending_message_id = "0-0";
        let count = 2;

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XREADGROUP")
                    .arg(
                        StreamReadOptions::default()
                            .group(group, consumer)
                            .count(count),
                    )
                    .arg("STREAMS")
                    .arg(&[key])
                    .arg(&[latest_pending_message_id]),
                Ok(Value::Array(vec![Value::Map(vec![
                    (
                        Value::SimpleString("my-key".to_string()),
                        Value::Array(vec![Value::Map(vec![(
                            Value::SimpleString("1-0".to_string()),
                            Value::Array(vec![
                                Value::SimpleString("code".to_string()),
                                Value::Int(1),
                            ]),
                        )])]),
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

        // Read pending messages:
        let result: RedisResult<(Vec<StreamId>, String)> = read_pending_messages(
            &mut conn,
            key,
            group,
            consumer,
            latest_pending_message_id,
            count,
        );

        // Verify the result:
        assert!(result.is_ok());

        let (messages, next_id_to_claim): (Vec<StreamId>, String) = result.unwrap();
        assert!(messages.len().eq(&1));

        assert!(messages[0].id.eq("1-0"));
        assert!(messages[0].map.get("code").unwrap().eq(&Value::Int(1)));

        assert!(next_id_to_claim.eq("1-0"));
    }

    #[test]
    fn test_read_pending_messages_error() {
        // Define the key, group, consumer, latest_pending_message_id, and count:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let latest_pending_message_id = "0-0";
        let count = 2;

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XREADGROUP")
                    .arg(
                        StreamReadOptions::default()
                            .group(group, consumer)
                            .count(count),
                    )
                    .arg("STREAMS")
                    .arg(&[key])
                    .arg(&[latest_pending_message_id]),
                Err(RedisError::from((
                    ErrorKind::ResponseError,
                    "XREADGROUP Error",
                ))),
            )]);

        // Read pending messages:
        let result: RedisResult<(Vec<StreamId>, String)> = read_pending_messages(
            &mut conn,
            key,
            group,
            consumer,
            latest_pending_message_id,
            count,
        );

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_claim_pending_messages {
    use redis::{cmd, ErrorKind, RedisError, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_claim_pending_messages_with_zero_count() {
        // Define the key, group, consumer, min_idle_time, next_id_to_claim, and count:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let min_idle_time = 1000;
        let next_id_to_claim = "0-0";
        let count = 0;

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![]);

        // Claim pending messages:
        let result: RedisResult<(Vec<StreamId>, String)> = claim_pending_messages(
            &mut conn,
            key,
            group,
            consumer,
            min_idle_time,
            next_id_to_claim,
            count,
        );

        // Verify the result:
        assert!(result.is_ok());

        let (messages, next_id_to_claim): (Vec<StreamId>, String) = result.unwrap();
        assert!(messages.is_empty());
        assert!(next_id_to_claim.eq(BEGINNING_OF_TIME_ID));
    }

    #[test]
    fn test_claim_pending_messages_empty() {
        // Define the key, group, consumer, min_idle_time, next_id_to_claim, and count:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let min_idle_time = 1000;
        let next_id_to_claim = "0-0";
        let count = 2;

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XAUTOCLAIM")
                    .arg(key)
                    .arg(group)
                    .arg(consumer)
                    .arg(min_idle_time)
                    .arg(next_id_to_claim)
                    .arg(StreamAutoClaimOptions::default().count(count)),
                Ok(Value::Array(vec![
                    Value::SimpleString("0-0".to_string()),
                    Value::Array(vec![]),
                    Value::Array(vec![]),
                ])),
            )]);

        // Claim pending messages:
        let result: RedisResult<(Vec<StreamId>, String)> = claim_pending_messages(
            &mut conn,
            key,
            group,
            consumer,
            min_idle_time,
            next_id_to_claim,
            count,
        );

        // Verify the result:
        assert!(result.is_ok());

        let (messages, next_id_to_claim): (Vec<StreamId>, String) = result.unwrap();
        assert!(messages.len().eq(&0));
        assert!(next_id_to_claim.eq(BEGINNING_OF_TIME_ID));
    }

    #[test]
    fn test_claim_pending_messages_ok() {
        // Define the key, group, consumer, min_idle_time, next_id_to_claim, and count:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let min_idle_time = 1000;
        let next_id_to_claim = "0-0";
        let count = 2;

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XAUTOCLAIM")
                    .arg(key)
                    .arg(group)
                    .arg(consumer)
                    .arg(min_idle_time)
                    .arg(next_id_to_claim)
                    .arg(StreamAutoClaimOptions::default().count(count)),
                Ok(Value::Array(vec![
                    Value::SimpleString("1-0".to_string()),
                    Value::Array(vec![Value::Array(vec![
                        Value::SimpleString("1-0".to_string()),
                        Value::Array(vec![Value::SimpleString("code".to_string()), Value::Int(1)]),
                    ])]),
                    Value::Array(vec![]),
                ])),
            )]);

        // Claim pending messages:
        let result: RedisResult<(Vec<StreamId>, String)> = claim_pending_messages(
            &mut conn,
            key,
            group,
            consumer,
            min_idle_time,
            next_id_to_claim,
            count,
        );

        // Verify the result:
        assert!(result.is_ok());

        let (messages, next_id_to_claim): (Vec<StreamId>, String) = result.unwrap();
        assert!(messages.len().eq(&1));

        assert!(messages[0].id.eq("1-0"));
        assert!(messages[0].map.get("code").unwrap().eq(&Value::Int(1)));

        assert!(next_id_to_claim.eq("1-0"));
    }

    #[test]
    fn test_claim_pending_messages_error() {
        // Define the key, group, consumer, min_idle_time, next_id_to_claim, and count:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let min_idle_time = 1000;
        let next_id_to_claim = "0-0";
        let count = 2;

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XAUTOCLAIM")
                    .arg(key)
                    .arg(group)
                    .arg(consumer)
                    .arg(min_idle_time)
                    .arg(next_id_to_claim)
                    .arg(StreamAutoClaimOptions::default().count(count)),
                Err(RedisError::from((
                    ErrorKind::ResponseError,
                    "XAUTOCLAIM Error",
                ))),
            )]);

        // Claim pending messages:
        let result: RedisResult<(Vec<StreamId>, String)> = claim_pending_messages(
            &mut conn,
            key,
            group,
            consumer,
            min_idle_time,
            next_id_to_claim,
            count,
        );

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_pending_message_state {
    use redis::{cmd, ErrorKind, RedisError, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_pending_message_state_found() {
        // Define the key, group, consumer, and id:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let id = "1-0";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XPENDING")
                    .arg(key)
                    .arg(group)
                    .arg(&[id])
                    .arg(&[id])
                    .arg(1)
                    .arg(consumer),
                Ok(Value::Array(vec![Value::Array(vec![
                    Value::BulkString(b"1526984818136-0".to_vec()),
                    Value::BulkString(b"consumer-123".to_vec()),
                    Value::Int(196415),
                    Value::Int(5),
                ])])),
            )]);

        // Verify if the message is still in the consumer pending list:
        let result: RedisResult<Result<Option<(usize, usize)>, String>> =
            pending_message_state(&mut conn, key, group, consumer, id);

        // Verify the result:
        assert!(result.is_ok());

        let state: Option<(usize, usize)> = result.unwrap().unwrap();
        assert_eq!(state, Some((196415, 5)));
    }

    #[test]
    fn test_pending_message_state_not_found() {
        // Define the key, group, consumer, and id:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";
        let id = "1-0";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XPENDING")
                    .arg(key)
                    .arg(group)
                    .arg(&[id])
                    .arg(&[id])
                    .arg(1)
                    .arg(consumer),
                Ok(Value::Array(vec![])),
            )]);

        // Verify if the message is still in the consumer pending list:
        let result: RedisResult<Result<Option<(usize, usize)>, String>> =
            pending_message_state(&mut conn, key, group, consumer, id);

        // Verify the result:
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap(), None);
    }

    #[test]
    fn test_pending_message_state_error() {
        // Define the key, group, consumer, and id:
        let key = "my-key";
        let group = "my-group";
        let consumer = "my-consumer";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XPENDING")
                    .arg(key)
                    .arg(group)
                    .arg(&["1-0"])
                    .arg(&["1-0"])
                    .arg(1)
                    .arg(consumer),
                Err(RedisError::from((
                    ErrorKind::ResponseError,
                    "XPENDING Error",
                ))),
            )]);

        // Verify if the message is still in the consumer pending list:
        let result: RedisResult<Result<Option<(usize, usize)>, String>> =
            pending_message_state(&mut conn, key, group, consumer, "1-0");

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_ack {
    use redis::{cmd, ErrorKind, RedisError};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_ack_ok_true() {
        // Define the key, group, and id:
        let key = "my-key";
        let group = "my-group";
        let id: &str = "1-0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XACK").arg(key).arg(group).arg(&[id]),
            Ok(1),
        )]);

        // Acknowledge the message:
        let result: RedisResult<bool> = ack(&mut conn, key, group, id);

        // Verify the result:
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_ack_ok_false() {
        // Define the key, group, and id:
        let key = "my-key";
        let group = "my-group";
        let id = "1-0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XACK").arg(key).arg(group).arg(&[id]),
            Ok(0),
        )]);

        // Acknowledge the message:
        let result: RedisResult<bool> = ack(&mut conn, key, group, id);

        // Verify the result:
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_ack_error() {
        // Define the key, group, and id:
        let key = "my-key";
        let group = "my-group";
        let id = "1-0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XACK").arg(key).arg(group).arg(&[id]),
            Err(RedisError::from((ErrorKind::ResponseError, "XACK Error"))),
        )]);

        // Acknowledge the message:
        let result: RedisResult<bool> = ack(&mut conn, key, group, id);

        // Verify the result:
        assert!(result.is_err());
    }
}
