# backend.Dockerfile

# Stage 1: Build the Rust app
FROM rust:1.75-slim

WORKDIR /usr/src/app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

EXPOSE 8081

CMD ["cargo", "run", "--release"]
