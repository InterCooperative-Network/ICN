FROM node:18-slim as builder

WORKDIR /app

# Copy package files
COPY frontend/package*.json ./

# Install dependencies with legacy peer deps to bypass dependency conflicts
RUN npm ci --legacy-peer-deps

# Set Node to use legacy OpenSSL provider
ENV NODE_OPTIONS=--openssl-legacy-provider

# Copy configuration files
COPY frontend/tsconfig.json ./
COPY frontend/craco.config.js ./
COPY frontend/tailwind.config.js ./
COPY frontend/postcss.config.js ./
COPY frontend/public ./public
COPY frontend/src ./src

# Install babel plugin for module resolution 
# This will help resolve the @/ path aliases in the TypeScript code
RUN npm install --save-dev babel-plugin-module-resolver --legacy-peer-deps

# Create a simple babel plugin to transform @/ imports to ./src/ imports
RUN echo '{\
  "plugins": [\
    ["module-resolver", {\
      "alias": {\
        "@": "./src"\
      }\
    }]\
  ]\
}' > .babelrc

# Create a temporary file that React scripts won't override our path aliases in tsconfig.json
RUN echo "// This file prevents React scripts from modifying tsconfig.json paths" > src/react-app-env.d.ts

# Build the application
RUN npm run build

# Production stage with Nginx
FROM nginx:alpine

# Copy built assets from builder stage
COPY --from=builder /app/build /usr/share/nginx/html

# Create a default nginx configuration
RUN echo 'server { \
    listen 80; \
    server_name localhost; \
    root /usr/share/nginx/html; \
    index index.html index.htm; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    location /api/ { \
        proxy_pass http://backend:8081/api/; \
        proxy_set_header Host $host; \
        proxy_set_header X-Real-IP $remote_addr; \
    } \
    # Add a health check endpoint for Docker health checks\
    location /health { \
        access_log off; \
        return 200 "healthy\n"; \
    }\
}' > /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]