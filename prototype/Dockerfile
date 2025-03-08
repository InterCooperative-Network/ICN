FROM node:18-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Create directory structure for ICN
RUN mkdir -p config data/keys data/storage data/workloads logs

# Copy package files and install dependencies
COPY package.json ./
RUN npm install

# Copy application code
COPY src/ ./src/
COPY config/ ./config/
COPY scripts/ ./scripts/

# Make scripts executable
RUN chmod +x ./scripts/*.sh

# Expose ports for P2P networking and API
EXPOSE 9000
EXPOSE 3000

# Set environment variables with defaults
ENV NODE_TYPE=regular \
    NODE_PORT=9000 \
    API_PORT=3000 \
    BOOTSTRAP_NODES="[]" \
    COOPERATIVE_ID="icn-prototype" \
    COOPERATIVE_TIER="contributor" \
    LOG_LEVEL="info"

# Volumes for persistence
VOLUME ["/app/data", "/app/logs"]

# Start the node
ENTRYPOINT ["./scripts/entrypoint.sh"]
CMD ["node", "src/index.js"]