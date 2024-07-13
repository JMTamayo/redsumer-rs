use redis::{
    Client, ConnectionAddr, ConnectionInfo, ConnectionLike, ErrorKind, RedisConnectionInfo,
    RedisError,
};

#[allow(unused_imports)]
use super::types::{RedsumerError, RedsumerResult};

/// To hold credentials to authenticate in Redis, that are used when server requires it.
#[derive(Debug, Clone)]
pub struct ClientCredentials<'k> {
    user: &'k str,
    password: &'k str,
}

impl<'k> ClientCredentials<'k> {
    /// Get *user*
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *user* to authenticate in Redis.
    fn get_user(&self) -> &str {
        self.user
    }

    /// Get *password*
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *password* to authenticate in Redis.
    fn get_password(&self) -> &str {
        self.password
    }

    /// Build a new instance of [`ClientCredentials`].
    ///
    /// # Arguments:
    /// - **user**: The username to authenticate in Redis.
    /// - **password**: The password to authenticate in Redis.
    ///
    /// # Returns:
    /// A new instance of [`ClientCredentials`].
    ///
    /// # Example:
    /// ```rust,no_run
    /// use redsumer::ClientCredentials;
    ///
    /// let credentials = ClientCredentials::new("user", "password");
    /// ```
    pub fn new(user: &'k str, password: &'k str) -> ClientCredentials<'k> {
        ClientCredentials { user, password }
    }
}

/// Define  the configuration parameters to create a [`Client`] instance.
///
/// Take a look at the following supported connection URL format to infer the client arguments:
///
/// `redis://[<user>][:<password>@]<host>:<port>/<db>`
///
/// *user* and *password* are optional. If you don't need to authenticate in Redis, you can ignore them. *port* and *db* are mandatory for the connection. Another connection URL formats are not implemented yet.
#[derive(Debug, Clone)]
pub struct ClientArgs<'k, 'a> {
    credentials: Option<ClientCredentials<'k>>,
    host: &'a str,
    port: u16,
    db: i64,
}

impl<'k, 'a> ClientArgs<'k, 'a> {
    /// Get *credentials*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *credentials* to authenticate in Redis.
    fn get_credentials(&self) -> &Option<ClientCredentials> {
        &self.credentials
    }

    /// Get *host*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *host* to connect to Redis.
    fn get_host(&self) -> &str {
        self.host
    }

    /// Get *port*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *port* to connect to Redis.
    fn get_port(&self) -> u16 {
        self.port
    }

    /// Get *db*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The database to connect to Redis.
    fn get_db(&self) -> i64 {
        self.db
    }

    /// Create a new instance of [`ClientArgs`].
    ///
    /// This function is used to create a new instance of [`ClientArgs`] with specific *host*, *port*, and *database*. The *credentials*are optional.
    ///
    /// # Arguments:
    /// - **credentials**: Credentials to authenticate in Redis.
    /// - **host**: Host to connect to Redis.
    /// - **port**: Redis server port.
    /// - **db**: Redis database
    ///
    /// # Returns:
    /// A new instance of [`ClientArgs`].
    ///
    /// # Example:
    /// ```rust,no_run
    /// use redsumer::{ClientArgs, ClientCredentials};
    ///
    /// let args = ClientArgs::new(
    /// 	Some(ClientCredentials::new("user", "password")),
    /// 	"localhost",
    /// 	6379,
    /// 	0
    /// );
    /// ```
    pub fn new(
        credentials: Option<ClientCredentials<'k>>,
        host: &'a str,
        port: u16,
        db: i64,
    ) -> ClientArgs<'k, 'a> {
        ClientArgs {
            credentials,
            host,
            port,
            db,
        }
    }
}

/// To build a new instance of [`Client`].
pub trait RedisClientBuilder {
    /// Build a new instance of [`Client`].
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// A [`RedsumerResult`] with a new instance of [`Client`]. Otherwise, a [`RedsumerError`] is returned.
    fn build(&self) -> RedsumerResult<Client>;
}

