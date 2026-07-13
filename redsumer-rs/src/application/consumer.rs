use tracing::{debug, info};

use crate::application::ports::MessageConsumer;
use crate::domain::message::{Message, MessageId};
use crate::domain::pending::{LastDeliveredMilliseconds, PendingMessageInfo, TotalTimesDelivered};
use crate::domain::result::RedsumerResult;
use crate::domain::stream::{ConsumerName, GroupName, StreamName};

/// Options used to configure the consume operation when reading new messages from a stream.
#[derive(Debug, Clone)]
pub struct ReadNewMessagesOptions {
    /// The number of new messages to read from the stream.
    count: usize,

    /// The block time in `milliseconds` to wait for new messages to arrive in the stream.
    block: usize,
}

impl ReadNewMessagesOptions {
    /// Get the number of new messages to read from the stream.
    pub fn get_count(&self) -> usize {
        self.count
    }

    /// Get the block time to wait for new messages to arrive in the stream.
    pub fn get_block(&self) -> usize {
        self.block
    }

    /// Create a new instance of [`ReadNewMessagesOptions`].
    ///
    /// # Arguments:
    /// - **count**: The number of new messages to read from the stream.
    /// - **block**: The block time in milliseconds to wait for new messages to arrive in the stream.
    ///
    /// # Returns:
    /// A new instance of [`ReadNewMessagesOptions`] with the given count and block time.
    pub fn new(count: usize, block: usize) -> Self {
        ReadNewMessagesOptions { count, block }
    }
}

/// Options used to configure the consume operation when reading pending messages from a stream.
#[derive(Debug, Clone)]
pub struct ReadPendingMessagesOptions {
    /// The number of pending messages to read from the stream.
    count: usize,
}

impl ReadPendingMessagesOptions {
    /// Get the number of pending messages to read from the stream.
    pub fn get_count(&self) -> usize {
        self.count
    }

    /// Create a new instance of [`ReadPendingMessagesOptions`].
    ///
    /// # Arguments:
    /// - **count**: The number of pending messages to read from the stream.
    ///
    /// # Returns:
    /// A new instance of [`ReadPendingMessagesOptions`] with the given count.
    pub fn new(count: usize) -> Self {
        ReadPendingMessagesOptions { count }
    }
}

/// Options used to configure the consume operation when claiming messages from a stream.
#[derive(Debug, Clone)]
pub struct ClaimMessagesOptions {
    /// The number of messages to claim from the stream.
    count: usize,

    /// The min idle time in `milliseconds` to claim the messages.
    min_idle_time: usize,
}

impl ClaimMessagesOptions {
    /// Get the number of messages to claim from the stream.
    pub fn get_count(&self) -> usize {
        self.count
    }

    /// Get the min idle time to claim the messages.
    pub fn get_min_idle_time(&self) -> usize {
        self.min_idle_time
    }

    /// Create a new instance of [`ClaimMessagesOptions`].
    ///
    /// # Arguments:
    /// - **count**: The number of messages to claim from the stream.
    /// - **min_idle_time**: The min idle time in milliseconds to claim the messages.
    ///
    /// # Returns:
    /// A new instance of [`ClaimMessagesOptions`] with the given count and min idle time.
    pub fn new(count: usize, min_idle_time: usize) -> Self {
        ClaimMessagesOptions {
            count,
            min_idle_time,
        }
    }
}

/// Define the configuration parameters to create a consumer instance.
///
/// This configuration is immutable: runtime cursors are kept in the consumer session state, not here.
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// Stream name where messages will be consumed.
    stream_name: String,

    /// Group name where the consumer is registered.
    group_name: String,

    /// Consumer name within the specified consumers group.
    consumer_name: String,

    /// Options to configure the read new messages operation.
    read_new_messages_options: ReadNewMessagesOptions,

    /// Options to configure the read pending messages operation.
    read_pending_messages_options: ReadPendingMessagesOptions,

    /// Options to configure the claim messages operation.
    claim_messages_options: ClaimMessagesOptions,
}

impl ConsumerConfig {
    /// Get **stream name**.
    pub fn get_stream_name(&self) -> &str {
        &self.stream_name
    }

