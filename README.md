![Build Status](https://github.com/ufoscout/eth_event_listener_test/actions/workflows/build_and_test.yml/badge.svg)

# Eth Event Listener

## Overview
This crate provides event listener that captures blockchain data and makes it accessible via RESTful APIs.
The current implementation is limited to ERC20 events.

## Limitations

The current implementation is limited to ERC20 events. It was tested only with the Infura endpoint for the mainnet node and the `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` token address.
In addition, despite the fact that the application is written to be platform independent, it was developed and tested only on an Ubuntu 24.04 x86_64 system.


## Usage

### Requirements

To build the application you need to have [Rust](https://www.rust-lang.org/) properly installed on your machine. The minimum supported version is 1.85.

To start the application or run the tests you need to have a [PostgreSQL](https://www.postgresql.org/) database running locally. The minimum supported version is 11. If you prefer to use Docker, you can start a local Postgres database by using the provided `docker-compose.yml` file with the `docker compose up db` command.

You need also to provide the URL of the Ethereum node to connect to in the `./config/default.toml` file or in the `APP__ETH_NODE__WSS_URL` environment variable. The application will establish a WebSocket connection to the node and subscribe to the events of the specified token address.


### Start the application using docker-compose

If you prefer using docker, you can build and start the application using it. In this case you don't need to have Rust configured in your machine nor the PostgreSQL database because everything will be provided by docker itself.

To start and automatically build the application using docker, use command:
```bash
INFURA_SECRET_KEY=<YOUR_INFURA_SECRET_KEY> docker compose up
```

Please note that by deafult you need to provide an Infura secret key as an environment variable, otherwise you can open the `docker-compose.yml` file and change the `APP__ETH_NODE__WSS_URL` environment variable value.


### Start the application using cargo

*Pre-requisites*: Before running the application, make sure you have a PostgreSQL database running locally. You can start it by running the `docker compose up db` command from the root of the repository.

The simplest way to start the application is by using the `cargo run` command:

```bash
APP__ETH_NODE__WSS_URL=wss://mainnet.infura.io/ws/v3/<YOUR_INFURA_SECRET_KEY> cargo run -p web
```

This will start a web server to the configured port (e.g. 3000 by default) and connect to the specified Ethereum node.

The `APP__ETH_NODE__WSS_URL` environment variable should be set to the URL of the Ethereum node WebSocket endpoint to connect to. It should be in the form of a WebSocket URL, for example: `wss://mainnet.infura.io/ws/v3/<YOUR_INFURA_SECRET_KEY>`. It is required to be an Infura endpoint, but the application was tested only with it.


### Start the application using the web executable

*Pre-requisites*: As in the previous case, before running the application, make sure you have a PostgreSQL database running locally.

The last option is to build the application using the `cargo build --release` command. This will produce the `./target/release/web` executable.
To start the application, copy the `web` executable and the `./config` folder to a common location. Then run the `./web` executable.


### Run the tests

To run the tests, use the `cargo test` command: 

```bash
APP__ETH_NODE__WSS_URL=wss://mainnet.infura.io/ws/v3/<YOUR_INFURA_SECRET_KEY> cargo test
```

The same requirements apply as in the previous section.


## Architecture

The architecture is based on the [hexagonal architecture](https://martinfowler.com/bliki/HexagonalArchitecture.html), where the `base` crate is the core of the application.
The main logic is implemented in the `base` crate, while the web server and RESTful APIs are implemented in the `web` crate.


### Base crate

The `base` crate is a library providing the main services for the Ethereum event listener. There are three Services:

* `SubscriberService`: This service is responsible for connecting to the Ethereum node and subscribing to the events of the specified token address.
* `StorageService`: This service is responsible for persisting and retrieving Ethereum events from a the database. It uses a PostgreSQL specific repository implementation and manages the database creation and updating at runtime.
* `Config`: This is responsible for reading and parsing the configuration file and the environment variables.

All services are indipendent from each other and loosely coupled.

The application configuration consists of a set of layers with priority:

1. `./config/default.toml`: this is the default configuration file. It should be in the same folder as the `web` executable when running locally.
2. `./config/local.toml`: this is the local configuration file that can be used for local development. It is not committed to the git repository.
   Values in this file will override the values in the `./config/default.toml` file.
3. Environment variables: These have the highest priority and override the values in the configuration files. They should have a prefix of `APP` and a separator of `__`. For example, the `APP__ETH_NODE__WSS_URL` will override the `eth_node.wss_url` value in the configuration files.



### Web crate   

The `web` crate is a web server that provides the RESTful APIs for the Ethereum event listener. It uses the services from the `base` crate to provide the main logic.

Currently, the web server provides a single endpoint for retrieving all the events from the database. It is accessible at the `/api/v1/logs` endpoint. It accepts the following query parameters:

- `from_id`: the ID of the first event to return. If not provided, the first event will be returned.
- `event_type`: the type of the event to return. If not provided, all events will be returned. Values are: `Transfer`, `Approve`, `Deposit`, `Withdrawal`.
- `max`: the maximum number of events to return. If not provided, the default value of 10 will be used. The maximum value is 100.

All parameters are optional and have a default value.

Example of a request using curl: 

```bash
curl -X GET "http://localhost:3000/api/v1/logs?from_id=1&event_type=Transfer&max=10"
```
