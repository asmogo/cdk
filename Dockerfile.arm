# Stage 1: Build the application
FROM rust:1.86.0 as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    curl \
    protobuf-compiler \
    libz-dev \
    # Install just from a release tarball
    && curl -LSfs https://raw.githubusercontent.com/casey/just/master/install.sh | bash -s -- --to /usr/local/bin \
    && rm -rf /var/lib/apt/lists/*

# Install wasm toolchain
RUN rustup target add wasm32-unknown-unknown

# Install cargo tools
RUN cargo install sqlx-cli \
    && cargo install cargo-outdated

# Set working directory
WORKDIR /usr/src/app

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Create the production image
FROM debian:stable-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libz-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/cdk /usr/local/bin/cdk

# Set the entrypoint
ENTRYPOINT ["cdk"]