    /// Get **group name**.
    pub fn get_group_name(&self) -> &str {
        &self.group_name
    }

    /// Get **consumer name**.
    pub fn get_consumer_name(&self) -> &str {
        &self.consumer_name
    }

    /// Get **read new messages options**.
    pub fn get_read_new_messages_options(&self) -> &ReadNewMessagesOptions {
        &self.read_new_messages_options
    }

    /// Get **read pending messages options**.
    pub fn get_read_pending_messages_options(&self) -> &ReadPendingMessagesOptions {
        &self.read_pending_messages_options
    }

    /// Get **claim messages options**.
    pub fn get_claim_messages_options(&self) -> &ClaimMessagesOptions {
        &self.claim_messages_options
    }

    /// Create a new [`ConsumerConfig`] instance.
    ///
    /// # Arguments:
    /// - **stream_name**: The name of the stream where messages will be consumed.
    /// - **group_name**: Consumers group name.
    /// - **consumer_name**: Represents the consumer name within the specified consumers group, which must be ensured to be unique. In a microservices architecture, for example, it is recommended to use the pod name.
    /// - **read_new_messages_options**: Options to configure the read new messages operation.
    /// - **read_pending_messages_options**: Options to configure the read pending messages operation.
    /// - **claim_messages_options**: Options to configure the claim messages operation.
    ///
    /// # Returns:
    /// A new [`ConsumerConfig`] instance.
    pub fn new(
        stream_name: &str,
        group_name: &str,
        consumer_name: &str,
        read_new_messages_options: ReadNewMessagesOptions,
        read_pending_messages_options: ReadPendingMessagesOptions,
        claim_messages_options: ClaimMessagesOptions,
    ) -> Self {
        ConsumerConfig {
            stream_name: stream_name.to_owned(),
            group_name: group_name.to_owned(),
            consumer_name: consumer_name.to_owned(),
            read_new_messages_options,
            read_pending_messages_options,
            claim_messages_options,
        }
    }
}

/// Define the kind of messages that were consumed by a specific consumer.
#[derive(Debug, Clone)]
enum MessagesKind {
    /// The messages were obtained from the new messages list and have not been delivered before to any consumer.
    New,

    /// The messages were read from the consumer pending list. They were delivered to a consumer before, but they were not acked yet and they were not claimed by another consumer.
    Pending,

    /// The messages were claimed by another consumer and they were not acked yet.
    Claimed,

    /// Messages were not obtained from stream. It means that there are no new, pending or claimed messages to be processed by a consumer in the specified group.
    NotFound,
}

impl MessagesKind {
    /// Check if the messages are new.
    fn are_new(&self) -> bool {
        matches!(self, MessagesKind::New)
    }

    /// Check if the messages are pending.
    fn are_pending(&self) -> bool {
        matches!(self, MessagesKind::Pending)
    }

    /// Check if the messages were claimed.
    fn were_claimed(&self) -> bool {
        matches!(self, MessagesKind::Claimed)
    }

    /// Check if the messages were not found.
    fn not_found(&self) -> bool {
        matches!(self, MessagesKind::NotFound)
    }
}

/// A reply to consume messages from a stream. It contains a list of messages and the kind of messages.
#[derive(Debug, Clone)]
pub struct ConsumeMessagesReply {
    /// A list of consumed messages.
    messages: Vec<Message>,

    /// The kind of messages.
    kind: MessagesKind,
}

impl ConsumeMessagesReply {
    /// Get **messages**.
    pub fn get_messages(&self) -> &Vec<Message> {
        &self.messages
    }

    /// Verify if the messages are new.
    pub fn are_new(&self) -> bool {
        self.kind.are_new()
    }

    /// Verify if the messages are pending in the consumer pending list.
    pub fn are_pending(&self) -> bool {
        self.kind.are_pending()
    }

    /// Verify if the messages were claimed by another consumer.
    pub fn were_claimed(&self) -> bool {
        self.kind.were_claimed()
    }

    /// Verify if the messages were not found.
    pub fn not_found(&self) -> bool {
        self.kind.not_found()
    }
}

