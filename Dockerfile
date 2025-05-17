# Stage 1: Build Stage
FROM rust:1.83-alpine AS builder

# Set the working directory
WORKDIR /app

# Install dependencies
RUN apk add --no-cache \
    pkgconf \
    postgresql-dev \
    gcc \
    musl-dev \
    make \
    openssl-dev # Add openssl-dev for building if needed by any crate

# Copy only dependency files first (to leverage caching)
COPY Cargo.toml Cargo.lock ./

# Create a temporary empty src/lib.rs to allow `cargo fetch` to run
RUN mkdir -p src && echo '' > src/lib.rs

# Fetch and compile dependencies only (ensuring cache reuse)
RUN cargo fetch

# Remove the temporary file before copying the actual source code
RUN rm -rf src

# Copy the actual source code separately
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Stage 2: Runtime Stage
FROM alpine:latest

# Install necessary runtime dependencies
# Corrected openssl package name
RUN apk add --no-cache postgresql-libs openssl

# Set the working directory
WORKDIR /app

# Copy only the built binary from the builder stage
# IMPORTANT: Ensure the binary name matches your project name (pub-sub-rs)
COPY --from=builder /app/target/release/pub-sub-rs ./pub-sub-rs

# Expose the application port (if applicable)
EXPOSE 8000

# Run the application
# IMPORTANT: Ensure the command matches your binary name (pub-sub-rs)
CMD ["./pub-sub-rs"]