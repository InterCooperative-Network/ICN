#!/bin/bash
set -e
sudo service postgresql start
sudo -u postgres psql -c "CREATE USER postgres WITH SUPERUSER PASSWORD 'postgres';"
