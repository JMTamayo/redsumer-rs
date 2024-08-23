use redis::{streams::StreamId, ToRedisArgs};

use crate::results::Id;

/// Options used to configure the consume operation when reading new messages from a Redis stream.
#[derive(Debug, Clone)]
pub struct ReadNewMessagesOptions {
    /// The number of new messages to read from the stream.
    count: usize,

    /// The block time [seconds] to wait for new messages to arrive in the stream.
    block: usize,
}

impl ReadNewMessagesOptions {
    /// Get the number of new messages to read from the stream.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The number of new messages to read from the stream.
    pub fn get_count(&self) -> usize {
        self.count
    }

    /// Get the block time to wait for new messages to arrive in the stream.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The block time [seconds] to wait for new messages to arrive in the stream.
    pub fn get_block(&self) -> usize {
        self.block
    }

    /// Create a new instance of [`ReadNewMessagesOptions`].
    ///
    /// # Arguments:
    /// - **count**: The number of new messages to read from the stream.
    /// - **block**: The block time [seconds] to wait for new messages to arrive in the stream.
    ///
    /// # Returns:
    /// A new instance of [`ReadNewMessagesOptions`] with the given count and block time.
    pub fn new(count: usize, block: usize) -> Self {
        ReadNewMessagesOptions { count, block }
    }
}

/// Options used to configure the consume operation when reading pending messages from a Redis stream.
#[derive(Debug, Clone)]
pub struct ReadPendingMessagesOptions<ID>
where
    ID: ToRedisArgs,
{
    /// The number of pending messages to read from the stream.
    count: usize,

    /// The latest pending message ID to start reading from.
    latest_pending_message_id: ID,
}

impl<ID> ReadPendingMessagesOptions<ID>
where
    ID: ToRedisArgs,
{
    /// Get the number of pending messages to read from the stream.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The number of pending messages to read from the stream.
    pub fn get_count(&self) -> usize {
        self.count
    }

    /// Get the latest pending message ID to start reading from.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The latest pending message ID to start reading from.
    pub fn get_latest_pending_message_id(&self) -> &ID {
        &self.latest_pending_message_id
    }

    /// Create a new instance of [`ReadPendingMessagesOptions`].
    ///
    /// # Arguments:
    /// - **count**: The number of pending messages to read from the stream.
    /// - **latest_pending_message_id**: The latest pending message [`Id`] to start reading from.
    ///
    /// # Returns:
    /// A new instance of [`ReadPendingMessagesOptions`] with the given count and latest pending message ID.
    pub fn new(count: usize, latest_pending_message_id: ID) -> Self {
        ReadPendingMessagesOptions {
            count,
            latest_pending_message_id,
        }
    }
}

/// Options used to configure the consume operation when claiming messages from a Redis stream.
#[derive(Debug, Clone)]
pub struct ClaimMessagesOptions<ID>
where
    ID: ToRedisArgs,
{
    /// The number of messages to claim from the stream.
    count: usize,

    /// The min idle time [milliseconds] to claim the messages.
    min_idle_time: usize,

    /// The latest pending message ID to start claiming from.
    next_id_to_claim: ID,
}

impl<ID> ClaimMessagesOptions<ID>
where
    ID: ToRedisArgs,
{
    /// Get the number of messages to claim from the stream.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The number of messages to claim from the stream.
    pub fn get_count(&self) -> usize {
        self.count
    }

    /// Get the min idle time to claim the messages.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The min idle time [milliseconds] to claim the messages.
    pub fn get_min_idle_time(&self) -> usize {
        self.min_idle_time
    }

    /// Get the latest pending message ID to start claiming from.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The latest pending message ID to start claiming from.
    pub fn get_next_id_to_claim(&self) -> &ID {
        &self.next_id_to_claim
    }

    /// Create a new instance of [`ClaimMessagesOptions`].
    ///
    /// # Arguments:
    /// - **count**: The number of messages to claim from the stream.
    /// - **min_idle_time**: The min idle time [milliseconds] to claim the messages.
    /// - **next_id_to_claim**: The latest pending message [`Id`] to start claiming from.
    ///
    /// # Returns:
    /// A new instance of [`ClaimMessagesOptions`] with the given count, min idle time and latest pending message ID.
    pub fn new(count: usize, min_idle_time: usize, next_id_to_claim: ID) -> Self {
        ClaimMessagesOptions {
            count,
            min_idle_time,
            next_id_to_claim,
        }
    }
}

