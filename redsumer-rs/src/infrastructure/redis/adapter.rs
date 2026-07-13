use redis::{Client, Commands};
use tracing::{debug, info};

use crate::application::ports::{MessageConsumer, MessageProducer};
use crate::domain::connection::ConnectionConfig;
use crate::domain::error::RedsumerError;
use crate::domain::message::{FieldValue, Message, MessageId};
use crate::domain::pending::PendingMessageInfo;
use crate::domain::result::RedsumerResult;
use crate::domain::stream::{ConsumerName, GroupName, StreamName};

use super::client::{ClientArgs, RedisClientBuilder};
use super::commands;
use super::mapping;

/// Connects to Redis Streams and implements [`MessageProducer`] and [`MessageConsumer`].
///
/// It sends produce and consume operations to the Redis server and converts responses into [`Message`], [`MessageId`] and related types.
#[derive(Debug, Clone)]
pub struct RedisStreamsAdapter<C = Client>
where
    C: Commands,
{
    /// Connection used to execute Redis commands.
    connection: C,
}

impl RedisStreamsAdapter<Client> {
    /// Build a new [`RedisStreamsAdapter`] instance from connection settings.
    ///
    /// Before creating the adapter, the following validations are performed:
    ///
    /// - If connection string is invalid, a [`RedsumerError`] is returned.
    /// - If connection to Redis server can not be established (verified with a `PING`), a [`RedsumerError`] is returned.
    ///
    /// # Arguments:
    /// - **connection**: Connection settings to build a new Redis client instance.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the new [`RedisStreamsAdapter`] instance. Otherwise, a [`RedsumerError`] is returned.
    pub fn new(connection: &ConnectionConfig) -> RedsumerResult<RedisStreamsAdapter<Client>> {
        let args: ClientArgs = connection.into();
        debug!("Creating a new Redis Streams adapter by: {:?}", args);

        let mut client: Client = args.build()?;
        client.ping::<String>().map_err(|e| {
            RedsumerError::ConnectionError(format!("Error pinging Redis server: {}", e))
        })?;

        info!("Redis Streams adapter created successfully and it is ready to be used");

        Ok(RedisStreamsAdapter { connection: client })
    }
}

impl<C> RedisStreamsAdapter<C>
where
    C: Commands,
{
    /// Build a new [`RedisStreamsAdapter`] instance from an existing connection. Useful for testing with mock connections.
    #[cfg(test)]
    pub fn from_connection(connection: C) -> RedisStreamsAdapter<C> {
        RedisStreamsAdapter { connection }
    }
}

impl<C> MessageProducer for RedisStreamsAdapter<C>
where
    C: Commands,
{
    fn produce(
        &mut self,
        stream: &StreamName,
        fields: &[(String, FieldValue)],
    ) -> RedsumerResult<MessageId> {
        let items: Vec<(String, Vec<u8>)> = fields
            .iter()
            .map(|(name, value)| (name.to_owned(), value.as_bytes().to_vec()))
            .collect();

        commands::produce(&mut self.connection, stream.as_str(), &items)
            .map(MessageId::from)
            .map_err(|e| RedsumerError::ProduceError(format!("Error producing message: {}", e)))
    }
}

