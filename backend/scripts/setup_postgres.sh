#!/bin/bash
set -e

# Start PostgreSQL service
sudo service postgresql start

# Create user and database
sudo -u postgres psql -c "CREATE USER postgres WITH SUPERUSER PASSWORD 'postgres';"
sudo -u postgres psql -c "CREATE DATABASE icn OWNER postgres;"

# Update pg_hba.conf to use md5 authentication
sudo sed -i 's/peer/md5/g' /etc/postgresql/*/main/pg_hba.conf
sudo sed -i 's/scram-sha-256/md5/g' /etc/postgresql/*/main/pg_hba.conf

# Restart PostgreSQL to apply changes
sudo service postgresql restart

echo "PostgreSQL setup completed successfully" 