/// Reply type used to represents the new messages retrieved from a Redis stream.
#[derive(Debug, Clone)]
pub struct NewMessagesReply {
    /// List of new messages retrieved from the stream.
    messages: Vec<StreamId>,
}

impl NewMessagesReply {
    /// Get the list of new messages retrieved from the stream.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A  [`Vec`] of [`StreamId`] instances representing the new messages retrieved from the stream.
    pub fn get_messages(&self) -> &Vec<StreamId> {
        &self.messages
    }

    /// Check if the list of new messages is empty.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A [`bool`] indicating if the list of new messages is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Factory trait used to create instances of [`NewMessagesReply`].
pub trait NewMessagesReplyFactory {
    /// Create an empty instance of [`NewMessagesReply`].
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A new instance of [`NewMessagesReply`] with an empty list of messages.
    fn empty() -> NewMessagesReply;

    /// Create an instance of [`NewMessagesReply`] from the given list of messages.
    ///
    /// # Arguments:
    /// - **messages**: A [`Vec`] of [`StreamId`] instances representing the new messages retrieved from the stream.
    ///
    /// # Returns:
    /// A new instance of [`NewMessagesReply`] with the given list of messages.
    fn build(messages: Vec<StreamId>) -> NewMessagesReply;
}

impl NewMessagesReplyFactory for NewMessagesReply {
    fn empty() -> NewMessagesReply {
        NewMessagesReply {
            messages: Vec::new(),
        }
    }

    fn build(messages: Vec<StreamId>) -> NewMessagesReply {
        NewMessagesReply { messages }
    }
}

/// Reply type used to represents the pending messages retrieved from a Redis stream.
#[derive(Debug, Clone)]
pub struct PendingMessagesReply {
    /// List of pending messages retrieved from the stream.
    messages: Vec<StreamId>,

    /// The [`Id`] of the latest pending message from the current list.
    latest_pending_message_id: Option<Id>,
}

impl PendingMessagesReply {
    /// Get the list of pending messages retrieved from the stream.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A  [`Vec`] of [`StreamId`] instances representing the pending messages retrieved from the stream.
    pub fn get_messages(&self) -> &Vec<StreamId> {
        &self.messages
    }

    /// Get the [`Id`] of the latest pending message.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The [`Id`] of the latest pending message from the current list.
    pub fn get_latest_pending_message_id(&self) -> &Option<Id> {
        &self.latest_pending_message_id
    }

    /// Check if the list of pending messages is empty.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A [`bool`] indicating if the list of pending messages is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Factory trait used to create instances of [`PendingMessagesReply`].
pub trait PendingMessagesReplyFactory {
    /// Create an empty instance of [`PendingMessagesReply`].
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A new instance of [`PendingMessagesReply`] with an empty list of messages and no latest pending message.
    fn empty() -> PendingMessagesReply;

    /// Create an instance of [`PendingMessagesReply`] from the given list of messages and latest pending message [`Id`].
    ///
    /// # Arguments:
    /// - **messages**: A [`Vec`] of [`StreamId`] instances representing the pending messages retrieved from the stream.
    /// - **latest_pending_message_id**: The [`Id`] of the latest pending message.
    ///
    /// # Returns:
    /// A new instance of [`PendingMessagesReply`] with the given list of messages and latest pending message [`Id`].
    fn build(
        messages: Vec<StreamId>,
        latest_pending_message_id: Option<Id>,
    ) -> PendingMessagesReply;
}

impl PendingMessagesReplyFactory for PendingMessagesReply {
    fn empty() -> PendingMessagesReply {
        PendingMessagesReply {
            messages: Vec::new(),
            latest_pending_message_id: None,
        }
    }

    fn build(
        messages: Vec<StreamId>,
        latest_pending_message_id: Option<Id>,
    ) -> PendingMessagesReply {
        PendingMessagesReply {
            messages,
            latest_pending_message_id,
        }
    }
}

/// Reply type used to represents the claimed messages retrieved from a Redis stream.
#[derive(Debug, Clone)]
pub struct ClaimedMessagesReply {
    /// List of claimed messages retrieved from the stream.
    messages: Vec<StreamId>,

    /// The [`Id`] of the next message to claim. This value must be used to claim the next message from the stream if you can not claim the current list of messages again.
    next_id_to_claim: Option<Id>,
}

impl ClaimedMessagesReply {
    /// Get the list of claimed messages retrieved from the stream.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A  [`Vec`] of [`StreamId`] instances representing the claimed messages retrieved from the stream.
    pub fn get_messages(&self) -> &Vec<StreamId> {
        &self.messages
    }

