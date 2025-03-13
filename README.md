![Build Status](https://github.com/ufoscout/eth_event_listener_test/actions/workflows/build_and_test.yml/badge.svg)

# Eth Event Listener

## Overview
This crate provides event listener that captures blockchain data and makes it accessible via RESTful APIs.
The current implementation is limited to ERC20 events.


## Requirements

### Build the application
To build the application you need to have [Rust](https://www.rust-lang.org/) properly installed on your machine. The minimum supported version is 1.85.

### Run the application
To start the application or run the tests you need to have a [PostgreSQL](https://www.postgresql.org/) database running locally. The minimum supported version is 11. If you prefer to use Docker, you can start a local Postgres database by using the provided `docker-compose.yml` file with the `docker compose up db` command.

You need also to provide the URL of the Ethereum node to connect to in the `./config/default.toml` file or in the `APP__ETH_NODE__WSS_URL` environment variable. The application will establish a WebSocket connection to the node and subscribe to the events of the specified token address.

### Limitations

The current implementation is limited to ERC20 events. It was tested only with the Infura endpoint for the mainnet node and the `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` token address.
In addition, the application was developed and tested only on an Ubuntu 24.04 x86_64 system.


## Usage

### Pre-requisites

Before running the application, make sure you have a PostgreSQL database running locally. You can start it by running the `docker compose up db` command from the root of the repository.

### Start the application using cargo

The simplest way to start the application is by using the `cargo run` command:

```bash
APP__ETH_NODE__WSS_URL=wss://mainnet.infura.io/ws/v3/<YOUR_INFURA_SECRET_KEY> cargo run -p web
```

This will start a web server to the configured port (e.g. 3000 by default) and connect to the specified Ethereum node.

The `APP__ETH_NODE__WSS_URL` environment variable should be set to the URL of the Ethereum node WebSocket endpoint to connect to. It should be in the form of a WebSocket URL, for example: `wss://mainnet.infura.io/ws/v3/<YOUR_INFURA_SECRET_KEY>`. It is required to be an Infura endpoint, but the application was tested only with it.


### Start the application using docker-compose

You can also build and start the application using docker. In this case you don't need to have Rust configured in your machine because it will be provided by a builder docker image.

To start the application using docker, by deafult you need to provide an Infura secret key as an environment variable:
```bash
INFURA_SECRET_KEY=<YOUR_INFURA_SECRET_KEY> docker compose up
```

otherwise you can open the `docker-compose.yml` file and change the `APP__ETH_NODE__WSS_URL` environment variable value to the one you need.


### Start the application using the web executable

The last option is to build the application using the `cargo build --release` command. This will produce the `./target/release/web` executable.
To start the application, copy the `web` executable and the `./config` folder to a common location. Then run the `./web` executable.
