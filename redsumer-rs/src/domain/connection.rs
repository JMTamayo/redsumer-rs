use std::fmt::{Debug, Formatter, Result as FmtResult};

/// Redis serialization protocol version used to communicate with the broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommunicationProtocol {
    /// RESP2 protocol (default).
    #[default]
    Resp2,
    /// RESP3 protocol.
    Resp3,
}

/// Credentials to authenticate against the message broker.
#[derive(Clone)]
pub struct Credentials {
    user: String,
    password: String,
}

impl Credentials {
    /// Build a new [`Credentials`] instance.
    ///
    /// # Arguments:
    /// - **user**: Username to authenticate.
    /// - **password**: Password to authenticate.
    ///
    /// # Returns:
    /// A new [`Credentials`] instance.
    pub fn new(user: &str, password: &str) -> Self {
        Credentials {
            user: user.to_owned(),
            password: password.to_owned(),
        }
    }

    pub(crate) fn get_user(&self) -> &str {
        &self.user
    }

    pub(crate) fn get_password(&self) -> &str {
        &self.password
    }
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Credentials")
            .field("user", &self.user)
            .field("password", &"****")
            .finish()
    }
}

/// Connection settings to reach the message broker.
///
/// # Example
///
/// ```
/// use redsumer::types::{CommunicationProtocol, ConnectionConfig};
///
/// let local = ConnectionConfig::new(None, "localhost", 6379, 0);
/// let with_resp3 = ConnectionConfig::new(None, "localhost", 6379, 0)
///     .with_protocol(CommunicationProtocol::Resp3);
/// ```
///
/// Supported URL format: `[<user>][:<password>@]<host>:<port>/<db>`
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    credentials: Option<Credentials>,
    host: String,
    port: u16,
    db: i64,
    protocol: CommunicationProtocol,
}

impl ConnectionConfig {
    /// Get *credentials*.
    pub fn get_credentials(&self) -> &Option<Credentials> {
        &self.credentials
    }

    /// Get *host*.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Get *port*.
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Get *db*.
    pub fn get_db(&self) -> i64 {
        self.db
    }

    /// Get the communication protocol.
    pub fn get_protocol(&self) -> CommunicationProtocol {
        self.protocol
    }

    /// Create a new [`ConnectionConfig`] instance.
    ///
    /// Uses [`CommunicationProtocol::Resp2`] by default.
    ///
    /// # Arguments:
    /// - **credentials**: Credentials to authenticate. Use `None` when authentication is not required.
    /// - **host**: Host to connect to.
    /// - **port**: Server port.
    /// - **db**: Database number.
    ///
    /// # Returns:
    /// A new [`ConnectionConfig`] instance.
    pub fn new(credentials: Option<Credentials>, host: &str, port: u16, db: i64) -> Self {
        ConnectionConfig {
            credentials,
            host: host.to_owned(),
            port,
            db,
            protocol: CommunicationProtocol::default(),
        }
    }

    /// Set the communication protocol for this connection.
    pub fn with_protocol(mut self, protocol: CommunicationProtocol) -> Self {
        self.protocol = protocol;
        self
    }
}

#[cfg(test)]
mod test_connection_config {
    use super::*;

    #[test]
    fn test_default_protocol_is_resp2() {
        let config = ConnectionConfig::new(None, "localhost", 6379, 0);

        assert_eq!(config.get_protocol(), CommunicationProtocol::Resp2);
    }

    #[test]
    fn test_with_protocol() {
        let config = ConnectionConfig::new(None, "localhost", 6379, 0)
            .with_protocol(CommunicationProtocol::Resp3);

        assert_eq!(config.get_protocol(), CommunicationProtocol::Resp3);
    }
}
