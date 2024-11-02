# Start from an official Rust image
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the current directory contents into the container at /app
COPY . .

# Install any needed dependencies
RUN cargo build --release

# Add integration tests step to Dockerfile
RUN cargo test --release

# Run the binary when the container launches
CMD ["cargo", "run", "--release"]
