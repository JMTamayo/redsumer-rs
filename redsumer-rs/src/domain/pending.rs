/// Represents the total time in milliseconds that elapsed since the last message was delivered to the consumer.
pub type LastDeliveredMilliseconds = usize;

/// Represents the total number of times that a message was delivered to any consumer in the group.
pub type TotalTimesDelivered = usize;

/// Information about a message that is pending in a consumer pending list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingMessageInfo {
    /// The total time in milliseconds that elapsed since the last message was delivered to the consumer.
    last_delivered_milliseconds: LastDeliveredMilliseconds,

    /// The total number of times that the message was delivered to any consumer in the group.
    total_times_delivered: TotalTimesDelivered,
}

impl PendingMessageInfo {
    /// Create a new instance of [`PendingMessageInfo`].
    ///
    /// # Arguments:
    /// - **last_delivered_milliseconds**: Milliseconds elapsed since the last delivery of the message.
    /// - **total_times_delivered**: Total number of times the message was delivered.
    ///
    /// # Returns:
    /// A new instance of [`PendingMessageInfo`].
    pub fn new(
        last_delivered_milliseconds: LastDeliveredMilliseconds,
        total_times_delivered: TotalTimesDelivered,
    ) -> PendingMessageInfo {
        PendingMessageInfo {
            last_delivered_milliseconds,
            total_times_delivered,
        }
    }

    /// Get the total time in milliseconds that elapsed since the last delivery of the message.
    pub fn get_last_delivered_milliseconds(&self) -> LastDeliveredMilliseconds {
        self.last_delivered_milliseconds
    }

    /// Get the total number of times that the message was delivered.
    pub fn get_total_times_delivered(&self) -> TotalTimesDelivered {
        self.total_times_delivered
    }
}

#[cfg(test)]
mod test_pending_message_info {
    use super::*;

    #[test]
    fn test_pending_message_info() {
        let info: PendingMessageInfo = PendingMessageInfo::new(1000, 5);

        assert_eq!(info.get_last_delivered_milliseconds(), 1000);
        assert_eq!(info.get_total_times_delivered(), 5);
    }
}