    /// Get the [`Id`] of the next message to claim.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The [`Id`] of the next message to claim.
    pub fn get_next_id_to_claim(&self) -> &Option<Id> {
        &self.next_id_to_claim
    }

    /// Check if the list of claimed messages is empty.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A [`bool`] indicating if the list of claimed messages is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Factory trait used to create instances of [`ClaimedMessagesReply`].
pub trait ClaimedMessagesReplyFactory {
    fn empty() -> ClaimedMessagesReply;
    fn build(messages: Vec<StreamId>, next_id_to_claim: Option<Id>) -> ClaimedMessagesReply;
}

impl ClaimedMessagesReplyFactory for ClaimedMessagesReply {
    fn empty() -> ClaimedMessagesReply {
        ClaimedMessagesReply {
            messages: Vec::new(),
            next_id_to_claim: None,
        }
    }

    fn build(messages: Vec<StreamId>, next_id_to_claim: Option<Id>) -> ClaimedMessagesReply {
        ClaimedMessagesReply {
            messages,
            next_id_to_claim,
        }
    }
}

/// Enum used to represent the different types of replies that can be returned by the consume operation. In consecuence, the consume operation only returns a single instance of this enum.
#[derive(Debug, Clone)]
pub enum ConsumeReplyRepr {
    /// Represents the new messages retrieved from a Redis stream.
    New(NewMessagesReply),

    /// Represents the pending messages retrieved from a Redis stream.
    Pending(PendingMessagesReply),

    /// Represents the claimed messages retrieved from a Redis stream.
    Claimed(ClaimedMessagesReply),

    /// Represents an empty reply.
    Empty,
}

/// Reply type used to represent the response returned by the consume operation. The reply can be of different types, so it is represented by the [`ConsumeReplyRepr`].
#[derive(Debug, Clone)]
pub struct ConsumeReply {
    /// The reply representation.
    repr: ConsumeReplyRepr,
}

impl ConsumeReply {
    /// Get the reply representation.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A new instance of [`ConsumeReplyRepr`] representing the response returned by the consume operation.
    pub fn get_repr(&self) -> &ConsumeReplyRepr {
        &self.repr
    }

    /// Check if the reply contains new messages.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A [`bool`] indicating if the reply contains new messages.
    pub fn contains_new_messages(&self) -> bool {
        matches!(self.get_repr(), ConsumeReplyRepr::New(_))
    }

    /// Check if the reply contains pending messages.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A [`bool`] indicating if the reply contains pending messages.
    pub fn contains_pending_messages(&self) -> bool {
        matches!(self.get_repr(), ConsumeReplyRepr::Pending(_))
    }

    /// Check if the reply contains claimed messages.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A [`bool`] indicating if the reply contains claimed messages.
    pub fn contains_claimed_messages(&self) -> bool {
        matches!(self.get_repr(), ConsumeReplyRepr::Claimed(_))
    }

    /// Check if the reply is empty.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A [`bool`] indicating if the reply is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self.get_repr(), ConsumeReplyRepr::Empty)
    }

    /// Get the list of messages retrieved from the stream.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// A [`Vec`] of [`StreamId`] instances representing the messages retrieved from the stream. If the reply is empty, it will return `None`. If the reply is not empty, it will return the list of messages.
    pub fn get_messages(&self) -> Vec<StreamId> {
        match self.get_repr() {
            ConsumeReplyRepr::New(reply) => reply.get_messages().to_owned(),
            ConsumeReplyRepr::Pending(reply) => reply.get_messages().to_owned(),
            ConsumeReplyRepr::Claimed(reply) => reply.get_messages().to_owned(),
            _ => Vec::new(),
        }
    }

    /// Get the [`Id`] of the latest pending message. If the reply is empty or does not contain pending messages, it will return `None`.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The [`Id`] of the latest pending message.
    pub fn get_latest_pending_message_id(&self) -> Option<Id> {
        match self.get_repr() {
            ConsumeReplyRepr::Pending(reply) => reply.get_latest_pending_message_id().to_owned(),
            _ => None,
        }
    }