/// Convert a tuple into a [`ConsumeMessagesReply`] instance.
impl From<(Vec<Message>, MessagesKind)> for ConsumeMessagesReply {
    fn from((messages, kind): (Vec<Message>, MessagesKind)) -> Self {
        ConsumeMessagesReply { messages, kind }
    }
}

/// A reply to verify if a specific message is still in consumer pending list.
#[derive(Debug, Clone)]
pub struct IsStillMineReply {
    /// Pending information about the message, if it is still in the consumer pending list.
    pending_info: Option<PendingMessageInfo>,
}

impl IsStillMineReply {
    /// Verify if the message still belongs to the consumer.
    pub fn belongs_to_me(&self) -> bool {
        self.pending_info.is_some()
    }

    /// Get the pending information about the message, if it is still in the consumer pending list.
    pub fn get_pending_info(&self) -> Option<&PendingMessageInfo> {
        self.pending_info.as_ref()
    }

    /// Get **last delivered milliseconds**.
    pub fn get_last_delivered_milliseconds(&self) -> Option<LastDeliveredMilliseconds> {
        self.pending_info
            .as_ref()
            .map(|i| i.get_last_delivered_milliseconds())
    }

    /// Get **total times delivered**.
    pub fn get_total_times_delivered(&self) -> Option<TotalTimesDelivered> {
        self.pending_info
            .as_ref()
            .map(|i| i.get_total_times_delivered())
    }
}

/// Convert an optional [`PendingMessageInfo`] into a [`IsStillMineReply`] instance.
impl From<Option<PendingMessageInfo>> for IsStillMineReply {
    fn from(pending_info: Option<PendingMessageInfo>) -> Self {
        IsStillMineReply { pending_info }
    }
}

/// A reply to ack a specific message.
#[derive(Debug, Clone)]
pub struct AckMessageReply {
    /// A boolean value indicating if the message is acked.
    was_acked: bool,
}

impl AckMessageReply {
    /// Get **was acked**. If the message was not acked, it is recommended to verify if another consumer has claimed the message before trying to process it again.
    pub fn was_acked(&self) -> bool {
        self.was_acked
    }
}

/// Convert a boolean value into a [`AckMessageReply`] instance.
impl From<bool> for AckMessageReply {
    fn from(was_acked: bool) -> Self {
        AckMessageReply { was_acked }
    }
}

/// Runtime state of a consumer session. It holds the cursors used by the consume operation, keeping the configuration immutable.
#[derive(Debug, Clone)]
struct ConsumerSessionState {
    /// The latest pending message ID to start reading from.
    latest_pending_message_id: MessageId,

    /// The next ID to start claiming from.
    next_id_to_claim: MessageId,
}

impl ConsumerSessionState {
    /// Create a new session state with both cursors at the beginning of time.
    fn new() -> Self {
        ConsumerSessionState {
            latest_pending_message_id: MessageId::beginning_of_time(),
            next_id_to_claim: MessageId::beginning_of_time(),
        }
    }
}

/// A consumer of stream messages. It can read new messages, pending messages or claim messages from other consumers according to their min idle time, through any [`MessageConsumer`] implementation received in the constructor.
#[derive(Debug, Clone)]
pub struct ConsumerService<C>
where
    C: MessageConsumer,
{
    /// Implementation used to consume messages.
    port: C,

    /// Consumer configuration parameters.
    config: ConsumerConfig,

    /// Stream name where messages will be consumed.
    stream: StreamName,

    /// Consumers group name.
    group: GroupName,

    /// Consumer name within the consumers group.
    consumer: ConsumerName,

    /// Runtime session state (cursors).
    session: ConsumerSessionState,
}

