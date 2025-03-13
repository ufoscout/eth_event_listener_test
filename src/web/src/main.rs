use c3p0::sqlx::SqlxPgC3p0Pool;
use common::{config::Settings, storage, subscriber};
use log::info;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::sync::Arc;
use web::app::create_app;

/// Main entry point of the web server
///
/// This function initializes the configuration, logger, services and starts the web server.
/// It expects the configuration file to be located in the `config` directory.
#[tokio::main]
async fn main() {
    // Read Settings
    let settings = Settings::new("./config").expect("Failed to read config");

    // Initialize logger
    init_logger(&settings.log_filter).expect("Failed to initialize logger");

    info!("Starting the web server...");

    // Initialize the services
    let log_provider = {
        let subscriber_service = subscriber::service::SubscriberService::new(
            settings.eth_node.wss_url,
            settings.eth_node.timeout_seconds,
            settings.eth_node.token_address.parse().unwrap(),
        );

        // Connect to the database
        let options = PgConnectOptions::new()
            .username(&settings.database.username)
            .password(&settings.database.password)
            .database(&settings.database.database)
            .host(&settings.database.host)
            .port(settings.database.port);

        let pool = PgPoolOptions::new()
            .max_connections(settings.database.max_connections)
            .connect_with(options)
            .await
            .expect("Failed to create database connection pool");

        let storage_service = storage::service::StorageService::new(SqlxPgC3p0Pool::new(pool))
            .await
            .expect("Failed to initialize storage service");

        // Subscribe to Ethereum events from a remote node
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let run_until = Arc::new(std::sync::atomic::AtomicBool::new(true));
        subscriber_service.subscribe_to(sender, run_until.clone()).await.expect("Failed to subscribe to Ethereum logs");

        // Subscribe the storage service to internal event stream
        storage_service.subscribe_to_event_stream(receiver);
        storage_service
    };

    let app = create_app(Arc::new(log_provider));
    let address = format!("{}:{}", settings.server.address, settings.server.port);
    info!("Starting the server on {}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.expect("Failed to start Axum server");
}

/// Initializes the logger
fn init_logger(logger_filter: &str) -> Result<(), log::SetLoggerError> {
    env_logger::Builder::new().parse_filters(logger_filter).try_init()
}