    /// Get the [`Id`] of the next message to claim. If the reply is empty or does not contain claimed messages, it will return `None`.
    ///
    /// # Arguments:
    /// *No arguments.*
    ///
    /// # Returns:
    /// The [`Id`] of the next message to claim.
    pub fn get_next_id_to_claim(&self) -> Option<Id> {
        match self.get_repr() {
            ConsumeReplyRepr::Claimed(reply) => reply.get_next_id_to_claim().to_owned(),
            _ => None,
        }
    }
}

impl From<ConsumeReplyRepr> for ConsumeReply {
    fn from(repr: ConsumeReplyRepr) -> Self {
        ConsumeReply { repr }
    }
}

/// Define the configuration parameters to create a consumer instance.
pub struct ConsumerConfig<'q> {
    stream_name: &'q str,
    group_name: &'q str,
    consumer_name: &'q str,
    read_new_messages_options: ReadNewMessagesOptions,
    read_pending_messages_options: ReadPendingMessagesOptions<Id>,
    claim_messages_options: ClaimMessagesOptions<Id>,
}

impl<'q> ConsumerConfig<'q> {
    /// Get **stream name**.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The stream name.
    pub fn get_stream_name(&self) -> &str {
        self.stream_name
    }

    /// Get **group name**.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The group name.
    pub fn get_group_name(&self) -> &str {
        self.group_name
    }

    /// Get **consumer name**.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The consumer name.
    pub fn get_consumer_name(&self) -> &str {
        self.consumer_name
    }

    /// Get **read new messages options**.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The read new messages options.
    pub fn get_read_new_messages_options(&self) -> &ReadNewMessagesOptions {
        &self.read_new_messages_options
    }

    /// Get **read pending messages options**.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The read pending messages options.
    pub fn get_read_pending_messages_options(&self) -> &ReadPendingMessagesOptions<Id> {
        &self.read_pending_messages_options
    }

    /// Get **claim messages options**.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The claim messages options.
    pub fn get_claim_messages_options(&self) -> &ClaimMessagesOptions<Id> {
        &self.claim_messages_options
    }

    /// Create a new [`ConsumerConfig`] instance.
    ///
    /// # Arguments:
    /// - **stream_name**: The name of the stream where messages will be consumed.
    /// - **group_name**: The name of the consumer group.
    /// - **consumer_name**: The name of the consumer.
    /// - **read_new_messages_options**: The options to configure the consume operation when reading new messages from the stream.
    /// - **read_pending_messages_options**: The options to configure the consume operation when reading pending messages from the stream.
    /// - **claim_messages_options**: The options to configure the consume operation when claiming messages from the stream.
    ///
    /// # Returns:
    /// A new [`ConsumerConfig`] instance.
    pub fn new(
        stream_name: &'q str,
        group_name: &'q str,
        consumer_name: &'q str,
        read_new_messages_options: &ReadNewMessagesOptions,
        read_pending_messages_options: &ReadPendingMessagesOptions<Id>,
        claim_messages_options: &ClaimMessagesOptions<Id>,
    ) -> Self {
        ConsumerConfig {
            stream_name,
            group_name,
            consumer_name,
            read_new_messages_options: read_new_messages_options.to_owned(),
            read_pending_messages_options: read_pending_messages_options.to_owned(),
            claim_messages_options: claim_messages_options.to_owned(),
        }
    }
}

#[cfg(test)]
mod test_consume_operation_options {
    use super::*;

    #[test]
    fn test_read_new_messages_options_builder() {
        // Define the options parameters:
        let count: usize = 10;
        let block: usize = 5;

        // Create the options instance:
        let options: ReadNewMessagesOptions = ReadNewMessagesOptions::new(count, block);

        // Check the options parameters:
        assert_eq!(options.get_count(), count);
        assert_eq!(options.get_block(), block);
    }

    #[test]
    fn test_read_new_messages_clone() {
        // Define the options parameters:
        let count: usize = 10;
        let block: usize = 5;

        // Create the options instance:
        let options: ReadNewMessagesOptions = ReadNewMessagesOptions::new(count, block);

        // Clone the options instance:
        let cloned_options: ReadNewMessagesOptions = options.clone();

        // Check the cloned options parameters:
        assert_eq!(cloned_options.get_count(), count);
        assert_eq!(cloned_options.get_block(), block);
    }

    #[test]
    fn test_read_new_messages_debug() {
        // Define the options parameters:
        let count: usize = 10;
        let block: usize = 5;

        // Create the options instance:
        let options: ReadNewMessagesOptions = ReadNewMessagesOptions::new(count, block);

        // Check the options debug representation:
        assert_eq!(
            format!("{:?}", options),
            "ReadNewMessagesOptions { count: 10, block: 5 }"
        );
    }