impl<C> ConsumerService<C>
where
    C: MessageConsumer,
{
    /// Get *config*.
    pub fn get_config(&self) -> &ConsumerConfig {
        &self.config
    }

    /// Build a new [`ConsumerService`] instance from a [`MessageConsumer`] implementation.
    ///
    /// Before creating a new consumer, the following validations are performed:
    ///
    /// - The stream, group and consumer names from the configuration are validated.
    /// - If the stream does not exist, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned: The stream must exist before creating a new consumer.
    /// - If the consumers group does not exist, it is created based on the *stream_name*, *group_name* and the given *initial_stream_id*. If an error occurs during the creation process, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    ///
    /// # Arguments:
    /// - **port**: The [`MessageConsumer`] implementation used to consume messages.
    /// - **config**: Consumer configuration parameters.
    /// - **initial_stream_id**: The ID of the message to start consuming from. If `None`, the beginning of time is used.
    ///
    /// # Returns:
    /// - A [`RedsumerResult`] containing a [`ConsumerService`] instance. Otherwise, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn from_port(
        mut port: C,
        config: ConsumerConfig,
        initial_stream_id: Option<MessageId>,
    ) -> RedsumerResult<Self> {
        debug!("Creating a new consumer instance by: {:?}", config);

        let stream: StreamName = StreamName::new(config.get_stream_name())?;
        let group: GroupName = GroupName::new(config.get_group_name())?;
        let consumer: ConsumerName = ConsumerName::new(config.get_consumer_name())?;

        port.ensure_stream_exists(&stream)?;
        port.ensure_consumer_group(
            &stream,
            &group,
            &initial_stream_id.unwrap_or_else(MessageId::beginning_of_time),
        )?;

        info!("Consumer was created successfully and it is ready to be used");

        Ok(Self {
            port,
            config,
            stream,
            group,
            consumer,
            session: ConsumerSessionState::new(),
        })
    }

    /// Consume messages from stream according to the following steps:
    ///
    /// 1. Consumer tries to get new messages. If new messages are found, they are returned as a result.
    /// 2. If new messages are not found, consumer tries to get pending messages. If pending messages are found, they are returned as a result.
    /// 3. If pending messages are not found, consumer tries to claim messages from other consumers according to *min_idle_time*. If claimed messages are found, they are returned as a result.
    /// 4. If new, pending or claimed messages are not found, an empty list is returned as a result.
    ///
    /// # Arguments:
    /// *No arguments*
    ///
    /// # Returns:
    /// - A [`RedsumerResult`] containing a [`ConsumeMessagesReply`] if new, pending or claimed messages are found, otherwise an empty reply is returned. If an error occurs, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn consume(&mut self) -> RedsumerResult<ConsumeMessagesReply> {
        debug!("Consuming messages from stream {}", self.stream);

        debug!(
            "Processing new messages by: {:?}",
            self.config.get_read_new_messages_options()
        );

        let new_messages: Vec<Message> = self.port.read_new_messages(
            &self.stream,
            &self.group,
            &self.consumer,
            self.config.get_read_new_messages_options().get_count(),
            self.config.get_read_new_messages_options().get_block(),
        )?;
        if !new_messages.is_empty() {
            debug!("Total new messages found: {}", new_messages.len());
            return Ok((new_messages, MessagesKind::New).into());
        }

        debug!(
            "Processing pending messages by: {:?}",
            self.config.get_read_pending_messages_options()
        );

        let (pending_messages, latest_pending_message_id): (Vec<Message>, MessageId) =
            self.port.read_pending_messages(
                &self.stream,
                &self.group,
                &self.consumer,
                &self.session.latest_pending_message_id,
                self.config.get_read_pending_messages_options().get_count(),
            )?;

        debug!("Updating latest pending message ID to: {latest_pending_message_id}");

        self.session.latest_pending_message_id = latest_pending_message_id;
        if !pending_messages.is_empty() {
            debug!("Total pending messages found: {}", pending_messages.len());
            return Ok((pending_messages, MessagesKind::Pending).into());
        }

        debug!(
            "Processing claimed messages by: {:?}",
            self.config.get_claim_messages_options()
        );

        let (claimed_messages, next_id_to_claim): (Vec<Message>, MessageId) =
            self.port.claim_pending_messages(
                &self.stream,
                &self.group,
                &self.consumer,
                self.config.get_claim_messages_options().get_min_idle_time(),
                &self.session.next_id_to_claim,
                self.config.get_claim_messages_options().get_count(),
            )?;

        debug!("Updating next ID to claim to: {next_id_to_claim}");

        self.session.next_id_to_claim = next_id_to_claim;
        if !claimed_messages.is_empty() {
            debug!("Total claimed messages found: {}", claimed_messages.len());
            return Ok((claimed_messages, MessagesKind::Claimed).into());
        }

        debug!("No messages found");

        Ok((Vec::new(), MessagesKind::NotFound).into())
    }

    /// Verify if a specific message by *id* is still in consumer pending list.
    ///
    /// If the message is not still in consumer pending list, it is recommended to verify if another consumer has claimed the message before trying to process it again.
    ///
    /// # Arguments:
    /// - **id**: Stream message id.
    ///
    /// # Returns:
    /// - A [`RedsumerResult`] containing a [`IsStillMineReply`] if successful. If an error occurs, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn is_still_mine(&mut self, id: &MessageId) -> RedsumerResult<IsStillMineReply> {
        self.port
            .get_pending_info(&self.stream, &self.group, &self.consumer, id)
            .map(IsStillMineReply::from)
    }

    /// Ack a message by *id*.
    ///
    /// If the message is acked, it is removed from the consumer pending list. Otherwise, it is recommended to verify if another consumer has claimed the message before trying to process it again.
    ///
    /// # Arguments:
    /// - **id**: Stream message id.
    ///
    /// # Returns:
    /// - A [`RedsumerResult`] containing a [`AckMessageReply`] if successful. If an error occurs, a [`RedsumerError`](crate::domain::error::RedsumerError) is returned.
    pub fn ack(&mut self, id: &MessageId) -> RedsumerResult<AckMessageReply> {
        self.port
            .ack(&self.stream, &self.group, id)
            .map(AckMessageReply::from)
    }
}

