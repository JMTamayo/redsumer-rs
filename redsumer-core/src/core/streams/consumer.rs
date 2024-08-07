use log::debug;
use redis::{Commands, RedisResult, ToRedisArgs};

use crate::types::RedsumerResult;

fn create_consumer_group<C, K, G, ID>(
    conn: &mut C,
    key: K,
    group: G,
    since_id: ID,
) -> RedisResult<bool>
where
    C: Commands,
    K: ToRedisArgs,
    G: ToRedisArgs,
    ID: ToRedisArgs,
{
    match conn.xgroup_create::<_, _, _, String>(key, group, since_id) {
        Ok(_) => {
            debug!("The consumer group was successfully created");
            Ok(true)
        }
        Err(error) => {
            if error.to_string().contains("BUSYGROUP") {
                debug!("The consumer group already exists");
                Ok(false)
            } else {
                Err(error)
            }
        }
    }
}

/// A trait that bundles methods for consuming messages from a Redis stream
pub trait ConsumerCommands<K, G>
where
    K: ToRedisArgs,
    G: ToRedisArgs,
{
    /// Create a consumer group in a Redis stream.
    ///
    /// # Arguments:
    /// - **key**: A stream key, which must implement the `ToRedisArgs` trait.
    /// - **group**: A consumers group, which must implement the `ToRedisArgs` trait.
    /// - **since_id**: The ID of the message to start consuming, which must implement the `ToRedisArgs` trait.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with [`bool`] if the consumer group was created successfully. Otherwise, a [`RedsumerError`] is returned.
    /// - If the consumer group is created, the method returns `true`.
    /// - If the consumer group already exists, the method returns `false`.
    fn create_consumer_group<ID>(&mut self, key: K, group: G, since_id: ID) -> RedsumerResult<bool>
    where
        ID: ToRedisArgs;
}

impl<C, K, G> ConsumerCommands<K, G> for C
where
    C: Commands,
    K: ToRedisArgs + ToString,
    G: ToRedisArgs,
{
    fn create_consumer_group<ID>(&mut self, key: K, group: G, since_id: ID) -> RedsumerResult<bool>
    where
        ID: ToRedisArgs,
    {
        create_consumer_group(self, key, group, since_id)
    }
}

#[cfg(test)]
mod test_create_consumer_group {
    use redis::{cmd, ErrorKind, RedisError};
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_create_non_existent_consumer_group() {
        // Define the key, group, and since_id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, &str>(
                cmd("XGROUP")
                    .arg("CREATE")
                    .arg(key)
                    .arg(group)
                    .arg(since_id),
                Ok("Ok"),
            )]);

        // Create the consumer group:
        let result: RedsumerResult<bool> = conn.create_consumer_group(key, group, since_id);

        // Verify the result:
        assert!(result.is_ok());
        assert!(result.unwrap())
    }

    #[test]
    fn test_create_existent_consumer_group() {
        // Define the key, group, and since_id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, &str>(
                cmd("XGROUP")
                    .arg("CREATE")
                    .arg(key)
                    .arg(group)
                    .arg(since_id),
                Err(RedisError::from((
                    ErrorKind::ResponseError,
                    "BUSYGROUP Consumer Group name already exists",
                ))),
            )]);

        // Create the consumer group:
        let result: RedsumerResult<bool> = conn.create_consumer_group(key, group, since_id);

        // Verify the result:
        assert!(result.is_ok());
        assert!(!result.unwrap())
    }

    #[test]
    fn test_create_consumer_group_error() {
        // Define the key, group, and since_id:
        let key: &str = "my-key";
        let group: &str = "my-group";
        let since_id: &str = "0";

        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, &str>(
                cmd("XGROUP")
                    .arg("CREATE")
                    .arg(key)
                    .arg(group)
                    .arg(since_id),
                Err(RedisError::from((ErrorKind::ResponseError, "XGROUP Error"))),
            )]);

        // Create the consumer group:
        let result: RedsumerResult<bool> = conn.create_consumer_group(key, group, since_id);

        // Verify the result:
        assert!(result.is_err());
    }
}
