FROM node:18-slim as builder

WORKDIR /app

# Copy package files and install dependencies using package-lock.json for deterministic builds
COPY frontend/package*.json ./
RUN npm ci

# Copy configuration files
COPY frontend/tsconfig.json ./
COPY frontend/tailwind.config.js ./
COPY frontend/postcss.config.js ./

# Copy source code
COPY frontend/public ./public
COPY frontend/src ./src

# Set API URL for production build
ARG REACT_APP_API_URL=http://localhost:8081/api
ENV REACT_APP_API_URL=$REACT_APP_API_URL

# Build the application
RUN npm run build

# Production stage with Nginx
FROM nginx:alpine

# Copy built assets from builder stage
COPY --from=builder /app/build /usr/share/nginx/html

# Copy nginx config (enhanced with proxy configuration)
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf

# Add startup script to dynamically configure the frontend
COPY docker/frontend/docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

# Health check to verify Nginx is serving content
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD wget -q -O- http://localhost:80/health || exit 1

EXPOSE 80

ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["nginx", "-g", "daemon off;"]
