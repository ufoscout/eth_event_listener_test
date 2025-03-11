use std::env;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct EthNode {
    pub wss_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    database: Database,
    eth_node: EthNode,
}

impl Settings {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(&format!("{path}/default")))
            // Add in the optional current environment file
            .add_source(
                File::with_name(&format!("{path}/{run_mode}"))
                    .required(false),
            )
            // Add in a local configuration file. This file shouldn't be checked in to git
            .add_source(File::with_name(&format!("{path}/local")).required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("app"))
            .build()?;

        s.try_deserialize()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    /// Tests that the configuration file can be read
    #[test]
    fn should_read_the_config_files() {
        // Act
        let conf = Settings::new("../../config").unwrap();

        // Assert
        assert_eq!("postgres://postgres@localhost", conf.database.url);

        println!("{}", conf.eth_node.wss_url)
    }

}