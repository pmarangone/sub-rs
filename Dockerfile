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
# Use a minimal Debian-based image
FROM debian:bullseye-slim

# Install any necessary runtime dependencies
# Based on your dependencies (postgres, lapin), you might need these.
# Adjust based on your actual application's runtime requirements.
RUN apt-get update && \
    apt-get install -y libpq-dev openssl pkg-config && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/pub-sub-rs ./

# Set the entrypoint to run your application
CMD ["./pub-sub-rs"]