impl<'k, 'a> RedisClientBuilder for ClientArgs<'k, 'a> {
    fn build(&self) -> RedsumerResult<Client> {
        let addr: ConnectionAddr =
            ConnectionAddr::Tcp(String::from(self.get_host()), self.get_port());

        let username: Option<String> = self
            .get_credentials()
            .to_owned()
            .map(|c| c.get_user().to_string());

        let password: Option<String> = self
            .get_credentials()
            .to_owned()
            .map(|c| c.get_password().to_string());

        let redis: RedisConnectionInfo = RedisConnectionInfo {
            db: self.get_db(),
            username,
            password,
        };

        Client::open(ConnectionInfo { addr, redis })
    }
}

/// Verify the connection to the Redis server.
///
/// This function is used to verify if the connection to the Redis server is working properly.
///
/// # Arguments:
/// - **c**: A reference to a connection to Redis, which implements the [`ConnectionLike`] trait.
///
/// # Returns:
/// A [`RedsumerResult`] with `()` if the connection is working properly. Otherwise, a [`RedsumerError`] is returned.
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

#[cfg(test)]
mod test_client {
    use redis_test::MockRedisConnection;

    use super::*;

    #[test]
    fn test_client_credentials() {
        // Define the user and password to authenticate in Redis:
        let user: &str = "user";
        let password: &str = "password";

        // Create a new instance of ClientCredentials:
        let credentials: ClientCredentials = ClientCredentials::new(user, password);

        // Verify if the user and password are correct:
        assert_eq!(credentials.get_user(), user);
        assert_eq!(credentials.get_password(), password);
    }

    #[test]
    fn test_redis_client_args() {
        // Define the user and password to authenticate in Redis:
        let user: &str = "user";
        let password: &str = "password";

        // Create a new instance of ClientCredentials:
        let credentials: ClientCredentials = ClientCredentials::new(user, password);

        // Define the host to connect to Redis:
        let host: &str = "localhost";

        // Define the port to connect to Redis:
        let port: u16 = 6379;

        // Define the database to connect to Redis:
        let db: i64 = 1;

        // Create a new instance of ClientArgs with default port and db:
        let args: ClientArgs = ClientArgs::new(Some(credentials), host, port, db);

        // Verify if the args are correct:
        assert!(args.get_credentials().is_some());
        assert_eq!(args.get_credentials().to_owned().unwrap().get_user(), user);
        assert_eq!(
            args.get_credentials().to_owned().unwrap().get_password(),
            password
        );
        assert_eq!(args.get_host(), host);
        assert_eq!(args.get_port(), port);
        assert_eq!(args.get_db(), db);
    }

    #[test]
    fn test_redis_client_builder_ok() {
        // Create a new instance of ClientArgs with default port and db:
        let args: ClientArgs = ClientArgs::new(None, "localhost", 6377, 16);

        // Build a new instance of Client:
        let client_result: RedsumerResult<Client> = args.build();

        // Verify if the client is correct:
        assert!(client_result.is_ok());
    }

    #[test]
    fn test_redis_client_builder_err() {
        // Create a new instance of ClientArgs with default port and db:
        let args: ClientArgs = ClientArgs::new(
            Some(ClientCredentials::new("user", "password")),
            "localhost",
            6377,
            16,
        );

        // Build a new instance of Client:
        let client_result: RedsumerResult<Client> = args.build();

        // Verify if the client is correct:
        assert!(client_result.is_ok());
    }

    #[test]
    fn test_ping_ok() {
        // Create a mock connection:
        let mut conn: MockRedisConnection = MockRedisConnection::new(vec![]);

        // Verify the connection to the server:
        assert!(ping(&mut conn).is_ok());
    }

    #[test]
    fn test_ping_error() {
        // Create a Redis client:
        let mut client: Client = ClientArgs::new(None, "localhost", 6377, 16)
            .build()
            .unwrap();

        // Verify the connection to the server:
        let result: RedsumerResult<()> = ping(&mut client);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Connection Verification Error - ClientError: The connection to the Redis server could not be verified. Please verify the client configuration or server availability");
    }
}
