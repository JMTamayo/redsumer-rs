use redis::{
    Client, ConnectionAddr, ConnectionInfo, ConnectionLike, ErrorKind, RedisConnectionInfo,
    RedisError,
};

use super::types::RedsumerResult;

#[derive(Clone)]
/// To hold credentials to authenticate in *Redis*.
///
/// These credentials are used to authenticate in *Redis* when server requires it.
/// If a server does not require it, you set it to `None`.
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

pub struct ClientArgs<'k> {
    credentials: Option<ClientCredentials<'k>>,
    host: &'k str,
    port: Option<u16>,
    db: Option<u8>,
}

impl<'k> ClientArgs<'k> {
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

    pub fn new(host: &'k str) -> ClientArgs<'k> {
        ClientArgs {
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

impl<'k> RedisClientBuilder for ClientArgs<'k> {
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
pub mod test_client {
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
        // Define the host to connect to Redis:
        let host: &str = "localhost";

        // Create a new instance of ClientArgs with default port and db:
        let mut args: ClientArgs = ClientArgs::new(host);

        // Verify if the args are correct:
        assert_eq!(args.get_host(), host);
        assert_eq!(args.get_port(), 6379);
        assert_eq!(args.get_db(), 0);

        // Define the user and password to authenticate in Redis:
        let user: &str = "user";
        let password: &str = "password";

        // Create a new instance of ClientCredentials:
        let credentials: ClientCredentials = ClientCredentials::new(user, password);

        // Set credentials in ClientArgs:
        args.set_credentials(credentials);

        // Verify if the credentials are correct:
        assert_eq!(args.get_credentials().to_owned().unwrap().get_user(), user);
        assert_eq!(
            args.get_credentials().to_owned().unwrap().get_password(),
            password
        );

        // Define the port to connect to Redis:
        let port: u16 = 6380;

        // Set port in ClientArgs:
        args.set_port(port);

        // Check if the port is correct:
        assert_eq!(args.get_port(), port);

        // Define the database to connect to Redis:
        let db: u8 = 1;

        // Set database in ClientArgs:
        args.set_db(db);

        // Verify if the database is correct:
        assert_eq!(args.get_db(), db.into());
    }

    #[test]
    fn test_redis_client_builder() {
        // Define the host to connect to Redis:
        let host: &str = "localhost";

        // Create a new instance of ClientArgs with default port and db:
        let mut args: ClientArgs = ClientArgs::new(host);

        // Build a new instance of Client:
        let client_result: RedsumerResult<Client> = args.build();

        // Verify if a client is correct:
        assert!(client_result.is_ok());

        // Define the user and password to authenticate in Redis:
        let user: &str = "user";
        let password: &str = "password";

        // Create a new instance of ClientCredentials:
        let credentials: ClientCredentials = ClientCredentials::new(user, password);

        // Set credentials in ClientArgs:
        args.set_credentials(credentials);

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
        let mut client: Client = ClientArgs::new("remotehost").build().unwrap();

        // Verify the connection to the server:
        let result: RedsumerResult<()> = ping(&mut client);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Connection Verification Error - ClientError: The connection to the Redis server could not be verified. Please verify the client configuration or server availability");
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