#[cfg(test)]
mod test_read_new_messages_options {
    use super::*;

    #[test]
    fn test_new_read_new_messages_options() {
        // Define count and block:
        let count: usize = 10;
        let block: usize = 3;

        // Create new ReadNewMessagesOptions instance:
        let options: ReadNewMessagesOptions = ReadNewMessagesOptions::new(count, block);

        // Verify the result:
        assert_eq!(options.get_count(), count);
        assert_eq!(options.get_block(), block);
    }
}

#[cfg(test)]
mod test_read_pending_messages_options {
    use super::*;

    #[test]
    fn test_new_read_pending_messages_options() {
        // Define count:
        let count: usize = 10;

        // Create new ReadPendingMessagesOptions instance:
        let options: ReadPendingMessagesOptions = ReadPendingMessagesOptions::new(count);

        // Verify the result:
        assert_eq!(options.get_count(), count);
    }
}

#[cfg(test)]
mod test_claim_messages_options {
    use super::*;

    #[test]
    fn test_new_claim_messages_options() {
        // Define count and min idle time:
        let count: usize = 10;
        let min_idle_time: usize = 1000;

        // Create new ClaimMessagesOptions instance:
        let options: ClaimMessagesOptions = ClaimMessagesOptions::new(count, min_idle_time);

        // Verify the result:
        assert_eq!(options.get_count(), count);
        assert_eq!(options.get_min_idle_time(), min_idle_time);
    }
}

#[cfg(test)]
mod test_consumer_config {
    use super::*;

    #[test]
    fn test_new_consumer_config() {
        // Define stream name, group name and consumer name:
        let stream_name: &str = "stream";
        let group_name: &str = "group";
        let consumer_name: &str = "consumer";

        // Define count, block and min idle time:
        let count: usize = 10;
        let block: usize = 3;
        let min_idle_time: usize = 1000;

        // Create new ConsumerConfig instance:
        let config: ConsumerConfig = ConsumerConfig::new(
            stream_name,
            group_name,
            consumer_name,
            ReadNewMessagesOptions::new(count, block),
            ReadPendingMessagesOptions::new(count),
            ClaimMessagesOptions::new(count, min_idle_time),
        );

        // Verify the result:
        assert_eq!(config.get_stream_name(), stream_name);
        assert_eq!(config.get_group_name(), group_name);
        assert_eq!(config.get_consumer_name(), consumer_name);

        assert_eq!(config.get_read_new_messages_options().get_count(), count);
        assert_eq!(config.get_read_new_messages_options().get_block(), block);

        assert_eq!(
            config.get_read_pending_messages_options().get_count(),
            count
        );

        assert_eq!(config.get_claim_messages_options().get_count(), count);
        assert_eq!(
            config.get_claim_messages_options().get_min_idle_time(),
            min_idle_time
        );
    }
}

