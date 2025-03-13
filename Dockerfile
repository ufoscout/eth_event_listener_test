# syntax=docker.io/docker/dockerfile:1.7-labs

# A multi stage dockerfile to build and deploy the rust application

# Stage 1: Build the application
FROM rust:1.85 AS builder
WORKDIR /app
COPY ./src ./src
COPY --exclude=*local.toml ./config ./config
COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release


# Stage 2: Deploy the application
FROM ubuntu:24.04

WORKDIR /app

COPY --from=ghcr.io/ufoscout/docker-compose-wait:latest /wait /wait
COPY --from=builder /app/target/release/web .
COPY --from=builder /app/config ./config

EXPOSE 3000

CMD /wait && ./web