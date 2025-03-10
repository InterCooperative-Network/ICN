FROM rust:1.75 AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev protobuf-compiler && \
    rm -rf /var/lib/apt/lists/* && \
    rustup component add rustfmt clippy

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./

# Create dummy source files for all crates
RUN mkdir -p backend/src \
    crates/icn-types/src \
    crates/icn-common/src \
    crates/icn-core/src \
    crates/icn-p2p/src \
    crates/icn-consensus/src \
    crates/icn-federation/src \
    crates/icn-reputation/src && \
    touch backend/src/lib.rs backend/src/main.rs \
          crates/icn-types/src/lib.rs \
          crates/icn-common/src/lib.rs \
          crates/icn-core/src/lib.rs \
          crates/icn-p2p/src/lib.rs \
          crates/icn-consensus/src/lib.rs \
          crates/icn-federation/src/lib.rs \
          crates/icn-reputation/src/lib.rs

# Copy crate manifests
COPY backend/Cargo.toml backend/
COPY crates/icn-types/Cargo.toml crates/icn-types/
COPY crates/icn-common/Cargo.toml crates/icn-common/
COPY crates/icn-core/Cargo.toml crates/icn-core/
COPY crates/icn-p2p/Cargo.toml crates/icn-p2p/
COPY crates/icn-consensus/Cargo.toml crates/icn-consensus/
COPY crates/icn-federation/Cargo.toml crates/icn-federation/
COPY crates/icn-reputation/Cargo.toml crates/icn-reputation/

# Build dependencies only
RUN cargo build --release -p icn-backend || true

# Remove the dummy files
RUN rm -rf backend/src crates/*/src

# Now copy the real source code
COPY backend/src backend/src/
COPY crates/icn-types/src crates/icn-types/src/
COPY crates/icn-common/src crates/icn-common/src/
COPY crates/icn-core/src crates/icn-core/src/
COPY crates/icn-p2p/src crates/icn-p2p/src/
COPY crates/icn-consensus/src crates/icn-consensus/src/
COPY crates/icn-federation/src crates/icn-federation/src/
COPY crates/icn-reputation/src crates/icn-reputation/src/

# Build the release version
RUN cargo build --release -p icn-backend

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and create directories
COPY --from=builder /usr/src/app/target/release/icn-backend .
RUN mkdir -p /data /config /logs

# Copy default configuration
COPY config/log4rs.yaml /config/
COPY config/feature-flags.json /config/

ENV RUST_LOG=info
ENV NODE_TYPE=regular
ENV NODE_PORT=8081
ENV API_PORT=8081

EXPOSE 8081 9000-9002

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:${API_PORT}/health || exit 1

ENTRYPOINT ["./icn-backend"]
