FROM rust:latest as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY crates ./crates

# Build the application
WORKDIR /usr/src/app/backend
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/backend/target/release/icn-backend .

EXPOSE 8081

CMD ["./icn-backend"]
