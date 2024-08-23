use redis::{Commands, RedisResult, ToRedisArgs};

#[allow(unused_imports)]
use crate::results::{Id, RedsumerError, RedsumerResult};
use crate::streams::producer::models::ProducerConfig;

fn produce_from_map<C, K, M>(c: &mut C, key: K, map: M) -> RedisResult<Id>
where
    C: Commands,
    K: ToRedisArgs,
    M: ToRedisArgs,
{
    c.xadd_map(key, "*", map)
}

fn produce_from_items<C, K, F, V>(c: &mut C, key: K, items: &[(F, V)]) -> RedisResult<Id>
where
    C: Commands,
    K: ToRedisArgs,
    F: ToRedisArgs,
    V: ToRedisArgs,
{
    c.xadd(key, "*", items)
}

/// A trait that bundles methods for producing messages in a Redis stream
pub trait ProducerCommands {
    /// Produce a message to a Redis stream from a map.
    ///
    /// # Arguments:
    /// - **producer_config**: Producer configuration parameters.
    /// - **map**: A map with the message fields and values, which must implement the `ToRedisArgs` trait.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the message [`Id`] if the message was produced successfully. Otherwise, a [`RedsumerError`] is returned.
    fn produce_from_map<M>(&mut self, config: &ProducerConfig, map: M) -> RedsumerResult<Id>
    where
        M: ToRedisArgs;

    /// Produce a message to a Redis stream from a list of items.
    ///
    /// # Arguments:
    ///  - **producer_config**: Producer configuration parameters.
    /// - **items**: A list of tuples with the message fields and values, which must implement the `ToRedisArgs` trait.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with the message [`Id`] if the message was produced successfully. Otherwise, a [`RedsumerError`] is returned.
    fn produce_from_items<F, V>(
        &mut self,
        config: &ProducerConfig,
        items: &[(F, V)],
    ) -> RedsumerResult<Id>
    where
        F: ToRedisArgs,
        V: ToRedisArgs;
}

impl<C> ProducerCommands for C
where
    C: Commands,
{
    fn produce_from_map<M>(&mut self, config: &ProducerConfig, map: M) -> RedsumerResult<Id>
    where
        M: ToRedisArgs,
    {
        produce_from_map(self, config.get_stream_name(), map)
    }

    fn produce_from_items<F, V>(
        &mut self,
        config: &ProducerConfig,
        items: &[(F, V)],
    ) -> RedsumerResult<Id>
    where
        F: ToRedisArgs,
        V: ToRedisArgs,
    {
        produce_from_items(self, config.get_stream_name(), items)
    }
}

#[cfg(test)]
mod test_produce_from_map {
    use std::collections::BTreeMap;

    use redis::{cmd, ErrorKind, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_produce_from_map_ok() {
        // Define the key and id:
        let key: &str = "my-key";

        // Create a ProducerConfig instance.
        let config: ProducerConfig = ProducerConfig::new(key);

        // Define the map:
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        map.insert("field", "value");

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XADD").arg(key).arg("*").arg(map.to_owned()),
                Ok(Value::SimpleString("1-0".to_string())),
            )]);

        // Produce the message:
        let result: RedsumerResult<Id> = conn.produce_from_map(&config, map);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[test]
    fn test_produce_from_map_error() {
        // Define the key and id:
        let key: &str = "my-key";

        // Create a ProducerConfig instance.
        let config: ProducerConfig = ProducerConfig::new(key);

        // Define the map:
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        map.insert("field", "value");

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XADD").arg(key).arg("*").arg(map.to_owned()),
                Err(RedsumerError::from((
                    ErrorKind::ResponseError,
                    "XADD Error",
                    "XADD command failed".to_string(),
                ))),
            )]);

        // Produce the message:
        let result: RedsumerResult<Id> = conn.produce_from_map(&config, map);

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_produce_from_items {
    use redis::{cmd, ErrorKind, Value};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_produce_from_items_ok() {
        // Define the key and id:
        let key: &str = "my-key";

        // Create a ProducerConfig instance.
        let config: ProducerConfig = ProducerConfig::new(key);

        // Define the items:
        let items: Vec<(&str, u8)> = vec![("number", 3), ("double", 6)];

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XADD").arg(key).arg("*").arg(&items),
                Ok(Value::SimpleString("1-0".to_string())),
            )]);

        // Produce the message:
        let result: RedsumerResult<Id> = conn.produce_from_items(&config, &items);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[test]
    fn test_produce_from_items_error() {
        // Define the key and id:
        let key: &str = "my-key";

        // Create a ProducerConfig instance:
        let config: ProducerConfig = ProducerConfig::new(key);

        // Define the items:
        let items: Vec<(&str, &str)> = vec![("field", "value")];

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, Value>(
                cmd("XADD").arg(key).arg("*").arg(&items),
                Err(RedsumerError::from((
                    ErrorKind::ResponseError,
                    "XADD Error",
                    "XADD command failed".to_string(),
                ))),
            )]);

        // Produce the message:
        let result: RedsumerResult<Id> = conn.produce_from_items(&config, &items);

        // Verify the result:
        assert!(result.is_err());
    }
}
