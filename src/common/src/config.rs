use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub username: String,
    pub password: String,
    pub database: String,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct EthNode {
    pub timeout_seconds: u64,
    pub token_address: String,
    pub wss_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub port: u16,
    pub address: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    /// Sets the logger [`EnvFilter`].
    /// Valid values: trace, debug, info, warn, error
    /// Example of a valid filter: "warn,my_crate=info,my_crate::my_mod=debug,[my_span]=trace".
    pub log_filter: String,

    pub database: Database,
    pub eth_node: EthNode,
    pub server: Server,
}

impl Settings {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(&format!("{path}/default")))
            // Add in a local configuration file. This file shouldn't be checked in to git
            .add_source(File::with_name(&format!("{path}/local")).required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    pub fn get_settings() -> Settings {
        let settings = Settings::new("../../config").expect("Failed to read config");
        let _ = env_logger::Builder::new().parse_filters(&settings.log_filter).try_init();
        settings
    }

    /// Tests that the configuration file can be read
    #[test]
    fn should_read_the_config_files() {
        // Act
        let conf = get_settings();

        // Assert
        assert_eq!(5432, conf.database.port);
    }
}
