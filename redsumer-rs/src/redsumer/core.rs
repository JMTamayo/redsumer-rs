use redis::{Commands, ConnectionLike, ErrorKind, RedisError, ToRedisArgs};

use super::types::*;

pub fn ping<C>(c: &mut C) -> RedsumerResult<()>
where
    C: ConnectionLike,
{
    match c.check_connection() {
        true => Ok(()),
        false => Err(RedisError::from((
            ErrorKind::ClientError,
            "Connection Verification Error",
            "The connection to the Redis server could not be verified. Please verify the client configuration or server availability".to_string(),
        ))),
    }
}

pub fn produce_from_map<C, K, M>(conn: &mut C, key: K, map: M) -> RedsumerResult<Id>
where
    C: Commands,
    K: ToRedisArgs,
    M: ToRedisArgs,
{
    conn.xadd_map::<_, _, _, Id>(key, "*", map)
}

pub fn produce_from_items<'a, C, K, F, V>(
    conn: &mut C,
    key: K,
    items: &'a [(F, V)],
) -> RedsumerResult<Id>
where
    C: Commands,
    K: ToRedisArgs,
    F: ToRedisArgs,
    V: ToRedisArgs,
{
    conn.xadd::<_, _, _, _, Id>(key, "*", items)
}

#[cfg(test)]
mod test_ping {
    use redis::Client;
    use redis_test::MockRedisConnection;

    use super::*;

    #[tokio::test]
    async fn test_ping_ok() {
        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![]);

        // Verify the connection to the server:
        assert!(ping(&mut conn).is_ok());
    }

    #[tokio::test]
    async fn test_ping_error() {
        // Create a client from a fake host:
        let mut client: Client = Client::open("redis://fakehost:6379/0").unwrap();

        // Ping the server:
        let ping_result: RedsumerResult<()> = ping(&mut client);

        // Verify the connection to the server:
        assert!(ping_result.is_err());
        assert_eq!(ping_result.unwrap_err().to_string(), "Connection Verification Error - ClientError: The connection to the Redis server could not be verified. Please verify the client configuration or server availability");
    }
}

#[cfg(test)]
mod test_produce_from_map {
    use std::collections::BTreeMap;

    use redis::cmd;
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[tokio::test]
    async fn test_produce_from_map_ok() {
        // Define the key and id:
        let key: &str = "my-key";
        let id: &str = "*";

        // Define the map:
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        map.insert("field", "value");

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Id>(
            cmd("XADD").arg(key).arg(id).arg(map.to_owned()),
            Ok(id.to_string()),
        )]);

        // Produce the message:
        let result: RedsumerResult<Id> = produce_from_map(&mut conn, key, map);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_produce_from_map_error() {
        // Define the key and id:
        let key: &str = "my-key";
        let id: &str = "*";

        // Define the map:
        let mut map: BTreeMap<&str, &str> = BTreeMap::new();
        map.insert("field", "value");

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Id>(
            cmd("XADD").arg(key).arg(id).arg(map.to_owned()),
            Err(RedisError::from((
                ErrorKind::ResponseError,
                "XADD Error",
                "XADD command failed".to_string(),
            ))),
        )]);

        // Produce the message:
        let result: RedsumerResult<Id> = produce_from_map(&mut conn, key, map);

        // Verify the result:
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_produce_from_items {
    use redis::cmd;
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[tokio::test]
    async fn test_produce_from_items_ok() {
        // Define the key and id:
        let key: &str = "my-key";
        let id: &str = "*";

        // Define the items:
        let items: &[(&str, &str)] = &[("field", "value")];

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Id>(
            cmd("XADD").arg(key).arg(id).arg(items),
            Ok(id.to_string()),
        )]);

        // Produce the message:
        let result: RedsumerResult<Id> = produce_from_items(&mut conn, key, items);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_produce_from_items_error() {
        // Define the key and id:
        let key: &str = "my-key";
        let id: &str = "*";

        // Define the items:
        let items: &[(&str, &str)] = &[("field", "value")];

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, Id>(
            cmd("XADD").arg(key).arg(id).arg(items),
            Err(RedisError::from((
                ErrorKind::ResponseError,
                "XADD Error",
                "XADD command failed".to_string(),
            ))),
        )]);

        // Produce the message:
        let result: RedsumerResult<Id> = produce_from_items(&mut conn, key, items);

        // Verify the result:
        assert!(result.is_err());
    }
}
