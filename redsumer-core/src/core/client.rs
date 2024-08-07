use redis::Client;
use redis::{ConnectionAddr, ConnectionInfo, RedisConnectionInfo};

#[allow(unused_imports)]
use crate::types::{CommunicationProtocol, RedsumerError, RedsumerResult};

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
    protocol: CommunicationProtocol,
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

    /// Get *protocol*.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The protocol version to connect to Redis server.
    pub fn get_protocol(&self) -> CommunicationProtocol {
        self.protocol
    }

    /// Create a new instance of [`ClientArgs`].
    ///
    /// This function is used to create a new instance of [`ClientArgs`] with specific *host*, *port*, *db* and *protocol*. The *credentials* are optional.
    ///
    /// # Arguments:
    /// - **credentials**: Credentials to authenticate in Redis.
    /// - **host**: Host to connect to Redis.
    /// - **port**: Redis server port.
    /// - **db**: Redis database
    /// - **protocol**: Redis protocol version to communicate with the server.
    ///
    /// # Returns:
    /// A new instance of [`ClientArgs`].
    pub fn new(
        credentials: Option<ClientCredentials<'k>>,
        host: &'a str,
        port: u16,
        db: i64,
        protocol: CommunicationProtocol,
    ) -> ClientArgs<'k, 'a> {
        ClientArgs {
            credentials,
            host,
            port,
            db,
            protocol,
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
            protocol: self.get_protocol(),
        };

        Client::open(ConnectionInfo { addr, redis })
    }
}

#[cfg(test)]
mod test_client_parameters {
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
        let protocol_version: CommunicationProtocol = CommunicationProtocol::RESP2;

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

#[cfg(test)]
mod test_redis_client_builder {
    use super::*;

    #[test]
    fn test_redis_client_builder_ok() {
        // Create a new instance of ClientArgs with default port and db:
        let args: ClientArgs =
            ClientArgs::new(None, "mylocalhost", 6377, 16, CommunicationProtocol::RESP2);

        // Build a new instance of Client:
        let client_result: RedsumerResult<Client> = args.build();

        // Verify if the client is correct:
        assert!(client_result.is_ok());
    }
}