impl<C> MessageConsumer for RedisStreamsAdapter<C>
where
    C: Commands,
{
    fn ensure_stream_exists(&mut self, stream: &StreamName) -> RedsumerResult<()> {
        match commands::stream_exists(&mut self.connection, stream.as_str()) {
            Ok(true) => Ok(()),
            Ok(false) => Err(RedsumerError::StreamNotFound(format!(
                "Stream {} does not exist",
                stream
            ))),
            Err(e) => Err(RedsumerError::ConnectionError(format!(
                "Error verifying if stream exists: {}",
                e
            ))),
        }
    }

    fn ensure_consumer_group(
        &mut self,
        stream: &StreamName,
        group: &GroupName,
        since_id: &MessageId,
    ) -> RedsumerResult<bool> {
        commands::create_consumer_group(
            &mut self.connection,
            stream.as_str(),
            group.as_str(),
            since_id.as_str(),
        )
        .map_err(|e| {
            RedsumerError::ConsumerGroupError(format!("Error creating consumer group: {}", e))
        })
    }

    fn read_new_messages(
        &mut self,
        stream: &StreamName,
        group: &GroupName,
        consumer: &ConsumerName,
        count: usize,
        block: usize,
    ) -> RedsumerResult<Vec<Message>> {
        commands::read_new_messages(
            &mut self.connection,
            stream.as_str(),
            group.as_str(),
            consumer.as_str(),
            count,
            block,
        )
        .map(|ids| mapping::messages_from_stream_ids(&ids))
        .map_err(|e| RedsumerError::ConsumeError(format!("Error reading new messages: {}", e)))
    }

    fn read_pending_messages(
        &mut self,
        stream: &StreamName,
        group: &GroupName,
        consumer: &ConsumerName,
        latest_pending_message_id: &MessageId,
        count: usize,
    ) -> RedsumerResult<(Vec<Message>, MessageId)> {
        commands::read_pending_messages(
            &mut self.connection,
            stream.as_str(),
            group.as_str(),
            consumer.as_str(),
            latest_pending_message_id.as_str(),
            count,
        )
        .map(|(ids, cursor)| {
            (
                mapping::messages_from_stream_ids(&ids),
                MessageId::from(cursor),
            )
        })
        .map_err(|e| RedsumerError::ConsumeError(format!("Error reading pending messages: {}", e)))
    }

    fn claim_pending_messages(
        &mut self,
        stream: &StreamName,
        group: &GroupName,
        consumer: &ConsumerName,
        min_idle_time: usize,
        next_id_to_claim: &MessageId,
        count: usize,
    ) -> RedsumerResult<(Vec<Message>, MessageId)> {
        commands::claim_pending_messages(
            &mut self.connection,
            stream.as_str(),
            group.as_str(),
            consumer.as_str(),
            min_idle_time,
            next_id_to_claim.as_str(),
            count,
        )
        .map(|(ids, cursor)| {
            (
                mapping::messages_from_stream_ids(&ids),
                MessageId::from(cursor),
            )
        })
        .map_err(|e| RedsumerError::ClaimError(format!("Error claiming pending messages: {}", e)))
    }

    fn get_pending_info(
        &mut self,
        stream: &StreamName,
        group: &GroupName,
        consumer: &ConsumerName,
        id: &MessageId,
    ) -> RedsumerResult<Option<PendingMessageInfo>> {
        let state: Result<Option<(usize, usize)>, String> = commands::pending_message_state(
            &mut self.connection,
            stream.as_str(),
            group.as_str(),
            consumer.as_str(),
            id.as_str(),
        )
        .map_err(|e| {
            RedsumerError::ConsumeError(format!("Error getting pending message info: {}", e))
        })?;

        match state {
            Ok(info) => Ok(info.map(|(last_delivered_ms, times_delivered)| {
                PendingMessageInfo::new(last_delivered_ms, times_delivered)
            })),
            Err(detail) => Err(RedsumerError::ConsumeError(detail)),
        }
    }

    fn ack(
        &mut self,
        stream: &StreamName,
        group: &GroupName,
        id: &MessageId,
    ) -> RedsumerResult<bool> {
        commands::ack(
            &mut self.connection,
            stream.as_str(),
            group.as_str(),
            id.as_str(),
        )
        .map_err(|e| RedsumerError::AckError(format!("Error acknowledging message: {}", e)))
    }
}

#[cfg(test)]
mod test_redis_streams_adapter {
    use redis::{cmd, ErrorKind, RedisError, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    fn stream() -> StreamName {
        StreamName::new("my-key").unwrap()
    }

    fn group() -> GroupName {
        GroupName::new("my-group").unwrap()
    }

    fn consumer() -> ConsumerName {
        ConsumerName::new("my-consumer").unwrap()
    }

    #[test]
    fn test_produce_ok() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
            cmd("XADD")
                .arg("my-key")
                .arg("*")
                .arg(&[("field".to_string(), b"value".to_vec())]),
            Ok(Value::SimpleString("1-0".to_string())),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        let fields: Vec<(String, FieldValue)> =
            vec![("field".to_string(), FieldValue::from("value"))];

        let result: RedsumerResult<MessageId> = adapter.produce(&stream(), &fields);

        assert_eq!(result.unwrap().as_str(), "1-0");
    }

    #[test]
    fn test_produce_error() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
            cmd("XADD")
                .arg("my-key")
                .arg("*")
                .arg(&[("field".to_string(), b"value".to_vec())]),
            Err(RedisError::from((ErrorKind::ResponseError, "XADD Error"))),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        let fields: Vec<(String, FieldValue)> =
            vec![("field".to_string(), FieldValue::from("value"))];

        let result: RedsumerResult<MessageId> = adapter.produce(&stream(), &fields);

