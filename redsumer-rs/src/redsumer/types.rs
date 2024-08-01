use redis::{ProtocolVersion, RedisError};

/// Error type for *redsumer* operations, it's an alias for [`RedisError`].
pub type RedsumerError = RedisError;

/// Result type for *redsumer* operations.
pub type RedsumerResult<T> = Result<T, RedsumerError>;

/// Stream message identifier.
pub type Id = String;

/// Redis connection protocol.
pub type CommunicationProtocol = ProtocolVersion;
