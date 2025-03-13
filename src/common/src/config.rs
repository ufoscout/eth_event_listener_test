use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

/// Settings for the database connection
#[derive(Debug, Deserialize)]
pub struct Database {
    /// The database username
    pub username: String,
    /// The database password
    pub password: String,
    /// The database name
    pub database: String,
    /// The database host
    pub host: String,
    /// The database port
    pub port: u16,
    /// The maximum number of database connections
    pub max_connections: u32,
}

/// Settings for the Ethereum node connection and the token address
/// for the subscription
#[derive(Debug, Deserialize)]
pub struct EthNode {
    pub timeout_seconds: u64,
    pub token_address: String,
    pub wss_url: String,
}

/// Settings for the local web server
#[derive(Debug, Deserialize)]
pub struct Server {
    pub port: u16,
    pub address: String,
}

/// Settings for the application
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    /// Sets the logger [`EnvFilter`].
    /// Valid values: trace, debug, info, warn, error
    /// Example of a valid filter: "warn,my_crate=info,my_crate::my_mod=debug,[my_span]=trace".
    pub log_filter: String,
    /// Database settings
    pub database: Database,
    /// Ethereum node settings
    pub eth_node: EthNode,
    /// Server settings
    pub server: Server,
}

impl Settings {
    /// Read the configuration data from the specified path and build the Settings
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        Config::builder()
            // Mandatory configuration file
            .add_source(File::with_name(&format!("{path}/default")))
            // Optional configuration file, used for local development
            .add_source(File::with_name(&format!("{path}/local")).required(false))
            // Read settings from the environment (with a prefix of APP and '__' as separator)
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