        assert!(matches!(result, Err(RedsumerError::ProduceError(_))));
    }

    #[test]
    fn test_ensure_stream_exists_ok() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("EXISTS").arg("my-key"),
            Ok(1),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        assert!(adapter.ensure_stream_exists(&stream()).is_ok());
    }

    #[test]
    fn test_ensure_stream_exists_not_found() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("EXISTS").arg("my-key"),
            Ok(0),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        assert!(matches!(
            adapter.ensure_stream_exists(&stream()),
            Err(RedsumerError::StreamNotFound(_))
        ));
    }

    #[test]
    fn test_ensure_stream_exists_connection_error() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("EXISTS").arg("my-key"),
            Err(RedisError::from((ErrorKind::ResponseError, "EXISTS Error"))),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        assert!(matches!(
            adapter.ensure_stream_exists(&stream()),
            Err(RedsumerError::ConnectionError(_))
        ));
    }

    #[test]
    fn test_ensure_consumer_group_created() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, &str>(
            cmd("XGROUP")
                .arg("CREATE")
                .arg("my-key")
                .arg("my-group")
                .arg("0-0"),
            Ok("Ok"),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        let result: RedsumerResult<bool> =
            adapter.ensure_consumer_group(&stream(), &group(), &MessageId::beginning_of_time());

        assert!(result.unwrap());
    }

    #[test]
    fn test_ensure_consumer_group_error() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, &str>(
            cmd("XGROUP")
                .arg("CREATE")
                .arg("my-key")
                .arg("my-group")
                .arg("0-0"),
            Err(RedisError::from((ErrorKind::ResponseError, "XGROUP Error"))),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        let result: RedsumerResult<bool> =
            adapter.ensure_consumer_group(&stream(), &group(), &MessageId::beginning_of_time());

        assert!(matches!(result, Err(RedsumerError::ConsumerGroupError(_))));
    }

    #[test]
    fn test_read_new_messages_maps_to_domain() {
        use redis::streams::StreamReadOptions;

        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
            cmd("XREADGROUP")
                .arg(
                    StreamReadOptions::default()
                        .group("my-group", "my-consumer")
                        .count(2)
                        .block(1),
                )
                .arg("STREAMS")
                .arg(&["my-key"])
                .arg(&[">"]),
            Ok(Value::Array(vec![Value::Map(vec![(
                Value::SimpleString("my-key".to_string()),
                Value::Array(vec![Value::Map(vec![(
                    Value::SimpleString("1-0".to_string()),
                    Value::Array(vec![Value::SimpleString("code".to_string()), Value::Int(1)]),
                )])]),
            )])])),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        let messages: Vec<Message> = adapter
            .read_new_messages(&stream(), &group(), &consumer(), 2, 1)
            .unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].get_id().as_str(), "1-0");
        assert_eq!(
            messages[0]
                .get_field("code")
                .unwrap()
                .parse::<i32>()
                .unwrap(),
            1
        );
    }

    #[test]
    fn test_get_pending_info_found() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
            cmd("XPENDING")
                .arg("my-key")
                .arg("my-group")
                .arg(&["1-0"])
                .arg(&["1-0"])
                .arg(1)
                .arg("my-consumer"),
            Ok(Value::Array(vec![Value::Array(vec![
                Value::BulkString(b"1-0".to_vec()),
                Value::BulkString(b"my-consumer".to_vec()),
                Value::Int(196415),
                Value::Int(5),
            ])])),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        let info: Option<PendingMessageInfo> = adapter
            .get_pending_info(&stream(), &group(), &consumer(), &MessageId::new("1-0"))
            .unwrap();

        let info: PendingMessageInfo = info.unwrap();
        assert_eq!(info.get_last_delivered_milliseconds(), 196415);
        assert_eq!(info.get_total_times_delivered(), 5);
    }

    #[test]
    fn test_ack_ok() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XACK").arg("my-key").arg("my-group").arg(&["1-0"]),
            Ok(1),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        let result: RedsumerResult<bool> = adapter.ack(&stream(), &group(), &MessageId::new("1-0"));

        assert!(result.unwrap());
    }

    #[test]
    fn test_ack_error() {
        let conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XACK").arg("my-key").arg("my-group").arg(&["1-0"]),
            Err(RedisError::from((ErrorKind::ResponseError, "XACK Error"))),
        )]);

        let mut adapter: RedisStreamsAdapter<MockRedisConnection> =
            RedisStreamsAdapter::from_connection(conn);

        let result: RedsumerResult<bool> = adapter.ack(&stream(), &group(), &MessageId::new("1-0"));

        assert!(matches!(result, Err(RedsumerError::AckError(_))));
    }
}
