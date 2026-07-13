use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use super::error::RedsumerError;
use super::result::RedsumerResult;

/// The ID that represents the beginning of time in a stream: `0-0`.
pub const BEGINNING_OF_TIME_ID: &str = "0-0";

/// Identifier of a message in a stream. It wraps the raw ID string (e.g. `1526919030474-55`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageId {
    /// Raw value of the message ID.
    value: String,
}

impl MessageId {
    /// Create a new [`MessageId`] from a raw string.
    ///
    /// # Arguments:
    /// - **value**: The raw ID value.
    ///
    /// # Returns:
    /// A new instance of [`MessageId`].
    pub fn new(value: &str) -> MessageId {
        MessageId {
            value: value.to_owned(),
        }
    }

    /// Create a [`MessageId`] that represents the beginning of time in a stream: `0-0`.
    pub fn beginning_of_time() -> MessageId {
        MessageId::new(BEGINNING_OF_TIME_ID)
    }

    /// Get the raw value of the ID.
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.value)
    }
}

impl From<String> for MessageId {
    fn from(value: String) -> Self {
        MessageId { value }
    }
}

impl From<&str> for MessageId {
    fn from(value: &str) -> Self {
        MessageId::new(value)
    }
}

/// Value of a message field. It wraps the raw bytes of the value and provides typed accessors like [`as_str`](FieldValue::as_str) and [`parse`](FieldValue::parse).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValue {
    /// Raw bytes of the field value.
    bytes: Vec<u8>,
}

impl FieldValue {
    /// Create a new [`FieldValue`] from raw bytes.
    ///
    /// # Arguments:
    /// - **bytes**: The raw bytes of the field value.
    ///
    /// # Returns:
    /// A new instance of [`FieldValue`].
    pub fn new(bytes: Vec<u8>) -> FieldValue {
        FieldValue { bytes }
    }

    /// Get the raw bytes of the field value.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the field value as a string slice, if it is valid UTF-8.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the string slice. Otherwise, a [`RedsumerError::InvalidFieldValue`] is returned.
    pub fn as_str(&self) -> RedsumerResult<&str> {
        std::str::from_utf8(&self.bytes).map_err(|e| {
            RedsumerError::InvalidFieldValue(format!("Field value is not valid UTF-8: {e}"))
        })
    }

    /// Parse the field value into a specific type that implements [`FromStr`].
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the parsed value. Otherwise, a [`RedsumerError::InvalidFieldValue`] is returned.
    pub fn parse<T>(&self) -> RedsumerResult<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.as_str()?.parse::<T>().map_err(|e| {
            RedsumerError::InvalidFieldValue(format!("Error parsing field value: {e}"))
        })
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes))
    }
}

impl From<Vec<u8>> for FieldValue {
    fn from(bytes: Vec<u8>) -> Self {
        FieldValue::new(bytes)
    }
}

impl From<&[u8]> for FieldValue {
    fn from(bytes: &[u8]) -> Self {
        FieldValue::new(bytes.to_vec())
    }
}

impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        FieldValue::new(value.into_bytes())
    }
}

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        FieldValue::new(value.as_bytes().to_vec())
    }
}

macro_rules! impl_field_value_from_to_string {
    ($($t:ty),*) => {
        $(
            impl From<$t> for FieldValue {
                fn from(value: $t) -> Self {
                    FieldValue::from(value.to_string())
                }
            }
        )*
    };
}

impl_field_value_from_to_string!(
    bool, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize, f32, f64
);

/// A message consumed from or produced in a stream. It is composed of a [`MessageId`] and a set of named fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// The ID of the message.
    id: MessageId,

    /// The fields of the message.
    fields: BTreeMap<String, FieldValue>,
}

