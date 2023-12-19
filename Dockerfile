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
