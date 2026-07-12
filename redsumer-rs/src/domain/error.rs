use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Error type for *redsumer* operations. Each variant represents a failure in a specific stage of the produce/consume lifecycle and carries a detail message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedsumerError {
    /// The connection to the message broker could not be established or was lost.
    ConnectionError(String),

    /// A configuration parameter is invalid (e.g. an empty stream name).
    ConfigurationError(String),

    /// The stream does not exist in the message broker.
    StreamNotFound(String),

    /// An error occurred while creating or verifying the consumers group.
    ConsumerGroupError(String),

    /// An error occurred while producing a message.
    ProduceError(String),

    /// An error occurred while consuming messages.
    ConsumeError(String),

    /// An error occurred while claiming pending messages.
    ClaimError(String),

    /// An error occurred while acknowledging a message.
    AckError(String),

    /// A message field value could not be interpreted as the requested type.
    InvalidFieldValue(String),
}

impl RedsumerError {
    /// Get the detail message of the error.
    pub fn get_detail(&self) -> &str {
        match self {
            RedsumerError::ConnectionError(detail)
            | RedsumerError::ConfigurationError(detail)
            | RedsumerError::StreamNotFound(detail)
            | RedsumerError::ConsumerGroupError(detail)
            | RedsumerError::ProduceError(detail)
            | RedsumerError::ConsumeError(detail)
            | RedsumerError::ClaimError(detail)
            | RedsumerError::AckError(detail)
            | RedsumerError::InvalidFieldValue(detail) => detail,
        }
    }

    /// Get the name of the error kind.
    fn get_kind_name(&self) -> &'static str {
        match self {
            RedsumerError::ConnectionError(_) => "ConnectionError",
            RedsumerError::ConfigurationError(_) => "ConfigurationError",
            RedsumerError::StreamNotFound(_) => "StreamNotFound",
            RedsumerError::ConsumerGroupError(_) => "ConsumerGroupError",
            RedsumerError::ProduceError(_) => "ProduceError",
            RedsumerError::ConsumeError(_) => "ConsumeError",
            RedsumerError::ClaimError(_) => "ClaimError",
            RedsumerError::AckError(_) => "AckError",
            RedsumerError::InvalidFieldValue(_) => "InvalidFieldValue",
        }
    }
}

impl Display for RedsumerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}: {}", self.get_kind_name(), self.get_detail())
    }
}

impl Error for RedsumerError {}

#[cfg(test)]
mod test_redsumer_error {
    use super::*;

    #[test]
    fn test_error_display() {
        let error: RedsumerError = RedsumerError::StreamNotFound("stream does not exist".into());

        assert_eq!(error.to_string(), "StreamNotFound: stream does not exist");
        assert_eq!(error.get_detail(), "stream does not exist");
    }

    #[test]
    fn test_error_kind_names() {
        let cases: Vec<(RedsumerError, &str)> = vec![
            (
                RedsumerError::ConnectionError("d".into()),
                "ConnectionError",
            ),
            (
                RedsumerError::ConfigurationError("d".into()),
                "ConfigurationError",
            ),
            (RedsumerError::StreamNotFound("d".into()), "StreamNotFound"),
            (
                RedsumerError::ConsumerGroupError("d".into()),
                "ConsumerGroupError",
            ),
            (RedsumerError::ProduceError("d".into()), "ProduceError"),
            (RedsumerError::ConsumeError("d".into()), "ConsumeError"),
            (RedsumerError::ClaimError("d".into()), "ClaimError"),
            (RedsumerError::AckError("d".into()), "AckError"),
            (
                RedsumerError::InvalidFieldValue("d".into()),
                "InvalidFieldValue",
            ),
        ];

        for (error, kind_name) in cases {
            assert!(error.to_string().starts_with(kind_name));
            assert_eq!(error.get_detail(), "d");
        }
    }

    #[test]
    fn test_error_equality_and_clone() {
        let error: RedsumerError = RedsumerError::AckError("failed".into());
        let cloned: RedsumerError = error.clone();

        assert_eq!(error, cloned);
    }

    #[test]
    fn test_error_implements_std_error() {
        fn assert_error<E: Error>(_e: &E) {}

        let error: RedsumerError = RedsumerError::ProduceError("failed".into());
        assert_error(&error);
    }
}
