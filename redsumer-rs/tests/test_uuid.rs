#[cfg(feature = "uuid")]
#[cfg(test)]
mod test_feature_uuid {
    use num_bigint::{BigInt, Sign};
    use redis::{PushKind, Value, VerbatimFormat};
    use uuid::Uuid;

    use redsumer::{FromRedisValueExtended, RedsumerResult};

    #[test]
    fn test_uuid_from_redis_value() {
        let uuid_str: &str = "d1cc4b5e-3295-4924-b05b-e59bfcbf5f21";
        let uuid_base: Uuid = Uuid::parse_str(uuid_str).unwrap_or_else(|error| {
            panic!("Failed to parse UUID from string '{uuid_str}': {error} ")
        });

        // Test from [`Value::Nil`]
        let uuid_from_nil: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Nil);

        assert!(uuid_from_nil.is_err());
        assert_eq!(
            uuid_from_nil.unwrap_err().to_string(),
            "Response was of incompatible type - TypeError: \"Response type not string compatible.\" (response was nil)"
        );

        // Test from [`Value::Int`]
        let uuid_from_int: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Int(123));

        assert!(uuid_from_int.is_err());
        assert_eq!(
            uuid_from_int.unwrap_err().to_string(),
            "Response was of incompatible type - TypeError: Value int(123) is not parsable as Uuid: \"invalid length: expected length 32 for simple format, found 3\""
        );

        // Test from [`Value::BulkString`]
        let uuid_from_bulk_string: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::BulkString(uuid_str.as_bytes().to_vec()));

        assert!(uuid_from_bulk_string.is_ok());
        assert_eq!(uuid_from_bulk_string.unwrap(), uuid_base);

        // Test from [`Value::Array`]
        let uuid_from_array: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Array(vec![
            Value::SimpleString(uuid_str.to_string()),
            Value::Int(1),
        ]));

        assert!(uuid_from_array.is_err());
        assert_eq!(
			uuid_from_array.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: \"Response type not string compatible.\" (response was array([simple-string(\"d1cc4b5e-3295-4924-b05b-e59bfcbf5f21\"), int(1)]))"
		);

        // Test from [`Value::SimpleString`]
        let uuid_from_simple_string: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::SimpleString(uuid_str.to_string()));

        assert!(uuid_from_simple_string.is_ok());
        assert_eq!(uuid_from_simple_string.unwrap(), uuid_base);

        // Test from [`Value::Okay`]
        let uuid_from_okay: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Okay);

        assert!(uuid_from_okay.is_err());
        assert_eq!(
			uuid_from_okay.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: Value ok is not parsable as Uuid: \"invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `O` at 1\""
		);

        // Test from [`Value::Map`]
        let uuid_from_map: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Map(vec![(
            Value::SimpleString(uuid_str.to_string()),
            Value::Int(1),
        )]));

        assert!(uuid_from_map.is_err());
        assert_eq!(
			uuid_from_map.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: \"Response type not string compatible.\" (response was map([(simple-string(\"d1cc4b5e-3295-4924-b05b-e59bfcbf5f21\"), int(1))]))"
		);

        // Test success from [`Value::Attribute`]
        let uuid_from_attribute: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Attribute {
            data: Box::new(Value::SimpleString(uuid_str.to_string())),
            attributes: vec![(
                Value::SimpleString("key".to_string()),
                Value::SimpleString("value".to_string()),
            )],
        });

        assert!(uuid_from_attribute.is_ok());
        assert_eq!(uuid_from_attribute.unwrap(), uuid_base);

        // Test error from [`Value::Attribute`]
        let uuid_from_attribute_error: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::Attribute {
                data: Box::new(Value::SimpleString("invalid-uuid".to_string())),
                attributes: vec![(
                    Value::SimpleString("key".to_string()),
                    Value::SimpleString("value".to_string()),
                )],
            });

        assert!(uuid_from_attribute_error.is_err());
        assert_eq!(
			uuid_from_attribute_error.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: Value attribute(simple-string(\"invalid-uuid\")) is not parsable as Uuid: \"invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `i` at 1\""
		);

        // Test from [`Value::Set`]
        let uuid_from_set: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Set(vec![
            Value::SimpleString(uuid_str.to_string()),
            Value::Int(1),
        ]));

        assert!(uuid_from_set.is_err());
        assert_eq!(
			uuid_from_set.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: \"Response type not string compatible.\" (response was set([simple-string(\"d1cc4b5e-3295-4924-b05b-e59bfcbf5f21\"), int(1)]))"
		);

        // Test from [`Value::Double`]
        let uuid_from_double: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Double(1.0));

        assert!(uuid_from_double.is_err());
        assert_eq!(
			uuid_from_double.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: Value double(1.0) is not parsable as Uuid: \"invalid length: expected length 32 for simple format, found 1\""
		);

        // Test from [`Value::Boolean`]
        let uuid_from_boolean: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Boolean(true));

        assert!(uuid_from_boolean.is_err());
        assert_eq!(
			uuid_from_boolean.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: \"Response type not string compatible.\" (response was boolean(true))"
		);

        // Test successfull from [`Value::VerbatimString`] with [`VerbatimFormat::Text`]
        let uuid_from_verbatim_string: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::VerbatimString {
                format: VerbatimFormat::Text,
                text: uuid_str.to_string(),
            });

        assert!(uuid_from_verbatim_string.is_ok());
        assert_eq!(uuid_from_verbatim_string.unwrap(), uuid_base);

        // Test error from [`Value::VerbatimString`] with [`VerbatimFormat::Text`]
        let uuid_from_verbatim_string_error: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::VerbatimString {
                format: VerbatimFormat::Text,
                text: "invalid-uuid".to_string(),
            });

        assert!(uuid_from_verbatim_string_error.is_err());
        assert_eq!(
			uuid_from_verbatim_string_error.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: Value verbatim-string(Text,\"invalid-uuid\") is not parsable as Uuid: \"invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `i` at 1\""
		);

        // Test successfull from [`Value::VerbatimString`] with [`VerbatimFormat::Markdown`]
        let uuid_from_verbatim_string_markdown: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::VerbatimString {
                format: VerbatimFormat::Markdown,
                text: uuid_str.to_string(),
            });

        assert!(uuid_from_verbatim_string_markdown.is_ok());
        assert_eq!(uuid_from_verbatim_string_markdown.unwrap(), uuid_base);

        // Test error from [`Value::VerbatimString`] with [`VerbatimFormat::Markdown`]
        let uuid_from_verbatim_string_error_markdown: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::VerbatimString {
                format: VerbatimFormat::Markdown,
                text: "invalid-uuid".to_string(),
            });

        assert!(uuid_from_verbatim_string_error_markdown.is_err());
        assert_eq!(
			uuid_from_verbatim_string_error_markdown.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: Value verbatim-string(Markdown,\"invalid-uuid\") is not parsable as Uuid: \"invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `i` at 1\""
		);

        // Test successfull from [`Value::VerbatimString`] with [`VerbatimFormat::Unknown`]
        let uuid_from_verbatim_string_unknown: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::VerbatimString {
                format: VerbatimFormat::Unknown("unknown-format".to_string()),
                text: uuid_str.to_string(),
            });

        assert!(uuid_from_verbatim_string_unknown.is_ok());
        assert_eq!(uuid_from_verbatim_string_unknown.unwrap(), uuid_base);

        // Test error from [`Value::VerbatimString`] with [`VerbatimFormat::Unknown`]
        let uuid_from_verbatim_string_error_unknown: RedsumerResult<Uuid> =
            Uuid::from_redis_value(&Value::VerbatimString {
                format: VerbatimFormat::Unknown("unknown-format".to_string()),
                text: "invalid-uuid".to_string(),
            });

        assert!(uuid_from_verbatim_string_error_unknown.is_err());
        assert_eq!(
			uuid_from_verbatim_string_error_unknown.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: Value verbatim-string(Unknown(\"unknown-format\"),\"invalid-uuid\") is not parsable as Uuid: \"invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `i` at 1\""
		);

        // Test from [`Value::BigNumber`]
        let uuid_from_big_number: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::BigNumber(
            BigInt::from_slice(Sign::Plus, &[1, 2, 3]),
        ));

        assert!(uuid_from_big_number.is_err());
        assert_eq!(
			uuid_from_big_number.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: \"Response type not string compatible.\" (response was big-number(55340232229718589441))"
		);

        // Test from [`Value::Push`]
        let uuid_from_push: RedsumerResult<Uuid> = Uuid::from_redis_value(&Value::Push {
            kind: PushKind::SMessage,
            data: vec![Value::SimpleString(uuid_str.to_string())],
        });

        assert!(uuid_from_push.is_err());
        assert_eq!(
			uuid_from_push.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: \"Response type not string compatible.\" (response was push(SMessage, [simple-string(\"d1cc4b5e-3295-4924-b05b-e59bfcbf5f21\")]))"
		);
    }

    #[test]
    fn test_optional_uuid_from_redis_value() {
        let uuid_str: &str = "d1cc4b5e-3295-4924-b05b-e59bfcbf5f21";
        let uuid_base: Uuid = Uuid::parse_str(uuid_str).unwrap_or_else(|error| {
            panic!("Failed to parse UUID from string '{uuid_str}': {error} ")
        });

        // Test from [`Value::Nil`]
        let uuid_from_nil: RedsumerResult<Option<Uuid>> =
            Option::<Uuid>::from_redis_value(&Value::Nil);

        assert!(uuid_from_nil.is_ok());
        assert_eq!(uuid_from_nil.unwrap(), None);

        // Test from [`Value::BulkString`]
        let uuid_from_bulk_string: RedsumerResult<Option<Uuid>> =
            Option::<Uuid>::from_redis_value(&Value::BulkString(uuid_str.as_bytes().to_vec()));

        assert!(uuid_from_bulk_string.is_ok());
        assert_eq!(uuid_from_bulk_string.unwrap(), Some(uuid_base));

        // Test from [`Value::Int`]
        let uuid_from_int: RedsumerResult<Option<Uuid>> =
            Option::<Uuid>::from_redis_value(&Value::Int(123));

        assert!(uuid_from_int.is_err());
        assert_eq!(
			uuid_from_int.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: Value int(123) is not parsable as Uuid: \"invalid length: expected length 32 for simple format, found 3\""
		);

        // Test from [`Value::Array`]
        let uuid_from_array: RedsumerResult<Option<Uuid>> =
            Option::<Uuid>::from_redis_value(&Value::Array(vec![
                Value::SimpleString(uuid_str.to_string()),
                Value::Int(1),
            ]));

        assert!(uuid_from_array.is_err());
        assert_eq!(
			uuid_from_array.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: \"Response type not string compatible.\" (response was array([simple-string(\"d1cc4b5e-3295-4924-b05b-e59bfcbf5f21\"), int(1)]))"
		);

        // Test from [`Value::SimpleString`]
        let uuid_from_simple_string: RedsumerResult<Option<Uuid>> =
            Option::<Uuid>::from_redis_value(&Value::SimpleString(uuid_str.to_string()));

        assert!(uuid_from_simple_string.is_ok());
        assert_eq!(uuid_from_simple_string.unwrap(), Some(uuid_base));

        // Test from [`Value::Okay`]
        let uuid_from_okay: RedsumerResult<Option<Uuid>> =
            Option::<Uuid>::from_redis_value(&Value::Okay);

        assert!(uuid_from_okay.is_err());
        assert_eq!(
			uuid_from_okay.unwrap_err().to_string(),
			"Response was of incompatible type - TypeError: Value ok is not parsable as Uuid: \"invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `O` at 1\""
		);
    }
}