#[cfg(test)]
mod test_messages_kind {
    use super::MessagesKind;

    #[test]
    fn test_messages_kind() {
        // Create new MessagesKind instances:
        let new_messages: MessagesKind = MessagesKind::New;
        let pending_messages: MessagesKind = MessagesKind::Pending;
        let claimed_messages: MessagesKind = MessagesKind::Claimed;
        let not_found_messages: MessagesKind = MessagesKind::NotFound;

        // Verify the result:
        assert!(new_messages.are_new());
        assert!(!new_messages.are_pending());
        assert!(!new_messages.were_claimed());
        assert!(!new_messages.not_found());

        assert!(!pending_messages.are_new());
        assert!(pending_messages.are_pending());
        assert!(!pending_messages.were_claimed());
        assert!(!pending_messages.not_found());

        assert!(!claimed_messages.are_new());
        assert!(!claimed_messages.are_pending());
        assert!(claimed_messages.were_claimed());
        assert!(!claimed_messages.not_found());

        assert!(!not_found_messages.are_new());
        assert!(!not_found_messages.are_pending());
        assert!(!not_found_messages.were_claimed());
        assert!(not_found_messages.not_found());
    }
}

#[cfg(test)]
mod test_consume_messages_reply {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_consume_messages_reply() {
        // Define messages and kind:
        let messages: Vec<Message> = vec![Message::new(MessageId::new("1-0"), BTreeMap::new())];
        let kind: MessagesKind = MessagesKind::New;

        // Create new ConsumeMessagesReply instance:
        let reply: ConsumeMessagesReply = ConsumeMessagesReply::from((messages, kind));

        // Verify the result:
        assert!(reply.get_messages().len().eq(&1));
        assert!(reply.are_new());
        assert!(!reply.are_pending());
        assert!(!reply.were_claimed());
        assert!(!reply.not_found());
    }
}

#[cfg(test)]
mod test_is_still_mine_reply {
    use super::*;

    #[test]
    fn test_is_still_mine_reply_with_pending_info() {
        // Define last delivered milliseconds and total times delivered:
        let last_delivered_milliseconds: LastDeliveredMilliseconds = 1000;
        let total_times_delivered: TotalTimesDelivered = 787;

        // Create new IsStillMineReply instance:
        let reply: IsStillMineReply = IsStillMineReply::from(Some(PendingMessageInfo::new(
            last_delivered_milliseconds,
            total_times_delivered,
        )));

        // Verify the result:
        assert!(reply.belongs_to_me());
        assert!(reply.get_pending_info().is_some());

        assert_eq!(
            reply.get_last_delivered_milliseconds(),
            Some(last_delivered_milliseconds)
        );
        assert_eq!(
            reply.get_total_times_delivered(),
            Some(total_times_delivered)
        );
    }

    #[test]
    fn test_is_still_mine_reply_without_pending_info() {
        // Create new IsStillMineReply instance:
        let reply: IsStillMineReply = IsStillMineReply::from(None);

        // Verify the result:
        assert!(!reply.belongs_to_me());
        assert!(reply.get_pending_info().is_none());
        assert!(reply.get_last_delivered_milliseconds().is_none());
        assert!(reply.get_total_times_delivered().is_none());
    }
}

#[cfg(test)]
mod test_ack_message_reply {
    use super::*;

    #[test]
    fn test_ack_message_reply() {
        // Define was acked:
        let was_acked: bool = true;

        // Create new AckMessageReply instance:
        let reply: AckMessageReply = AckMessageReply::from(was_acked);

        // Verify the result:
        assert!(reply.was_acked());
    }
}

#[cfg(test)]
mod test_consumer_service {
    use std::collections::BTreeMap;

    use crate::domain::error::RedsumerError;

    use super::*;

    /// Build a domain message with a single field.
    fn build_message(id: &str) -> Message {
        let mut fields: BTreeMap<String, crate::domain::message::FieldValue> = BTreeMap::new();
        fields.insert(
            "code".to_string(),
            crate::domain::message::FieldValue::from(1),
        );

        Message::new(MessageId::new(id), fields)
    }