impl Message {
    /// Create a new [`Message`].
    ///
    /// # Arguments:
    /// - **id**: The ID of the message.
    /// - **fields**: The fields of the message.
    ///
    /// # Returns:
    /// A new instance of [`Message`].
    pub fn new(id: MessageId, fields: BTreeMap<String, FieldValue>) -> Message {
        Message { id, fields }
    }

    /// Get the ID of the message.
    pub fn get_id(&self) -> &MessageId {
        &self.id
    }

    /// Get all the fields of the message.
    pub fn get_fields(&self) -> &BTreeMap<String, FieldValue> {
        &self.fields
    }

    /// Get a specific field value by name, if it exists.
    pub fn get_field(&self, name: &str) -> Option<&FieldValue> {
        self.fields.get(name)
    }

    /// Verify if the message contains a field by name.
    pub fn contains_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// Get the total number of fields in the message.
    pub fn total_fields(&self) -> usize {
        self.fields.len()
    }
}

#[cfg(test)]
mod test_message_id {
    use super::*;

    #[test]
    fn test_message_id_new() {
        let id: MessageId = MessageId::new("1526919030474-55");

        assert_eq!(id.as_str(), "1526919030474-55");
        assert_eq!(id.to_string(), "1526919030474-55");
    }

    #[test]
    fn test_message_id_beginning_of_time() {
        let id: MessageId = MessageId::beginning_of_time();

        assert_eq!(id.as_str(), BEGINNING_OF_TIME_ID);
    }

    #[test]
    fn test_message_id_from() {
        let from_string: MessageId = MessageId::from("1-0".to_string());
        let from_str: MessageId = MessageId::from("1-0");

        assert_eq!(from_string, from_str);
    }
}

#[cfg(test)]
mod test_field_value {
    use super::*;

    #[test]
    fn test_field_value_as_str_ok() {
        let value: FieldValue = FieldValue::from("hello");

        assert_eq!(value.as_str().unwrap(), "hello");
        assert_eq!(value.as_bytes(), b"hello");
        assert_eq!(value.to_string(), "hello");
    }

    #[test]
    fn test_field_value_as_str_invalid_utf8() {
        let value: FieldValue = FieldValue::new(vec![0xff, 0xfe]);

        assert!(matches!(
            value.as_str(),
            Err(RedsumerError::InvalidFieldValue(_))
        ));
    }

    #[test]
    fn test_field_value_parse_ok() {
        let value: FieldValue = FieldValue::from(42);

        assert_eq!(value.parse::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_field_value_parse_error() {
        let value: FieldValue = FieldValue::from("not-a-number");

        assert!(matches!(
            value.parse::<i32>(),
            Err(RedsumerError::InvalidFieldValue(_))
        ));
    }

    #[test]
    fn test_field_value_from_impls() {
        assert_eq!(FieldValue::from(true).to_string(), "true");
        assert_eq!(FieldValue::from(3.5f64).to_string(), "3.5");
        assert_eq!(FieldValue::from(7u64).to_string(), "7");
        assert_eq!(FieldValue::from("text".to_string()).to_string(), "text");
        assert_eq!(FieldValue::from(b"bytes".as_slice()).as_bytes(), b"bytes");
    }
}

#[cfg(test)]
mod test_message {
    use super::*;

    fn build_message() -> Message {
        let mut fields: BTreeMap<String, FieldValue> = BTreeMap::new();
        fields.insert("code".to_string(), FieldValue::from(1));
        fields.insert("name".to_string(), FieldValue::from("test"));

        Message::new(MessageId::new("1-0"), fields)
    }

    #[test]
    fn test_message_getters() {
        let message: Message = build_message();

        assert_eq!(message.get_id().as_str(), "1-0");
        assert_eq!(message.total_fields(), 2);
        assert!(message.contains_field("code"));
        assert!(!message.contains_field("missing"));
        assert_eq!(message.get_field("code").unwrap().parse::<u8>().unwrap(), 1);
        assert!(message.get_field("missing").is_none());
        assert_eq!(message.get_fields().len(), 2);
    }
}
