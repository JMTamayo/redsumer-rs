# REDSUMER

<div align="left">
	<img src="https://img.shields.io/github/license/enerBit/redsumer-rs">
	<a href="https://deps.rs/repo/github/enerBit/redsumer-rs">
		<img src="https://deps.rs/repo/github/enerBit/redsumer-rs/status.svg">
	</a>
	<a href="https://github.com/enerBit/redsumer-rs/actions/workflows/CI.yml">
		<img src="https://github.com/enerBit/redsumer-rs/actions/workflows/CI.yml/badge.svg">
	</a>
	<a href="https://crates.io/crates/redsumer">
		<img src="https://img.shields.io/crates/v/redsumer.svg?label=crates.io&color=orange&logo=rust">
	</a>
	<a href="http://docs.rs/redsumer/latest/">
		<img src="https://img.shields.io/static/v1?label=docs.rs&message=latest&color=blue&logo=docsdotrs">
	</a>
</div>

A lightweight implementation of Redis Streams for Rust, allowing you to manage streaming messages in a simplified way. With redsumer you can:

- **Produce** new messages in a specific *stream*.
- **Consume** messages from specific *stream*, setting config parameters that allow you a flexible implementation. It also provides an option to minimize the possibility of consuming the same message simultaneously by more than one consumers from the same consumer group.

## MODULES

The public API is organized by use case:

- **`types`**: `Message`, `MessageId`, `FieldValue`, `ConnectionConfig`, stream and group names, and pending message metadata.
- **`producer`**: `Producer` to publish messages; `ProducerConfig` for stream settings.
- **`consumer`**: `Consumer` to read and acknowledge messages; `ConsumerConfig` for stream, group and read options.
- **`errors`** / **`results`**: error and result types for all operations.

`Producer` and `Consumer` connect to the message broker and are ready to use. For custom backends or unit tests, use `ProducerService::from_port` or `ConsumerService::from_port` with your own implementation of `MessageProducer` or `MessageConsumer`.

## BASIC OPERATION

#### Produce a new stream message:

Create a new producer instance and produce a new stream message from a **BTreeMap**:

```rust,no_run
use std::collections::BTreeMap;

use redsumer::prelude::*;

fn main() {
    let connection: ConnectionConfig = ConnectionConfig::new(None, "localhost", 6379, 0);
    let stream_name: &str = "my-stream";

    let config: ProducerConfig = ProducerConfig::new(stream_name);

    let mut producer: Producer = Producer::new(&connection, &config).unwrap_or_else(|error| {
        panic!("Error creating a new producer instance: {:?}", error);
    });

    let mut message_1: BTreeMap<&str, FieldValue> = BTreeMap::new();
    message_1.insert("id", FieldValue::from("bfd8ab4b-2c48-4d95-a344-0d5539e8a207"));
    message_1.insert("started_at", FieldValue::from("2024-01-01T00:00:00Z"));

    let mut message_2: Vec<(String, String)> = Vec::new();
    message_2.push(("id".to_string(), "0e5fe829-4e01-4a29-b0e0-b40040dc985e".to_string()));
    message_2.push(("started_at".to_string(), "2024-01-01T00:00:00Z".to_string()));

    let reply_1: ProduceMessageReply = producer.produce_from_map(message_1).unwrap_or_else(|error| {
        panic!("Error producing stream message from BTreeMap: {:?}", error.to_string());
    });

    let reply_2: ProduceMessageReply = producer.produce_from_items(message_2).unwrap_or_else(|error| {
        panic!("Error producing stream message from Vec: {:?}", error.to_string());
    });

    println!("Message 1 produced with id: {:?}", reply_1);
    println!("Message 2 produced with id: {:?}", reply_2);
}
```

Similar to the previous example, you can produce a message from a **HashMap** or any iterator of field/value pairs.

The **produce_from_map** and **produce_from_items** methods accept generic types where field names convert into **String** and values into **FieldValue** (implemented for the most common primitive types). Take a look at the documentation for more information.

#### Consume messages from a stream:

Create a new consumer instance and consume messages from stream:

```rust,no_run
use redsumer::prelude::*;

fn main() {
    let connection: ConnectionConfig = ConnectionConfig::new(None, "localhost", 6379, 0);
    let stream_name: &str = "my-stream";
    let group_name: &str = "group-name";
    let consumer_name: &str = "consumer";
    let initial_stream_id: &str = "0-0";
    let min_idle_time_milliseconds: usize = 1000;
    let new_messages_count: usize = 3;
    let pending_messages_count: usize = 2;
    let claimed_messages_count: usize = 1;
    let block: usize = 5;

    let config: ConsumerConfig = ConsumerConfig::new(
        stream_name,
        group_name,
        consumer_name,
        ReadNewMessagesOptions::new(
            new_messages_count,
            block
        ),
        ReadPendingMessagesOptions::new(
            pending_messages_count
        ),
        ClaimMessagesOptions::new(
            claimed_messages_count,
            min_idle_time_milliseconds
        ),
    );

    let mut consumer: Consumer = Consumer::new(
        connection,
        config,
        Some(initial_stream_id.to_string()),
    ).unwrap_or_else(|error| {
        panic!("Error creating a new consumer instance: {:?}", error);
    });

    loop {
        let consume_reply: ConsumeMessagesReply = consumer.consume().unwrap_or_else(|error| {
            panic!("Error consuming messages from stream: {:?}", error);
        });

        for message in consume_reply.get_messages().to_owned() {
            if consumer.is_still_mine(message.get_id()).unwrap_or_else(|error| {
                panic!(
                    "Error checking if message is still in consumer pending list: {:?}", error
                );
            }).belongs_to_me() {
                // Process message ...
                println!("Processing message: {:?}", message);
                // ...

                let ack_reply: AckMessageReply = consumer.ack(message.get_id()).unwrap_or_else(|error| {
                    panic!("Error acknowledging message: {:?}", error);
                });

                if ack_reply.was_acked() {
                     println!("Message acknowledged: {:?}", message);
                }
            }
        }
    }
}
```

In this example, the **consume** method is called in a loop to consume messages from the stream.
The **consume** method returns a **ConsumeMessagesReply** with a list of **Message** instances. Each **Message** instance represents a message in the stream, with its **MessageId** and its fields as **FieldValue** instances.
The **is_still_mine** method is used to check if the message is still in the consumer pending list.
If it is, the message is processed and then acknowledged using the **ack** method.
The **ack** method returns a reply indicating if the message was successfully acknowledged.

The main objective of this message consumption strategy is to minimize the possibility that two or more consumers from the same consumer group operating simultaneously consume the same message at the same time.
Knowing that it is a complex problem with no definitive solution, including business logic in the message processing instance will always improve results.

#### Message fields:

Each message field is a **FieldValue**: it wraps the raw bytes of the field. Use **as_str()** to read it as UTF-8 or **parse::&lt;T&gt;()** to convert it to any type that implements **FromStr**.

#### Examples:

Take a look at the [examples](https://github.com/enerBit/redsumer-rs/tree/main/redsumer-rs/examples) directory for runnable producer and consumer examples. With a Redis server running on `localhost:6379`, you can execute them with:

```bash
cargo run --example producer
cargo run --example consumer
```

## DEVELOPMENT
To try **redsumer**, follow these recommendations:

#### Install required tools:
Install the following tools required for the development process.

##### [Cargo LLVM cov](https://github.com/taiki-e/cargo-llvm-cov/blob/main/README.md):
```bash
  cargo install cargo-llvm-cov
```

##### [Cargo Nextest](https://nexte.std):
```bash
  cargo install cargo-nextest
```

##### [Cargo Deny](https://embarkstudios.github.io/cargo-deny/):
```bash
  cargo install cargo-deny
```

##### [Taplo Client](https://taplo.tamasfe.dev):
```bash
  cargo install taplo-cli
```

##### [Yamlfmt](https://github.com/google/yamlfmt/blob/main/README.md):
```bash
  go install github.com/google/yamlfmt/cmd/yamlfmt@latest
```

#### To format the code:
```bash
  cargo fmt --all
```

#### To verify the code format:
```bash
  cargo fmt --all --check
```

#### To verify lints:
```bash
  cargo clippy --workspace --all-features
```

#### To check the project:
```bash
  cargo check --workspace --all-features
```

#### To format TOML files:
```bash
  taplo fmt
```

#### To verify the TOML files format:
```bash
  taplo fmt --check --verbose  --diff
```

#### To check correctness of crate manifest:
```bash
  cargo verify-project --verbose
```

#### To format yaml files:
```bash
  yamlfmt .
```

#### To verify the yaml files format:
```bash
  yamlfmt --lint .
```

#### To generate the documentation:
```bash
  cargo doc --workspace --all-features --document-private-items --verbose --open
```

#### To run doc tests:
```bash
  cargo test --workspace --all-features --doc
```

#### To test using llvm-cov:
```bash
  cargo llvm-cov nextest --workspace --all-features --show-missing-lines --open
```

### Security analysis:
```bash
  cargo deny --log-level debug check
```

## CONTRIBUTING

We welcome contributions to **redsumer**. Here are some ways you can contribute:

- **Bug Reports**: If you find a bug, please create an issue detailing the problem, the steps to reproduce it, and the expected behavior.
- **Feature Requests**: If you have an idea for a new feature or an enhancement to an existing one, please create an issue describing your idea.
- **Pull Requests**: If you've fixed a bug or implemented a new feature, we'd love to see your work! Please submit a pull request. Make sure your code follows the existing style and all tests pass.

Thank you for your interest in improving **redsumer**!