    /// In-memory fake implementation of the [`MessageConsumer`] port.
    struct FakeConsumerPort {
        stream_exists: bool,
        new_messages: Vec<Message>,
        pending_messages: Vec<Message>,
        claimed_messages: Vec<Message>,
        pending_info: Option<PendingMessageInfo>,
        ack_result: bool,
        group_created: bool,
    }

    impl FakeConsumerPort {
        fn new() -> Self {
            FakeConsumerPort {
                stream_exists: true,
                new_messages: Vec::new(),
                pending_messages: Vec::new(),
                claimed_messages: Vec::new(),
                pending_info: None,
                ack_result: false,
                group_created: false,
            }
        }
    }

    impl MessageConsumer for FakeConsumerPort {
        fn ensure_stream_exists(&mut self, stream: &StreamName) -> RedsumerResult<()> {
            match self.stream_exists {
                true => Ok(()),
                false => Err(RedsumerError::StreamNotFound(format!(
                    "Stream {stream} does not exist"
                ))),
            }
        }

        fn ensure_consumer_group(
            &mut self,
            _stream: &StreamName,
            _group: &GroupName,
            _since_id: &MessageId,
        ) -> RedsumerResult<bool> {
            self.group_created = true;
            Ok(true)
        }

        fn read_new_messages(
            &mut self,
            _stream: &StreamName,
            _group: &GroupName,
            _consumer: &ConsumerName,
            _count: usize,
            _block: usize,
        ) -> RedsumerResult<Vec<Message>> {
            Ok(self.new_messages.to_owned())
        }

        fn read_pending_messages(
            &mut self,
            _stream: &StreamName,
            _group: &GroupName,
            _consumer: &ConsumerName,
            _latest_pending_message_id: &MessageId,
            _count: usize,
        ) -> RedsumerResult<(Vec<Message>, MessageId)> {
            let next_cursor: MessageId = match self.pending_messages.last() {
                Some(m) => m.get_id().to_owned(),
                None => MessageId::beginning_of_time(),
            };

            Ok((self.pending_messages.to_owned(), next_cursor))
        }

        fn claim_pending_messages(
            &mut self,
            _stream: &StreamName,
            _group: &GroupName,
            _consumer: &ConsumerName,
            _min_idle_time: usize,
            _next_id_to_claim: &MessageId,
            _count: usize,
        ) -> RedsumerResult<(Vec<Message>, MessageId)> {
            let next_cursor: MessageId = match self.claimed_messages.last() {
                Some(m) => m.get_id().to_owned(),
                None => MessageId::beginning_of_time(),
            };

            Ok((self.claimed_messages.to_owned(), next_cursor))
        }

        fn get_pending_info(
            &mut self,
            _stream: &StreamName,
            _group: &GroupName,
            _consumer: &ConsumerName,
            _id: &MessageId,
        ) -> RedsumerResult<Option<PendingMessageInfo>> {
            Ok(self.pending_info.to_owned())
        }

        fn ack(
            &mut self,
            _stream: &StreamName,
            _group: &GroupName,
            _id: &MessageId,
        ) -> RedsumerResult<bool> {
            Ok(self.ack_result)
        }
    }

    fn build_config() -> ConsumerConfig {
        ConsumerConfig::new(
            "stream",
            "group",
            "consumer",
            ReadNewMessagesOptions::new(10, 5),
            ReadPendingMessagesOptions::new(10),
            ClaimMessagesOptions::new(10, 1000),
        )
    }

    #[test]
    fn test_from_port_verifies_stream_and_creates_group() {
        let port: FakeConsumerPort = FakeConsumerPort::new();

        let consumer: ConsumerService<FakeConsumerPort> =
            ConsumerService::from_port(port, build_config(), None).unwrap();

        assert!(consumer.port.group_created);
        assert_eq!(consumer.get_config().get_stream_name(), "stream");
    }

    #[test]
    fn test_from_port_fails_if_stream_does_not_exist() {
        let mut port: FakeConsumerPort = FakeConsumerPort::new();
        port.stream_exists = false;

        let result: RedsumerResult<ConsumerService<FakeConsumerPort>> =
            ConsumerService::from_port(port, build_config(), None);

        assert!(matches!(result, Err(RedsumerError::StreamNotFound(_))));
    }

