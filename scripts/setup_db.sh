#!/bin/bash
set -e

echo "Setting up PostgreSQL for ICN..."

# Start PostgreSQL service
sudo service postgresql start

# Create user and database
sudo -u postgres psql -c "CREATE USER icn_user WITH PASSWORD 'development_password';"
sudo -u postgres psql -c "CREATE DATABASE icn_db OWNER icn_user;"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE icn_db TO icn_user;"

# Allow local connections
sudo sed -i 's/local   all             all                                     peer/local   all             all                                     md5/' /etc/postgresql/13/main/pg_hba.conf
sudo service postgresql restart

echo "Database setup complete!" 