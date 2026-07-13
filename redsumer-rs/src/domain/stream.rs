use std::fmt::{Display, Formatter, Result as FmtResult};

use super::error::RedsumerError;
use super::result::RedsumerResult;

/// Validate that a name is not empty (ignoring surrounding whitespace).
fn validate_non_empty(name: &str, entity: &str) -> RedsumerResult<String> {
    let trimmed: &str = name.trim();
    if trimmed.is_empty() {
        return Err(RedsumerError::ConfigurationError(format!(
            "{entity} must not be empty"
        )));
    }

    Ok(trimmed.to_owned())
}

macro_rules! define_name_value_object {
    ($name:ident, $entity:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            /// Raw value of the name.
            value: String,
        }

        impl $name {
            /// Create a new instance, validating that the name is not empty.
            ///
            /// # Arguments:
            /// - **value**: The raw name value.
            ///
            /// # Returns:
            /// A [`RedsumerResult`] with the new instance. Otherwise, a [`RedsumerError::ConfigurationError`] is returned.
            pub fn new(value: &str) -> RedsumerResult<$name> {
                Ok($name {
                    value: validate_non_empty(value, $entity)?,
                })
            }

            /// Get the raw value of the name.
            pub fn as_str(&self) -> &str {
                &self.value
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                write!(f, "{}", self.value)
            }
        }
    };
}

define_name_value_object!(
    StreamName,
    "Stream name",
    "Name of a stream. It is guaranteed to be non-empty."
);

define_name_value_object!(
    GroupName,
    "Group name",
    "Name of a consumers group. It is guaranteed to be non-empty."
);

define_name_value_object!(
    ConsumerName,
    "Consumer name",
    "Name of a consumer within a consumers group. It is guaranteed to be non-empty."
);

#[cfg(test)]
mod test_stream_value_objects {
    use super::*;

    #[test]
    fn test_valid_names() {
        let stream: StreamName = StreamName::new("my-stream").unwrap();
        let group: GroupName = GroupName::new("my-group").unwrap();
        let consumer: ConsumerName = ConsumerName::new("my-consumer").unwrap();

        assert_eq!(stream.as_str(), "my-stream");
        assert_eq!(group.as_str(), "my-group");
        assert_eq!(consumer.as_str(), "my-consumer");
        assert_eq!(stream.to_string(), "my-stream");
    }

    #[test]
    fn test_names_are_trimmed() {
        let stream: StreamName = StreamName::new("  my-stream  ").unwrap();

        assert_eq!(stream.as_str(), "my-stream");
    }

    #[test]
    fn test_empty_names_are_rejected() {
        assert!(matches!(
            StreamName::new(""),
            Err(RedsumerError::ConfigurationError(_))
        ));
        assert!(matches!(
            GroupName::new("   "),
            Err(RedsumerError::ConfigurationError(_))
        ));
        assert!(matches!(
            ConsumerName::new(""),
            Err(RedsumerError::ConfigurationError(_))
        ));
    }
}
