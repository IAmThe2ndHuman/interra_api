# Leveraging the pre-built Docker images with 
# cargo-chef and the Rust toolchain
#FROM lukemathwalker/cargo-chef:latest-rust-1.66.0 AS chef
#WORKDIR interra_api
#
#FROM chef AS planner
#COPY . .
#RUN cargo chef prepare --recipe-path recipe.json
#
#FROM chef AS builder
#COPY --from=planner /interra_api/recipe.json recipe.json
## Build dependencies - this is the caching Docker layer!
#RUN cargo chef cook --release --recipe-path recipe.json
## Build application
#COPY . .
#RUN cargo build --release --bin interra_api
#
## We do not need the Rust toolchain to run the binary!
#FROM debian:bullseye-slim AS runtime
#WORKDIR interra_api
#COPY --from=builder /interra_api/target/release/interra_api /usr/local/bin
#ENTRYPOINT ["/usr/local/bin/interra_api"]

# FROM alpine:latest
# COPY ./please .
# EXPOSE 80
# RUN ["chmod", "+x", "./please"]
# ENTRYPOINT ["./please"]

# FROM rust:1.72.0 as builder
# WORKDIR /usr/src/interra_api
# COPY . .
# RUN cargo install --path .
# FROM debian:buster-slim
# RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
# COPY --from=builder /usr/local/cargo/bin/interra_api /usr/local/bin/interra_api
# CMD ["interra_api"]

# Use the official Rust image as the base image
FROM rust:1.72

# Set the working directory in the container
WORKDIR /interra_api

# Copy the application files into the working directory
COPY . /interra_api

# Build the application
RUN cargo build --release

# Expose port 8080
EXPOSE 80

# Define the entry point for the container
CMD ["./target/release/interra_api"]