    #[test]
    fn test_from_port_fails_with_invalid_config() {
        let port: FakeConsumerPort = FakeConsumerPort::new();
        let config: ConsumerConfig = ConsumerConfig::new(
            "",
            "group",
            "consumer",
            ReadNewMessagesOptions::new(10, 5),
            ReadPendingMessagesOptions::new(10),
            ClaimMessagesOptions::new(10, 1000),
        );

        let result: RedsumerResult<ConsumerService<FakeConsumerPort>> =
            ConsumerService::from_port(port, config, None);

        assert!(matches!(result, Err(RedsumerError::ConfigurationError(_))));
    }

    #[test]
    fn test_consume_returns_new_messages_first() {
        let mut port: FakeConsumerPort = FakeConsumerPort::new();
        port.new_messages = vec![build_message("1-0")];
        port.pending_messages = vec![build_message("2-0")];

        let mut consumer: ConsumerService<FakeConsumerPort> =
            ConsumerService::from_port(port, build_config(), None).unwrap();

        let reply: ConsumeMessagesReply = consumer.consume().unwrap();

        assert!(reply.are_new());
        assert_eq!(reply.get_messages().len(), 1);
        assert_eq!(reply.get_messages()[0].get_id().as_str(), "1-0");
    }

    #[test]
    fn test_consume_returns_pending_messages_and_updates_cursor() {
        let mut port: FakeConsumerPort = FakeConsumerPort::new();
        port.pending_messages = vec![build_message("2-0"), build_message("3-0")];

        let mut consumer: ConsumerService<FakeConsumerPort> =
            ConsumerService::from_port(port, build_config(), None).unwrap();

        let reply: ConsumeMessagesReply = consumer.consume().unwrap();

        assert!(reply.are_pending());
        assert_eq!(reply.get_messages().len(), 2);
        assert_eq!(consumer.session.latest_pending_message_id.as_str(), "3-0");
    }

    #[test]
    fn test_consume_returns_claimed_messages_and_updates_cursor() {
        let mut port: FakeConsumerPort = FakeConsumerPort::new();
        port.claimed_messages = vec![build_message("4-0")];

        let mut consumer: ConsumerService<FakeConsumerPort> =
            ConsumerService::from_port(port, build_config(), None).unwrap();

        let reply: ConsumeMessagesReply = consumer.consume().unwrap();

        assert!(reply.were_claimed());
        assert_eq!(reply.get_messages().len(), 1);
        assert_eq!(consumer.session.next_id_to_claim.as_str(), "4-0");
    }

    #[test]
    fn test_consume_returns_not_found_when_no_messages() {
        let port: FakeConsumerPort = FakeConsumerPort::new();

        let mut consumer: ConsumerService<FakeConsumerPort> =
            ConsumerService::from_port(port, build_config(), None).unwrap();

        let reply: ConsumeMessagesReply = consumer.consume().unwrap();

        assert!(reply.not_found());
        assert!(reply.get_messages().is_empty());
    }

    #[test]
    fn test_is_still_mine() {
        let mut port: FakeConsumerPort = FakeConsumerPort::new();
        port.pending_info = Some(PendingMessageInfo::new(500, 3));

        let mut consumer: ConsumerService<FakeConsumerPort> =
            ConsumerService::from_port(port, build_config(), None).unwrap();

        let reply: IsStillMineReply = consumer.is_still_mine(&MessageId::new("1-0")).unwrap();

        assert!(reply.belongs_to_me());
        assert_eq!(reply.get_last_delivered_milliseconds(), Some(500));
        assert_eq!(reply.get_total_times_delivered(), Some(3));
    }

    #[test]
    fn test_ack() {
        let mut port: FakeConsumerPort = FakeConsumerPort::new();
        port.ack_result = true;

        let mut consumer: ConsumerService<FakeConsumerPort> =
            ConsumerService::from_port(port, build_config(), None).unwrap();

        let reply: AckMessageReply = consumer.ack(&MessageId::new("1-0")).unwrap();

        assert!(reply.was_acked());
    }
}
