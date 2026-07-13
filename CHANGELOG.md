# Changelog 📘💜

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## ✨ v0.6.0-alpha.2 [Unreleased]

### Added:

- ⚡ Expose `CommunicationProtocol` (`Resp2`, `Resp3`) in `ConnectionConfig` via `types`, with `Resp2` as the default. By [@JMTamayo](https://github.com/JMTamayo).

### Changed:

- 🚀 Upgrade the `redis` dependency from `0.28` to `1.3`. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Upgrade `redis-test` from `0.8.0` to `1.0.4`. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Raise MSRV to **1.88** and adopt Rust **edition 2024**. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Adapt the Redis client builder to the `redis` 1.x connection info API. By [@JMTamayo](https://github.com/JMTamayo).

## ✨ v0.6.0-alpha.1

### Added:

- ⚡ Migrate the crate to a hexagonal architecture with three layers: `domain` (pure business types), `application` (use case services and ports) and `infrastructure` (Redis adapter). By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement domain types `Message`, `MessageId` and `FieldValue` to replace `redis::streams::StreamId` in the public API. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement the `RedsumerError` enum as a domain-specific error type, replacing the alias to `RedisError`. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement the outbound ports `MessageProducer` and `MessageConsumer`, and the generic services `ProducerService` and `ConsumerService` with dependency injection, enabling testing with in-memory fakes through `from_port`. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `RedisStreamsAdapter` as the only component that interacts with Redis. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement the value objects `StreamName`, `GroupName` and `ConsumerName` with non-empty validation, and `PendingMessageInfo` for pending message metadata. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `ConnectionConfig` and `Credentials` as broker-neutral connection settings for the public API. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `Producer` and `Consumer` as public facades that wire the default adapter internally, so users do not need to configure ports. By [@JMTamayo](https://github.com/JMTamayo).

### Changed:

- 🚀 `Producer` and `Consumer` are now concrete facades over the application services with the default adapter pre-wired. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 `Producer::new` and `Consumer::new` receive `ConnectionConfig` instead of `ClientArgs`. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 `produce_from_map`, `produce_from_items`, `consume` and `ack` are now synchronous functions: they were `async` without any await point. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 `ConsumeMessagesReply::get_messages` now returns `Vec<Message>` instead of `Vec<StreamId>`. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 `ProducerConfig` and `ConsumerConfig` are now immutable: the consume cursors were moved to an internal consumer session state. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Producer and consumer methods now require `&mut self` since the port is injected by ownership. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Reorganize the public modules: domain types are exposed under `types`, `MessageProducer` and `MessageConsumer` live in `producer` and `consumer` respectively, `results` exposes `RedsumerResult` and the new `errors` module exposes `RedsumerError`. By [@JMTamayo](https://github.com/JMTamayo).

### Removed:

- ❌ Remove the public `redsumer::client` module: `ClientArgs`, `ClientCredentials`, `CommunicationProtocol` and `RedisStreamsAdapter` are no longer part of the public API. Use `ConnectionConfig`, `Producer` and `Consumer` instead. By [@JMTamayo](https://github.com/JMTamayo).
- ❌ Remove the public `redsumer::redis` module: `StreamId`, `Value`, `FromRedisValue`, `ToRedisArgs` and `from_redis_value` are no longer part of the public API. Use `Message` and `FieldValue` instead. By [@JMTamayo](https://github.com/JMTamayo).
- ❌ Remove the deprecated function `IsStillMineReply::is_still_mine` in favor of `belongs_to_me`. By [@JMTamayo](https://github.com/JMTamayo).
- ❌ Remove `tokio`, `time` and `uuid` from dev-dependencies: doc examples no longer require an async runtime. By [@JMTamayo](https://github.com/JMTamayo).

## ✨ v0.5.4

### Changed:

- 🚀 Modified the Redis dependency version constraint to be >=0.28.0,<0.29.0 instead of just >=0.28.0. By [@JMTamayo](https://github.com/JMTamayo).

## ✨ v0.5.3 [2025-01-27]

### Changed:

- 🚀 Update Redis version to be greater than or equal to 0.28.0. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Update development dependencies to the latest version. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Include useful commands in README.md for the development process. By [@JMTamayo](https://github.com/JMTamayo).

### Removed:

- ❌ Remove unnecessary module connection.rs. By [@JMTamayo](https://github.com/JMTamayo).
- ❌ Remove Makefile. By [@JMTamayo](https://github.com/JMTamayo).

## ✨ v0.5.2 [2024-12-05]

### Added:

- ⚡ Implement `cargo-deny` in CI pipeline to verify security vulnerabilities. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `cargo verify-project`, `yamlfmt` and `taplo` in CI pipeline to verify config files format. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `cargo-audit`, `cargo-deny`, `cargo verify-project`, `yamlfmt`and `taplo` in Makefile. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement scheduled job to run CI pipeline each Sunday at 05:00 UTC. By [@JMTamayo](https://github.com/JMTamayo).

### Changed:

- 🚀 CI pipeline refactor to split the process into the following steps: Project-Config, Docs, Build, Tests and Security. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Implement cargo-nextest to run unit tests alongside llvm-cov. By [@JMTamayo](https://github.com/JMTamayo).

## ✨ v0.5.1 [2024-11-27]

### Added:

- ⚡ Implement `belongs_to_me()` in `IsStillMineReply` to verify if the message is still in consumer pending list. Function `is_still_mine()` was deprecated. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement *Doc Tests* in CI pipeline. By [@JMTamayo](https://github.com/JMTamayo).

### Changed:

- 🚀 Unify *Unit Tests* and *Coverage* steps in CI pipeline. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Making the *Checkout* step the first step in the pipeline to ensure that the code is available prior to the execution of the CI flow. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Apply all steps for workspace in CI pipeline. By [@JMTamayo](https://github.com/JMTamayo).

## ✨ v0.5.0 [2024-10-26]

### Added:

- ⚡ Implement `Debug` for `ClientCredentials`. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `CommunicationProtocol` type to define Redis Protocol version. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `ClientArgs` and `RedisClientBuilder` to build Redis Client. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `VerifyConnection` trait and `ping()` function to verify connection to Redis Server. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `produce_from_map()`, `produce_from_items()` and `ProducerCommands` in producer core module.  By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `ProducerConfig` to manage the configuration parameters for `Producer`. Implement `ClientArgs` in  `Producer` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `ProduceMessageReply` to handle the response from `produce_from_map()` and `produce_from_items()` functions in `Producer`. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `ConsumerConfig` to manage the configuration parameters for `Consumer`. Implement `ClientArgs` in  `Consumer`. Implement `ReadNewMessagesOptions` , `ReadPendingMessagesOptions`  and `ClaimMessagesOptions`  in  `ConsumerConfig` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement types `LastDeliveredMilliseconds` and`TotalTimesDelivered` to handle the response from the `is_still_mine()` core function. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `ConsumeMessagesReply` to handle the response from `consume()` function in `Consumer`. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `AckMessageReply` to handle the response from `ack()` function in `Consumer`. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Implement `IsStillMineReply` to handle the response from `is_still_mine()` function in `Consumer`. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ Refactor in the import of library modules: The prelude, consumer, producer and client modules were implemented **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).

### Changed:

- 🚀 Rename `RedsumerProducer` to `Producer` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Include minimum line coverage target as a env variable in CI pipeline. By [@JMTamayo](https://github.com/JMTamayo).

### Removed:

- ❌ Remove `FromRedisValueHandler` from crate **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- ❌ Remove internal function `get_redis_client()` from client module. By [@JMTamayo](https://github.com/JMTamayo).
- ❌ Remove step `Upload coverage to Codecov` from CI pipeline. By [@JMTamayo](https://github.com/JMTamayo).

## ✨ v0.4.1 [2024-06-13]

### Fixed:

- 🛠 Fixing BUG reported in [issue #15](https://github.com/enerBit/redsumer-rs/issues/15) with arguments in function xclaim. By [@JMTamayo](https://github.com/JMTamayo).

## ✨ v0.4.0 [2024-04-23]

### Added:

- ⚡ Implementation of new types: `RedsumerResult`, `RedsumerError` and `Id` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ `Debug` and `Clone` implementation in `RedsumerProducer` and `RedsumerConsumer`. By [@JMTamayo](https://github.com/JMTamayo).
- ⚡ The consumer configuration parameters were implemented directly in `RedsumerConsumer` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).

### Fixed:

- 🛠 General refactoring of the package in order to improve performance. By [@JMTamayo](https://github.com/JMTamayo).

### Changed:

- 🚀 New project structure as workspace. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Update dependencies and documentation. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 Library modules reorganization **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 `FromRedisValueImplHandler` was changed to `FromRedisValueHandler` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 The `produce_from_map()` method was replaced by the `produce()` method in `RedsumerProducer` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 The `validate_pending_message_ownership()` method was replaced by `is_still_mine()` in `RedsumerConsumer` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 The acknowledge method was replaced by ack in `RedsumerConsumer` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- 🚀 The consume method was refactored in `RedsumerConsumer` in order to implement a new consumption methodology that allows scalability in distributed systems. To understand this new implementation in detail, take a look at the project https://github.com/elpablete/refactored-computing-machine. By [@JMTamayo](https://github.com/JMTamayo).

### Removed:

- ❌ The *stream_information.rs* module was removed from the project: `StreamInfo` and `StreamConsumersInfo` implementations were removed **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- ❌ `RedsumerConsumerOptions` was removed **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).
- ❌ The `produce_from_items()` method was removed from `RedsumerProducer` **[BreakingChange]**. By [@JMTamayo](https://github.com/JMTamayo).