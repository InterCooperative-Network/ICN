# Use Node.js base image
FROM node:23-slim

# Set working directory
WORKDIR /usr/src/app

# Copy everything from frontend directory
COPY frontend/package.json frontend/package-lock.json ./

# Install dependencies
RUN npm install

# Copy the rest of the frontend source code
COPY frontend/. ./

# Expose the frontend port
EXPOSE 3000

# Start the frontend app
CMD ["npm", "start"]
