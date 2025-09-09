# Use Rust nightly base image to support edition2024 crates
FROM rustlang/rust:nightly-slim AS builder

# Install build dependencies including protoc compiler
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Ensure nightly is the default
RUN rustup default nightly

# Set the working directory
WORKDIR /usr/src/app

# Copy workspace files and crates directory into the container
COPY Cargo.toml ./Cargo.toml
COPY crates ./crates

# Build with cargo (feature flags provided at build time)
ARG CARGO_FEATURES="sqlite"
ENV RUSTFLAGS="-C target-cpu=native"

# Use CROSS_COMPILE env to speed up multi-arch builds without QEMU
ARG CARGO_BUILD_TARGET=""
ENV CARGO_BUILD_TARGET=${CARGO_BUILD_TARGET}

RUN if [ "$CARGO_BUILD_TARGET" != "" ]; then \
      echo "Cross compiling for target=$CARGO_BUILD_TARGET with features=$CARGO_FEATURES"; \
      rustup target add $CARGO_BUILD_TARGET; \
      cargo build --target $CARGO_BUILD_TARGET --release --bin cdk-mintd --features ${CARGO_FEATURES}; \
    else \
      echo "Native build with features=$CARGO_FEATURES"; \
      cargo build --release --bin cdk-mintd --features ${CARGO_FEATURES}; \
    fi

# Separate targets for multi-image publishing
# minimal: sqlite only
# base: sqlite + postgres + redis
# standalone: sqlite + postgres + redis + ldk-node
# cloud: sqlite + postgres + prometheus + redis + grpc + ldk-node

# Example build targets:
# docker build -t cdk-mintd-minimal --build-arg CARGO_FEATURES=sqlite .
# docker build -t cdk-mintd --build-arg CARGO_FEATURES="sqlite,postgres,redis" .
# docker build -t cdk-mintd-standalone --build-arg CARGO_FEATURES="sqlite,postgres,redis,ldk-node" .
# docker build -t cdk-mintd-cloud --build-arg CARGO_FEATURES="sqlite,postgres,prometheus,redis,grpc,ldk-node" .

# Create a runtime stage
FROM debian:trixie-slim

# Set the working directory
WORKDIR /usr/src/app

# Install needed runtime dependencies (if any)
RUN apt-get update && \
    apt-get install -y --no-install-recommends patchelf && \
    rm -rf /var/lib/apt/lists/*

# Copy the built application from the build stage
COPY --from=builder /usr/src/app/target/release/cdk-mintd /usr/local/bin/cdk-mintd

# Detect the architecture and set the interpreter accordingly
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "aarch64" ]; then \
        patchelf --set-interpreter /lib/ld-linux-aarch64.so.1 /usr/local/bin/cdk-mintd; \
    elif [ "$ARCH" = "x86_64" ]; then \
        patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 /usr/local/bin/cdk-mintd; \
    else \
        echo "Unsupported architecture: $ARCH"; exit 1; \
    fi

# Set the entry point for the container
ENTRYPOINT ["cdk-mintd"]

# Cloud variant requires extra system libs for prometheus/grpc support
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      libssl3 \
      && rm -rf /var/lib/apt/lists/*
