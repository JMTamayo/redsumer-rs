use redis::streams::StreamId;

use crate::types::Id;

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
}

impl From<ConsumeReplyRepr> for ConsumeReply {
    fn from(repr: ConsumeReplyRepr) -> Self {
        ConsumeReply { repr }
    }
}
