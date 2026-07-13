//! A lightweight implementation of Redis Streams for Rust, allowing you to manage streaming messages in a simplified way. With redsumer you can:
//!
//! - **Produce** new messages in a specific *stream*.
//! - **Consume** messages from specific *stream*, setting config parameters that allow you a flexible implementation. It also provides an option to minimize the possibility of consuming the same message simultaneously by more than one consumers from the same consumer group.
//!
//! ## Getting started
//!
//! - [`types`]: messages, field values, connection settings and stream names.
//! - [`producer`]: `Producer` to publish messages in a stream.
//! - [`consumer`]: `Consumer` to read, verify and acknowledge messages.
//! - [`errors`] / [`results`]: `RedsumerError` and `RedsumerResult`.
//!
//! [`producer::Producer`] and [`consumer::Consumer`] are ready to use: they connect to the message broker and expose the produce and consume operations without extra wiring. For custom integrations or tests, use [`producer::ProducerService::from_port`] or [`consumer::ConsumerService::from_port`] with your own [`producer::MessageProducer`] or [`consumer::MessageConsumer`] implementation.
//!
//! ## Basic Usage
//!
//! #### Produce a new stream message:
//!
//! Create a new producer instance and produce a new stream message from a [BTreeMap](`std::collections::BTreeMap`):
//!
//! ```rust,no_run
//! use std::collections::BTreeMap;
//!
//! use redsumer::prelude::*;
//!
//! fn main() {
//!     let connection: ConnectionConfig = ConnectionConfig::new(None, "localhost", 6379, 0);
//!     let stream_name: &str = "my-stream";
//!
//!     let config: ProducerConfig = ProducerConfig::new(stream_name);
//!
//!     let mut producer: Producer = Producer::new(&connection, &config).unwrap_or_else(|error| {
//!         panic!("Error creating a new producer instance: {:?}", error);
//!     });
//!
//!     let mut message: BTreeMap<&str, FieldValue> = BTreeMap::new();
//!     message.insert("id", FieldValue::from("bfd8ab4b-2c48-4d95-a344-0d5539e8a207"));
//!     message.insert("started_at", FieldValue::from("2024-01-01T00:00:00Z"));
//!
//!     let reply: ProduceMessageReply = producer.produce_from_map(message)
//!         .unwrap_or_else(|error| {
//!             panic!("Error producing stream message from BTreeMap: {:?}", error.to_string());
//!         });
//!
//!     println!("Message produced with id: {:?}", reply.get_id());
//! }
//! ```
//!
//! Similar to the previous example, you can produce a message from a list of items using [produce_from_items](producer::ProducerService::produce_from_items). Field names must convert into [String] and values into [FieldValue](types::FieldValue), which is implemented for the most common primitive types.
//!
//! #### Consume messages from a stream:
//!
//! Create a new consumer instance and consume messages from stream:
//!
//! ```rust,no_run
//! use redsumer::prelude::*;
//!
//! fn main() {
//!     let connection: ConnectionConfig = ConnectionConfig::new(None, "localhost", 6379, 0);
//!     let stream_name: &str = "my-stream";
//!     let group_name: &str = "group-name";
//!     let consumer_name: &str = "consumer";
//!     let initial_stream_id: &str = "0-0";
//!     let min_idle_time_milliseconds: usize = 1000;
//!     let new_messages_count: usize = 3;
//!     let pending_messages_count: usize = 2;
//!     let claimed_messages_count: usize = 1;
//!     let block: usize = 5;
//!
//!     let config: ConsumerConfig = ConsumerConfig::new(
//!         stream_name,
//!         group_name,
//!         consumer_name,
//!         ReadNewMessagesOptions::new(
//!             new_messages_count,
//!             block
//!         ),
//!         ReadPendingMessagesOptions::new(
//!             pending_messages_count
//!         ),
//!         ClaimMessagesOptions::new(
//!             claimed_messages_count,
//!             min_idle_time_milliseconds
//!         ),
//!     );
//!
//!     let mut consumer: Consumer = Consumer::new(
//!         connection,
//!         config,
//!         Some(initial_stream_id.to_string()),
//!     ).unwrap_or_else(|error| {
//!         panic!("Error creating a new consumer instance: {:?}", error);
//!     });
//!
//!     loop {
//!         let consume_reply: ConsumeMessagesReply = consumer.consume().unwrap_or_else(|error| {
//!             panic!("Error consuming messages from stream: {:?}", error);
//!         });
//!
//!         for message in consume_reply.get_messages().to_owned() {
//!             if consumer.is_still_mine(message.get_id()).unwrap_or_else(|error| {
//!                 panic!(
//!                     "Error checking if message is still in consumer pending list: {:?}", error
//!                 );
//!             }).belongs_to_me() {
//!                 // Process message ...
//!                 println!("Processing message: {:?}", message);
//!                 // ...
//!
//!                 let ack_reply: AckMessageReply = consumer.ack(message.get_id()).unwrap_or_else(|error| {
//!                     panic!("Error acknowledging message: {:?}", error);
//!                 });
//!
//!                 if ack_reply.was_acked() {
//!                      println!("Message acknowledged: {:?}", message);
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! In this example, the [consume](consumer::ConsumerService::consume) method is called in a loop to consume messages from the stream.
//! The [consume](consumer::ConsumerService::consume) method returns a [ConsumeMessagesReply](consumer::ConsumeMessagesReply) with a list of [Message](types::Message) instances. Each [Message](types::Message) instance represents a message in the stream, with its [MessageId](types::MessageId) and its fields as [FieldValue](types::FieldValue) instances.
//! The [is_still_mine](consumer::ConsumerService::is_still_mine) method is used to check if the message is still in the consumer pending list.
//! If it is, the message is processed and then acknowledged using the [ack](consumer::ConsumerService::ack) method.
//! The [ack](consumer::ConsumerService::ack) method returns a reply indicating if the message was successfully acknowledged.
//!
//! The main objective of this message consumption strategy is to minimize the possibility that two or more consumers from the same consumer group operating simultaneously consume the same message at the same time.
//! Knowing that it is a complex problem with no definitive solution, including business logic in the message processing instance will always improve results.
//!
//! ## Contributing
//!
//! We welcome contributions to **redsumer** project. Here are some ways you can contribute:
//!
//! - **Bug Reports**: If you find a bug, please create an issue detailing the problem, the steps to reproduce it, and the expected behavior.
//! - **Feature Requests**: If you have an idea for a new feature or an enhancement to an existing one, please create an issue describing your idea.
//! - **Pull Requests**: If you've fixed a bug or implemented a new feature, we'd love to see your work! Please submit a pull request. Make sure your code follows the existing style and all tests pass.
//!
//! Thank you for your interest in improving **redsumer**!
mod application;
mod domain;
mod infrastructure;

pub mod consumer;
pub mod producer;

pub mod types {
    //! Core types: messages, identifiers, connection settings and stream names.
    pub use super::domain::connection::{CommunicationProtocol, ConnectionConfig, Credentials};
    pub use super::domain::message::{BEGINNING_OF_TIME_ID, FieldValue, Message, MessageId};
    pub use super::domain::pending::{
        LastDeliveredMilliseconds, PendingMessageInfo, TotalTimesDelivered,
    };
    pub use super::domain::stream::{ConsumerName, GroupName, StreamName};
}

pub mod results {
    //! The result type used in redsumer.
    pub use super::domain::result::RedsumerResult;
}

pub mod errors {
    //! The error types used in redsumer.
    pub use super::domain::error::RedsumerError;
}

pub mod prelude {
    //! Common imports for quick setup: producer, consumer, types, errors and results.
    pub use super::consumer::*;
    pub use super::errors::*;
    pub use super::producer::*;
    pub use super::results::*;
    pub use super::types::*;
}