    #[test]
    fn test_read_pending_messages_options_builder() {
        // Define the options parameters:
        let count: usize = 10;
        let latest_pending_message_id: Id = Id::from("0-0");

        // Create the options instance:
        let options: ReadPendingMessagesOptions<Id> =
            ReadPendingMessagesOptions::new(count, latest_pending_message_id.to_owned());

        // Check the options parameters:
        assert_eq!(options.get_count(), count);
        assert!(options
            .get_latest_pending_message_id()
            .eq(&latest_pending_message_id));
    }

    #[test]
    fn test_read_pending_messages_clone() {
        // Define the options parameters:
        let count: usize = 10;
        let latest_pending_message_id: Id = Id::from("0-0");

        // Create the options instance:
        let options: ReadPendingMessagesOptions<Id> =
            ReadPendingMessagesOptions::new(count, latest_pending_message_id.to_owned());

        // Clone the options instance:
        let cloned_options: ReadPendingMessagesOptions<Id> = options.clone();

        // Check the cloned options parameters:
        assert_eq!(cloned_options.get_count(), count);
        assert!(cloned_options
            .get_latest_pending_message_id()
            .eq(&latest_pending_message_id));
    }

    #[test]
    fn test_read_pending_messages_debug() {
        // Define the options parameters:
        let count: usize = 10;
        let latest_pending_message_id: Id = Id::from("0-0");

        // Create the options instance:
        let options: ReadPendingMessagesOptions<Id> =
            ReadPendingMessagesOptions::new(count, latest_pending_message_id);

        // Check the options debug representation:
        assert_eq!(
            format!("{:?}", options),
            "ReadPendingMessagesOptions { count: 10, latest_pending_message_id: \"0-0\" }"
        );
    }

    #[test]
    fn test_claim_messages_options_builder() {
        // Define the options parameters:
        let count: usize = 10;
        let min_idle_time: usize = 1000;
        let next_id_to_claim: Id = Id::from("0-0");

        // Create the options instance:
        let options: ClaimMessagesOptions<Id> =
            ClaimMessagesOptions::new(count, min_idle_time, next_id_to_claim.to_owned());

        // Check the options parameters:
        assert_eq!(options.get_count(), count);
        assert_eq!(options.get_min_idle_time(), min_idle_time);
        assert!(options.get_next_id_to_claim().eq(&next_id_to_claim));
    }

    #[test]
    fn test_claim_messages_clone() {
        // Define the options parameters:
        let count: usize = 10;
        let min_idle_time: usize = 1000;
        let next_id_to_claim: Id = Id::from("0-0");

        // Create the options instance:
        let options: ClaimMessagesOptions<Id> =
            ClaimMessagesOptions::new(count, min_idle_time, next_id_to_claim.to_owned());

        // Clone the options instance:
        let cloned_options: ClaimMessagesOptions<Id> = options.clone();

        // Check the cloned options parameters:
        assert_eq!(cloned_options.get_count(), count);
        assert_eq!(cloned_options.get_min_idle_time(), min_idle_time);
        assert!(cloned_options.get_next_id_to_claim().eq(&next_id_to_claim));
    }

    #[test]
    fn test_claim_messages_debug() {
        // Define the options parameters:
        let count: usize = 10;
        let min_idle_time: usize = 1000;
        let next_id_to_claim: Id = Id::from("0-0");

        // Create the options instance:
        let options: ClaimMessagesOptions<Id> =
            ClaimMessagesOptions::new(count, min_idle_time, next_id_to_claim);

        // Check the options debug representation:
        assert_eq!(
            format!("{:?}", options),
            "ClaimMessagesOptions { count: 10, min_idle_time: 1000, next_id_to_claim: \"0-0\" }"
        );
    }
}

#[cfg(test)]
mod test_new_messages_reply {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_new_messages_reply_empty() {
        // Create an empty new messages reply:
        let reply: NewMessagesReply = NewMessagesReply::empty();

        // Check the reply parameters:
        assert!(reply.get_messages().is_empty());
        assert!(reply.is_empty());
    }

    #[test]
    fn test_new_messages_reply_builder() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a new messages reply:
        let reply: NewMessagesReply = NewMessagesReply::build(messages.clone());

        // Check the reply parameters:
        assert!(!reply.is_empty());
        assert!(reply.get_messages().len().eq(&1));

        assert!(reply.get_messages()[0].id.eq(&id));
        assert!(reply.get_messages()[0].map.is_empty());
    }

    #[test]
    fn test_new_messages_reply_clone() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a new messages reply:
        let reply: NewMessagesReply = NewMessagesReply::build(messages.clone());

