//! Converts Redis stream entries into [`Message`](crate::domain::message::Message) and [`FieldValue`](crate::domain::message::FieldValue) instances.

use std::collections::BTreeMap;

use redis::{streams::StreamId, Value};

use crate::domain::message::{FieldValue, Message, MessageId};

/// Convert a Redis [`Value`] into a [`FieldValue`].
///
/// Scalar values are converted to their raw byte representation. Non-scalar values (arrays, maps, etc.) are not expected in stream entries and are converted to their debug representation as a fallback.
pub(crate) fn field_value_from_redis_value(value: &Value) -> FieldValue {
    match value {
        Value::Nil => FieldValue::new(Vec::new()),
        Value::BulkString(bytes) => FieldValue::new(bytes.to_owned()),
        Value::SimpleString(s) => FieldValue::from(s.to_owned()),
        Value::VerbatimString { text, .. } => FieldValue::from(text.to_owned()),
        Value::Int(i) => FieldValue::from(i.to_string()),
        Value::Double(d) => FieldValue::from(d.to_string()),
        Value::Boolean(b) => FieldValue::from(b.to_string()),
        Value::BigNumber(n) => FieldValue::from(n.to_string()),
        Value::Okay => FieldValue::from("OK"),
        other => FieldValue::from(format!("{:?}", other)),
    }
}

/// Convert a Redis [`StreamId`] into a [`Message`].
pub(crate) fn message_from_stream_id(stream_id: &StreamId) -> Message {
    let fields: BTreeMap<String, FieldValue> = stream_id
        .map
        .iter()
        .map(|(name, value)| (name.to_owned(), field_value_from_redis_value(value)))
        .collect();

    Message::new(MessageId::new(&stream_id.id), fields)
}

/// Convert a list of Redis [`StreamId`]s into a list of [`Message`] instances.
pub(crate) fn messages_from_stream_ids(stream_ids: &[StreamId]) -> Vec<Message> {
    stream_ids.iter().map(message_from_stream_id).collect()
}

#[cfg(test)]
mod test_field_value_from_redis_value {
    use super::*;

    #[test]
    fn test_scalar_conversions() {
        assert_eq!(
            field_value_from_redis_value(&Value::Nil).as_bytes(),
            b"" as &[u8]
        );
        assert_eq!(
            field_value_from_redis_value(&Value::BulkString(b"hello".to_vec()))
                .as_str()
                .unwrap(),
            "hello"
        );
        assert_eq!(
            field_value_from_redis_value(&Value::SimpleString("text".to_string()))
                .as_str()
                .unwrap(),
            "text"
        );
        assert_eq!(
            field_value_from_redis_value(&Value::Int(42))
                .parse::<i64>()
                .unwrap(),
            42
        );
        assert_eq!(
            field_value_from_redis_value(&Value::Double(1.5))
                .parse::<f64>()
                .unwrap(),
            1.5
        );
        assert!(field_value_from_redis_value(&Value::Boolean(true))
            .parse::<bool>()
            .unwrap());
        assert_eq!(
            field_value_from_redis_value(&Value::Okay).as_str().unwrap(),
            "OK"
        );
    }

    #[test]
    fn test_non_scalar_fallback() {
        let value: Value = Value::Array(vec![Value::Int(1)]);
        let field_value: FieldValue = field_value_from_redis_value(&value);

        assert!(!field_value.as_bytes().is_empty());
    }
}

#[cfg(test)]
mod test_message_from_stream_id {
    use super::*;

    #[test]
    fn test_message_from_stream_id() {
        let mut stream_id: StreamId = StreamId {
            id: "1-0".to_string(),
            ..Default::default()
        };
        stream_id.map.insert("code".to_string(), Value::Int(1));
        stream_id
            .map
            .insert("name".to_string(), Value::BulkString(b"test".to_vec()));

        let message: Message = message_from_stream_id(&stream_id);

        assert_eq!(message.get_id().as_str(), "1-0");
        assert_eq!(message.total_fields(), 2);
        assert_eq!(
            message.get_field("code").unwrap().parse::<i32>().unwrap(),
            1
        );
        assert_eq!(message.get_field("name").unwrap().as_str().unwrap(), "test");
    }

    #[test]
    fn test_messages_from_stream_ids() {
        let first: StreamId = StreamId {
            id: "1-0".to_string(),
            ..Default::default()
        };

        let second: StreamId = StreamId {
            id: "2-0".to_string(),
            ..Default::default()
        };

        let messages: Vec<Message> = messages_from_stream_ids(&[first, second]);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].get_id().as_str(), "1-0");
        assert_eq!(messages[1].get_id().as_str(), "2-0");
    }
}
