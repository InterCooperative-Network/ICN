# backend_new.Dockerfile

# Stage 1: Build the Rust app
FROM rust:1.85.0 AS builder

WORKDIR /app

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src

# Copy the rest of the project
COPY . .

# Build the project
RUN cargo build --release
RUN cargo test --release

# Stage 2: Create a smaller image to run the binary
FROM debian:bullseye-slim AS runner
WORKDIR /app
COPY --from=builder /app/target/release/icn-backend /usr/local/bin/icn-backend

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Expose only the necessary ports
EXPOSE 8081  # WebSocket port for backend

CMD ["icn-backend"]