        // Clone the new messages reply:
        let cloned_reply: NewMessagesReply = reply.clone();

        // Check the cloned reply parameters:
        assert!(!cloned_reply.is_empty());
        assert!(cloned_reply.get_messages().len().eq(&1));

        assert!(cloned_reply.get_messages()[0].id.eq(&id));
        assert!(cloned_reply.get_messages()[0].map.is_empty());
    }

    #[test]
    fn test_new_messages_reply_debug() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a new messages reply:
        let reply: NewMessagesReply = NewMessagesReply::build(messages.clone());

        // Check the reply debug representation:
        assert_eq!(
            format!("{:?}", reply),
            "NewMessagesReply { messages: [StreamId { id: \"0-0\", map: {} }] }"
        );
    }
}

#[cfg(test)]
mod test_pending_messages_reply {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_pending_messages_reply_empty() {
        // Create an empty pending messages reply:
        let reply: PendingMessagesReply = PendingMessagesReply::empty();

        // Check the reply parameters:
        assert!(reply.get_messages().is_empty());
        assert!(reply.get_latest_pending_message_id().is_none());
        assert!(reply.is_empty());
    }

    #[test]
    fn test_pending_messages_reply_builder() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a pending messages reply:
        let reply: PendingMessagesReply =
            PendingMessagesReply::build(messages.clone(), Some(id.clone()));

        // Check the reply parameters:
        assert!(!reply.is_empty());
        assert!(reply.get_messages().len().eq(&1));

        assert!(reply.get_messages()[0].id.eq(&id));
        assert!(reply.get_messages()[0].map.is_empty());

        assert!(reply.get_latest_pending_message_id().is_some());
        assert!(reply
            .get_latest_pending_message_id()
            .to_owned()
            .unwrap()
            .eq(&id));
    }

    #[test]
    fn test_pending_messages_reply_clone() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a pending messages reply:
        let reply: PendingMessagesReply =
            PendingMessagesReply::build(messages.clone(), Some(id.clone()));

        // Clone the pending messages reply:
        let cloned_reply: PendingMessagesReply = reply.clone();

        // Check the cloned reply parameters:
        assert!(!cloned_reply.is_empty());
        assert!(cloned_reply.get_messages().len().eq(&1));

        assert!(cloned_reply.get_messages()[0].id.eq(&id));
        assert!(cloned_reply.get_messages()[0].map.is_empty());

        assert!(cloned_reply.get_latest_pending_message_id().is_some());
        assert!(cloned_reply
            .get_latest_pending_message_id()
            .to_owned()
            .unwrap()
            .eq(&id));
    }

    #[test]
    fn test_pending_messages_reply_debug() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a pending messages reply:
        let reply: PendingMessagesReply =
            PendingMessagesReply::build(messages.clone(), Some(id.clone()));

        // Check the reply debug representation:
        assert_eq!(
			format!("{:?}", reply),
			"PendingMessagesReply { messages: [StreamId { id: \"0-0\", map: {} }], latest_pending_message_id: Some(\"0-0\") }"
		);
    }
}

#[cfg(test)]
mod test_claimed_messages_reply {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_claimed_messages_reply_empty() {
        // Create an empty claimed messages reply:
        let reply: ClaimedMessagesReply = ClaimedMessagesReply::empty();

        // Check the reply parameters:
        assert!(reply.get_messages().is_empty());
        assert!(reply.get_next_id_to_claim().is_none());
        assert!(reply.is_empty());
    }

    #[test]
    fn test_claimed_messages_reply_builder() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a claimed messages reply:
        let reply: ClaimedMessagesReply =
            ClaimedMessagesReply::build(messages.clone(), Some(id.clone()));

        // Check the reply parameters:
        assert!(!reply.is_empty());
        assert!(reply.get_messages().len().eq(&1));

        assert!(reply.get_messages()[0].id.eq(&id));
        assert!(reply.get_messages()[0].map.is_empty());

