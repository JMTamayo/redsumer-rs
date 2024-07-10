use redis::{
    Client, ConnectionAddr, ConnectionInfo, ConnectionLike, ErrorKind, RedisConnectionInfo,
    RedisError,
};

use super::types::RedsumerResult;

#[derive(Clone)]
/// To hold credentials to authenticate in *Redis*.
///
/// This credentials are used to authenticate in *Redis* when server requires it. If server does not require it, you set it to `None`.
pub struct ClientCredentials<'k> {
    user: &'k str,
    password: &'k str,
}

impl<'k> ClientCredentials<'k> {
    /// Get *user*
    fn get_user(&self) -> &str {
        self.user
    }

    /// Get *password*
    fn get_password(&self) -> &str {
        self.password
    }

    /// Build a new instance of [`ClientCredentials`].
    ///
    /// # Arguments:
    /// - **user**: Redis user.
    /// - **password**: Redis password.
    ///
    /// # Returns:
    /// A new instance of [`ClientCredentials`].
    ///
    /// ```rust,no_run
    /// use redsumer::ClientCredentials;
    /// let credentials = ClientCredentials::new("user", "password");
    /// ```
    pub fn new(user: &'k str, password: &'k str) -> ClientCredentials<'k> {
        ClientCredentials { user, password }
    }
}

pub struct RedisClientArgs<'k> {
    credentials: Option<ClientCredentials<'k>>,
    host: &'k str,
    port: Option<u16>,
    db: Option<u8>,
}

impl<'k> RedisClientArgs<'k> {
    pub fn get_credentials(&self) -> &Option<ClientCredentials<'k>> {
        &self.credentials
    }

    pub fn get_host(&self) -> &str {
        self.host
    }

    pub fn get_port(&self) -> u16 {
        self.port.unwrap_or(6379)
    }

    pub fn get_db(&self) -> i64 {
        self.db.unwrap_or(0) as i64
    }

    pub fn set_credentials(&mut self, credentials: ClientCredentials<'k>) -> &mut Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }

    pub fn set_db(&mut self, db: u8) -> &mut Self {
        self.db = Some(db);
        self
    }

    pub fn new(host: &'k str) -> RedisClientArgs<'k> {
        RedisClientArgs {
            credentials: None,
            host,
            port: None,
            db: None,
        }
    }
}

pub trait RedisClientBuilder {
    fn build(&self) -> RedsumerResult<Client>;
}

impl<'k> RedisClientBuilder for RedisClientArgs<'k> {
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

pub fn ping<C>(c: &mut C) -> RedsumerResult<()>
where
    C: ConnectionLike,
{
    match redis::cmd("PING").query::<String>(c).is_ok() {
        true => Ok(()),
        false => Err(RedisError::from((
            ErrorKind::ClientError,
            "Connection Verification Error",
            format!(
				"The connection to the Redis server could not be verified. Please verify the client configuration or server availability"
			),
        ))),
    }
}

#[cfg(test)]
pub mod test_client {
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[test]
    fn test_client_credentials() {
        // Define the user and password to authenticate in Redis:
        let user: &str = "user";
        let password: &str = "password";

        // Create a new instance of ClientCredentials:
        let credentials: ClientCredentials = ClientCredentials::new(user, password);

        // Check user and password are correct:
        assert_eq!(credentials.get_user(), user);
        assert_eq!(credentials.get_password(), password);
    }

    #[test]
    fn test_redis_client_args() {
        // Define the host to connect to Redis:
        let host: &str = "localhost";

        // Create a new instance of RedisClientArgs with default port and db:
        let mut args: RedisClientArgs = RedisClientArgs::new(host);

        // Check host is correct:
        assert_eq!(args.get_host(), host);
        assert_eq!(args.get_port(), 6379);
        assert_eq!(args.get_db(), 0);

        // Define the user and password to authenticate in Redis:
        let user: &str = "user";
        let password: &str = "password";

        // Create a new instance of ClientCredentials:
        let credentials: ClientCredentials = ClientCredentials::new(user, password);

        // Set credentials in RedisClientArgs:
        args.set_credentials(credentials);

        // Check credentials are correct:
        assert_eq!(args.get_credentials().to_owned().unwrap().get_user(), user);
        assert_eq!(
            args.get_credentials().to_owned().unwrap().get_password(),
            password
        );

        // Define the port to connect to Redis:
        let port: u16 = 6380;

        // Set port in RedisClientArgs:
        args.set_port(port);

        // Check port is correct:
        assert_eq!(args.get_port(), port);

        // Define the database to connect to Redis:
        let db: u8 = 1;

        // Set database in RedisClientArgs:
        args.set_db(db);

        // Check database is correct:
        assert_eq!(args.get_db(), db.into());
    }

    #[test]
    fn test_redis_client_builder() {
        // Define the host to connect to Redis:
        let host: &str = "localhost";

        // Create a new instance of RedisClientArgs with default port and db:
        let mut args: RedisClientArgs = RedisClientArgs::new(host);

        // Build a new instance of Client:
        let client_result: RedsumerResult<Client> = args.build();

        // Check client is correct:
        assert!(client_result.is_ok());

        // Define the user and password to authenticate in Redis:
        let user: &str = "user";
        let password: &str = "password";

        // Create a new instance of ClientCredentials:
        let credentials: ClientCredentials = ClientCredentials::new(user, password);

        // Set credentials in RedisClientArgs:
        args.set_credentials(credentials);

        // Build a new instance of Client:
        let client_result: RedsumerResult<Client> = args.build();

        // Check client is correct:
        assert!(client_result.is_ok());
    }

    #[test]
    fn test_ping_ok() {
        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, String>(
                redis::cmd("PING"),
                Ok("1".to_string()),
            )]);

        // Check server availability:
        assert!(ping(&mut conn).is_ok());
    }

    #[test]
    fn test_ping_error() {
        // Create a mock connection:
        let mut conn: MockRedisConnection =
            MockRedisConnection::new(vec![MockCmd::new::<_, String>(
                redis::cmd("PING"),
                Err(RedisError::from((
                    ErrorKind::ExtensionError,
                    "Connection Verification Error",
                ))),
            )]);

        // Check server availability:
        assert!(ping(&mut conn).is_err());
    }
}

/// Get a new [`Client`] instance to connect to *Redis* using a connection URL in format:
/// `redis://[<username>][:<password>@]<host>:<port>/<db>`
///
/// # Arguments:
/// - **credentials**: Option to authenticate in *Redis*.
/// - **host**: Redis host.
/// - **port**: Redis port.
/// - **db**: Redis database.
///
/// # Returns:
/// - A [`RedsumerResult`] with a new instance of [`Client`] to connect to *Redis*. Otherwise, a [`RedsumerError`] is returned.
pub fn get_redis_client(
    credentials: Option<ClientCredentials>,
    host: &str,
    port: &str,
    db: &str,
) -> RedsumerResult<Client> {
    let url: String = match credentials {
        Some(credentials) => {
            format!(
                "redis://{}:{}@{}:{}/{}",
                credentials.get_user(),
                credentials.get_password(),
                host,
                port,
                db,
            )
        }
        None => format!("redis://{}:{}/{}", host, port, db,),
    };

    Ok(Client::open(url)?)
}
