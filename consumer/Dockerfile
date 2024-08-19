ARG RUST_VERSION

FROM rust:${RUST_VERSION}-bullseye as builder

# Set the working directory in the container
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

# Use a Debian-based image for the runtime that matches your host architecture
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/consumer /usr/local/bin/consumer

# Set the binary as the entrypoint
ENTRYPOINT ["consumer"]