        assert!(reply.get_next_id_to_claim().is_some());
        assert!(reply.get_next_id_to_claim().to_owned().unwrap().eq(&id));
    }

    #[test]
    fn test_claimed_messages_reply_clone() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a claimed messages reply:
        let reply: ClaimedMessagesReply =
            ClaimedMessagesReply::build(messages.clone(), Some(id.clone()));

        // Clone the claimed messages reply:
        let cloned_reply: ClaimedMessagesReply = reply.clone();

        // Check the cloned reply parameters:
        assert!(!cloned_reply.is_empty());
        assert!(cloned_reply.get_messages().len().eq(&1));

        assert!(cloned_reply.get_messages()[0].id.eq(&id));
        assert!(cloned_reply.get_messages()[0].map.is_empty());

        assert!(cloned_reply.get_next_id_to_claim().is_some());
        assert!(cloned_reply
            .get_next_id_to_claim()
            .to_owned()
            .unwrap()
            .eq(&id));
    }

    #[test]
    fn test_claimed_messages_reply_debug() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a claimed messages reply:
        let reply: ClaimedMessagesReply =
            ClaimedMessagesReply::build(messages.clone(), Some(id.clone()));

        // Check the reply debug representation:
        assert_eq!(
			format!("{:?}", reply),
			"ClaimedMessagesReply { messages: [StreamId { id: \"0-0\", map: {} }], next_id_to_claim: Some(\"0-0\") }"
		);
    }
}

#[cfg(test)]
mod test_consume_reply {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_consume_reply_from_new_messages() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a new messages reply:
        let new_messages_reply: NewMessagesReply = NewMessagesReply::build(messages.clone());

        // Create a consume reply from the new messages reply:
        let consume_reply: ConsumeReply =
            ConsumeReply::from(ConsumeReplyRepr::New(new_messages_reply.clone()));

        // Check the consume reply parameters:
        assert!(consume_reply.contains_new_messages());
        assert!(!consume_reply.contains_pending_messages());
        assert!(!consume_reply.contains_claimed_messages());
        assert!(!consume_reply.is_empty());

        // Test get messages:
        assert!(consume_reply.get_messages().len().eq(&messages.len()));
        assert!(consume_reply.get_messages()[0].id.eq(&id));
        assert!(consume_reply.get_messages()[0].map.is_empty());

        // Test get latest pending message id:
        assert!(consume_reply.get_latest_pending_message_id().is_none());

        // Test get next id to claim:
        assert!(consume_reply.get_next_id_to_claim().is_none());
    }

    #[test]
    fn test_consume_reply_from_pending_messages() {
        // Define the messages list:
        let id: Id = "1-1".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a pending messages reply:
        let pending_messages_reply: PendingMessagesReply =
            PendingMessagesReply::build(messages.clone(), Some(id.clone()));

        // Create a consume reply from the pending messages reply:
        let consume_reply: ConsumeReply =
            ConsumeReply::from(ConsumeReplyRepr::Pending(pending_messages_reply.clone()));

        // Check the consume reply parameters:
        assert!(!consume_reply.contains_new_messages());
        assert!(consume_reply.contains_pending_messages());
        assert!(!consume_reply.contains_claimed_messages());
        assert!(!consume_reply.is_empty());

        // Test get messages:
        assert!(consume_reply.get_messages().len().eq(&messages.len()));
        assert!(consume_reply.get_messages()[0].id.eq(&id));
        assert!(consume_reply.get_messages()[0].map.is_empty());

        // Test get latest pending message id:
        assert!(consume_reply
            .get_latest_pending_message_id()
            .is_some_and(|id| id.eq(&id)));

        // Test get next id to claim:
        assert!(consume_reply.get_next_id_to_claim().is_none());
    }

    #[test]
    fn test_consume_reply_from_claimed_messages() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a claimed messages reply:
        let claimed_messages_reply: ClaimedMessagesReply =
            ClaimedMessagesReply::build(messages.clone(), Some(id.clone()));

        // Create a consume reply from the claimed messages reply:
        let consume_reply: ConsumeReply =
            ConsumeReply::from(ConsumeReplyRepr::Claimed(claimed_messages_reply.clone()));

        // Check the consume reply parameters:
        assert!(!consume_reply.contains_new_messages());
        assert!(!consume_reply.contains_pending_messages());
        assert!(consume_reply.contains_claimed_messages());
        assert!(!consume_reply.is_empty());

        // Test get messages:
        assert!(consume_reply.get_messages().len().eq(&messages.len()));
        assert!(consume_reply.get_messages()[0].id.eq(&id));
        assert!(consume_reply.get_messages()[0].map.is_empty());

        // Test get latest pending message id:
        assert!(consume_reply.get_latest_pending_message_id().is_none());

        // Test get next id to claim:
        assert!(consume_reply
            .get_next_id_to_claim()
            .is_some_and(|id| id.eq(&id)));
    }

    #[test]
    fn test_consume_reply_from_empty_response() {
        // Create an empty consume reply:
        let consume_reply: ConsumeReply = ConsumeReply::from(ConsumeReplyRepr::Empty);

        // Check the consume reply parameters:
        assert!(!consume_reply.contains_new_messages());
        assert!(!consume_reply.contains_pending_messages());
        assert!(!consume_reply.contains_claimed_messages());
        assert!(consume_reply.is_empty());

        // Test get messages:
        assert!(consume_reply.get_messages().is_empty());

        // Test get latest pending message id:
        assert!(consume_reply.get_latest_pending_message_id().is_none());

        // Test get next id to claim:
        assert!(consume_reply.get_next_id_to_claim().is_none());
    }

    #[test]
    fn test_consume_reply_clone() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a new messages reply:
        let new_messages_reply: NewMessagesReply = NewMessagesReply::build(messages.clone());

        // Create a consume reply from the new messages reply:
        let consume_reply: ConsumeReply =
            ConsumeReply::from(ConsumeReplyRepr::New(new_messages_reply.clone()));

        // Clone the consume reply:
        let cloned_consume_reply: ConsumeReply = consume_reply.clone();

        // Check the cloned consume reply parameters:
        assert!(cloned_consume_reply.contains_new_messages());
        assert!(!cloned_consume_reply.contains_pending_messages());
        assert!(!cloned_consume_reply.contains_claimed_messages());
        assert!(!cloned_consume_reply.is_empty());
    }

    #[test]
    fn test_consume_reply_debug() {
        // Define the messages list:
        let id: Id = "0-0".to_string();
        let messages: Vec<StreamId> = vec![StreamId {
            id: id.to_owned(),
            map: HashMap::new(),
        }];

        // Create a new messages reply:
        let new_messages_reply: NewMessagesReply = NewMessagesReply::build(messages.clone());

        // Create a consume reply from the new messages reply:
        let consume_reply: ConsumeReply =
            ConsumeReply::from(ConsumeReplyRepr::New(new_messages_reply.clone()));

        // Check the consume reply debug representation:
        assert_eq!(
			format!("{:?}", consume_reply),
			"ConsumeReply { repr: New(NewMessagesReply { messages: [StreamId { id: \"0-0\", map: {} }] }) }"
		);
    }
}

