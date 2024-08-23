/// Define the configuration parameters to create a producer instance.
#[derive(Debug, Clone)]
pub struct ProducerConfig<'d> {
    stream_name: &'d str,
}

impl<'d> ProducerConfig<'d> {
    /// Get **stream name**.
    ///
    /// # Arguments:
    /// - No arguments.
    ///
    /// # Returns:
    /// The stream name.
    pub fn get_stream_name(&self) -> &str {
        self.stream_name
    }

    /// Create a new [`ProducerConfig`] instance.
    ///
    /// # Arguments:
    /// - **stream_name**: The name of the stream where messages will be produced.
    ///
    /// # Returns:
    /// A new [`ProducerConfig`] instance.
    ///
    /// # Example:
    /// ```rust,no_run
    /// use redsumer::ProducerConfig;
    ///
    /// // Define the stream name.
    /// let stream_name = "my_stream";
    ///
    /// // Create a new ProducerConfig instance.
    /// let config: ProducerConfig = ProducerConfig::new(stream_name);
    /// ```
    pub fn new(stream_name: &'d str) -> Self {
        ProducerConfig { stream_name }
    }
}

#[cfg(test)]
mod test_producer_config {
    use super::*;

    #[test]
    fn test_producer_config_builder() {
        // Define the stream name.
        let stream_name: &str = "my_stream";

        // Create a new ProducerConfig instance.
        let config: ProducerConfig = ProducerConfig::new(stream_name);

        // Assert the stream name.
        assert_eq!(config.get_stream_name(), stream_name);
    }
}
