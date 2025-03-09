#!/bin/sh
set -e

# Get the API URL from environment or use default
API_URL=${API_URL:-http://backend:8081/api}

# Replace API URL placeholders in JavaScript files
find /usr/share/nginx/html -type f -name "*.js" -exec sed -i "s|REACT_APP_API_URL|${API_URL}|g" {} \;
find /usr/share/nginx/html -type f -name "*.js" -exec sed -i "s|\"__API_URL__\"|\"${API_URL}\"|g" {} \;

echo "Frontend configured with API_URL=$API_URL"

# Execute the main command
exec "$@"