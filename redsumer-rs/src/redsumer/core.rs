use log::debug;
use redis::{Commands, ConnectionLike, ErrorKind, RedisError, ToRedisArgs};

use super::types::*;

pub fn ping<C>(c: &mut C) -> RedsumerResult<()>
where
    C: ConnectionLike,
{
    match c.check_connection() {
        true => {
            debug!("Connection to the Redis server was verified successfully");
            Ok(())
        }
        false => {
            debug!("Connection to the Redis server could not be verified");
            Err(RedisError::from((
				ErrorKind::ClientError,
					"Connection Verification Error",
					"The connection to the Redis server could not be verified. Please verify the client configuration or server availability".to_string(),
				))
			)
        }
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

pub fn create_consumers_group<C, K, G, ID>(
    conn: &mut C,
    key: K,
    group: G,
    since_id: ID,
) -> RedsumerResult<()>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    ID: ToRedisArgs,
{
    match conn.xgroup_create::<_, _, _, bool>(key, group, since_id) {
        Ok(_) => {
            debug!("Consumers group was created successfully");
            Ok(())
        }
        Err(error) => {
            if error.to_string().contains("BUSYGROUP") {
                debug!("Consumers group already exists");
                Ok(())
            } else {
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod test_ping {
    use redis::Client;
    use redis_test::MockRedisConnection;

    use super::*;

    #[test]
    fn test_ping_ok() {
        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![]);

        // Verify the connection to the server:
        assert!(ping(&mut conn).is_ok());
    }

    #[test]
    fn test_ping_error() {
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

    #[test]
    fn test_produce_from_map_ok() {
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

    #[test]
    fn test_produce_from_map_error() {
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
mod test_create_consumers_group {
    use redis::cmd;
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_create_consumers_group_ok() {
        // Define the key, group, and since id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XGROUP")
                .arg("CREATE")
                .arg(key)
                .arg(group)
                .arg(since_id),
            Ok(1),
        )]);

        // Create the consumers group:
        let result: RedsumerResult<()> = create_consumers_group(&mut conn, key, group, since_id);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_consumers_group_already_exists() {
        // Define the key, group, and since id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XGROUP")
                .arg("CREATE")
                .arg(key)
                .arg(group)
                .arg(since_id),
            Err(RedisError::from((
                ErrorKind::ResponseError,
                "BUSYGROUP Error",
                "BUSYGROUP Consumer Group already exists".to_string(),
            ))),
        )]);

        // Create the consumers group:
        let result: RedsumerResult<()> = create_consumers_group(&mut conn, key, group, since_id);

        // Verify the result:
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_consumers_group_error() {
        // Define the key, group, and since id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![MockCmd::new::<_, i64>(
            cmd("XGROUP")
                .arg("CREATE")
                .arg(key)
                .arg(group)
                .arg(since_id),
            Err(RedisError::from((
                ErrorKind::ResponseError,
                "XGROUP Error",
                "XGROUP command failed".to_string(),
            ))),
        )]);

        // Create the consumers group:
        let result: RedsumerResult<()> = create_consumers_group(&mut conn, key, group, since_id);

        // Verify the result:
        assert!(result.is_err());
    }
}
