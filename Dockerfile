# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the source code into the container
COPY . .

# Build the microservice using the cargo command
RUN cargo build --release

# Create a new image for the microservice using the Debian base image
FROM debian:latest

# Install system dependencies required by the microservice (e.g., OpenSSL)
RUN apt-get update && apt-get install -y ca-certificates openssl && rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/porse .

# Expose the port that the microservice listens on (change it if necessary)
EXPOSE 8080

# Set the RUST_LOG environment variable
ENV RUST_LOG=debug

# Set the command to run the microservice when the container starts
CMD ["./porse"]
