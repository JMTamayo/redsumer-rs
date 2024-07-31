pub use redis::ProtocolVersion;

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
    pub fn get_user(&self) -> &str {
        self.user
    }

    /// Get *password*
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *password* to authenticate in Redis.
    pub fn get_password(&self) -> &str {
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
    protocol_version: ProtocolVersion,
}

impl<'k, 'a> ClientArgs<'k, 'a> {
    /// Get *credentials*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *credentials* to authenticate in Redis.
    pub fn get_credentials(&self) -> &Option<ClientCredentials> {
        &self.credentials
    }

    /// Get *host*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *host* to connect to Redis.
    pub fn get_host(&self) -> &str {
        self.host
    }

    /// Get *port*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The *port* to connect to Redis.
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Get *db*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The database to connect to Redis.
    pub fn get_db(&self) -> i64 {
        self.db
    }

    pub fn get_protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
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
    /// - **protocol_version**: Redis protocol version.
    ///
    /// # Returns:
    /// A new instance of [`ClientArgs`].
    ///
    /// # Example:
    /// ```rust,no_run
    /// use redsumer::{ClientArgs, ClientCredentials, ProtocolVersion};
    ///
    /// let args = ClientArgs::new(
    ///     Some(ClientCredentials::new("user", "password")),
    ///     "localhost",
    ///     6379,
    ///     0,
    /// 	ProtocolVersion::RESP2
    /// );
    /// ```
    pub fn new(
        credentials: Option<ClientCredentials<'k>>,
        host: &'a str,
        port: u16,
        db: i64,
        protocol_version: ProtocolVersion,
    ) -> ClientArgs<'k, 'a> {
        ClientArgs {
            credentials,
            host,
            port,
            db,
            protocol_version,
        }
    }
}

#[cfg(test)]
mod test_client {
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

        // Define the redis protocol version:
        let protocol_version: ProtocolVersion = ProtocolVersion::RESP2;

        // Create a new instance of ClientArgs with default port and db:
        let args: ClientArgs = ClientArgs::new(Some(credentials), host, port, db, protocol_version);

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
}
