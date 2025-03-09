FROM node:18-slim as builder

WORKDIR /app

# Copy package files
COPY frontend/package*.json ./

# Install dependencies with legacy SSL provider for compatibility
ENV NODE_OPTIONS=--openssl-legacy-provider
RUN npm install --legacy-peer-deps

# Copy the rest of the application code
COPY frontend/ ./

# Build the application
RUN npm run build

# Production stage
FROM nginx:alpine

# Copy built files from builder stage
COPY --from=builder /app/build /usr/share/nginx/html

# Copy nginx configuration
RUN echo 'server { \
    listen 80; \
    server_name localhost; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    location /api/ { \
        proxy_pass http://backend:8081/api/; \
        proxy_http_version 1.1; \
        proxy_set_header Upgrade $http_upgrade; \
        proxy_set_header Connection "upgrade"; \
        proxy_set_header Host $host; \
        proxy_set_header X-Real-IP $remote_addr; \
    } \
    location = /health { \
        access_log off; \
        add_header Content-Type text/plain; \
        return 200 "healthy\n"; \
    } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]