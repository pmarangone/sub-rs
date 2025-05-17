# Stage 1: Builder
FROM rust:latest as builder

WORKDIR /app

# Copy the Cargo.toml and Cargo.lock first to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to build dependencies and cache them
RUN mkdir src/
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src/

# Copy the rest of your source code
COPY src ./src

# Build the final release binary
RUN cargo build --release

# Stage 2: Runner
# Use a more recent Debian-based image with a newer GLIBC
FROM debian:bookworm-slim

# Install any necessary runtime dependencies
# Based on your dependencies (postgres, lapin), you might need these.
# Adjust based on your actual application's runtime requirements.
# libpq5 is the client library for PostgreSQL
RUN apt-get update && \
    apt-get install -y libpq5 openssl pkg-config && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/pub-sub-rs ./

# Set the entrypoint to run your application
CMD ["./pub-sub-rs"]