pub struct Settings {
    redis_host: String,
    redis_port: String,
    redis_db: String,
    stream_name: String,
}

impl Settings {
    pub fn get_redis_host(&self) -> &str {
        &self.redis_host
    }

    pub fn get_redis_port(&self) -> &str {
        &self.redis_port
    }

    pub fn get_redis_db(&self) -> &str {
        &self.redis_db
    }

    pub fn get_stream_name(&self) -> &str {
        &self.stream_name
    }

    pub fn get() -> Self {
        Self {
            redis_host: "localhost".to_string(),
            redis_port: "6379".to_string(),
            redis_db: "0".to_string(),
            stream_name: "produce-from-struct".to_string(),
        }
    }
}
