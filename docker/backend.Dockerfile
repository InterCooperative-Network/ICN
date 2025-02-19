# Use specific Rust version
FROM rust:1.84.1

# Set work directory
WORKDIR /usr/src/app

# Upgrade Rust & Cargo
RUN rustup update

# Copy entire workspace, including backend, crates, and Cargo files
COPY . .

# Build the application
WORKDIR /usr/src/app/backend
RUN cargo build --release

EXPOSE 8081
CMD ["./target/release/icn-backend"]