#[cfg(test)]
mod test_consumer_config {
    use super::*;

    #[test]
    fn test_consumer_config_builder() {
        // Define the config parameters:
        let stream_name: &str = "stream";
        let group_name: &str = "group";
        let consumer_name: &str = "consumer";
        let count: usize = 10;
        let block: usize = 5;
        let latest_pending_message_id: Id = Id::from("0-0");
        let min_idle_time: usize = 1000;
        let next_id_to_claim: Id = Id::from("0-0");

        // Create the read new messages options:
        let read_new_messages_options: &ReadNewMessagesOptions =
            &ReadNewMessagesOptions::new(count, block);

        // Create the read pending messages options:
        let read_pending_messages_options: &ReadPendingMessagesOptions<Id> =
            &ReadPendingMessagesOptions::new(count, latest_pending_message_id.to_owned());

        // Create the claim messages options:
        let claim_messages_options: &ClaimMessagesOptions<Id> =
            &ClaimMessagesOptions::new(count, min_idle_time, next_id_to_claim.to_owned());

        // Create the consumer config instance:
        let config: ConsumerConfig = ConsumerConfig::new(
            stream_name,
            group_name,
            consumer_name,
            read_new_messages_options,
            read_pending_messages_options,
            claim_messages_options,
        );

        // Check the config parameters:
        assert_eq!(config.get_stream_name(), stream_name);
        assert_eq!(config.get_group_name(), group_name);
        assert_eq!(config.get_consumer_name(), consumer_name);

        assert_eq!(
            config.get_read_new_messages_options().get_block(),
            read_new_messages_options.get_block()
        );
        assert_eq!(
            config.get_read_new_messages_options().get_count(),
            read_new_messages_options.get_count()
        );

        assert_eq!(
            config.get_read_pending_messages_options().get_count(),
            read_pending_messages_options.get_count()
        );
        assert!(config
            .get_read_pending_messages_options()
            .get_latest_pending_message_id()
            .eq(&latest_pending_message_id));

        assert_eq!(
            config.get_claim_messages_options().get_count(),
            claim_messages_options.get_count()
        );
        assert_eq!(
            config.get_claim_messages_options().get_min_idle_time(),
            claim_messages_options.get_min_idle_time()
        );
        assert!(config
            .get_claim_messages_options()
            .get_next_id_to_claim()
            .eq(&next_id_to_claim));
    }
}
