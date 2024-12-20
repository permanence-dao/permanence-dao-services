#!/usr/bin/env bash
set -e

cd "${0%/*}" || exit # cd script directory
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "DROP DATABASE IF EXISTS pdao";
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -tc "SELECT 1 FROM pg_user WHERE usename = 'pdao'" | grep -q 1 ||  PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE USER pdao WITH ENCRYPTED PASSWORD 'pdao';"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE DATABASE pdao;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "GRANT ALL ON DATABASE pdao TO pdao;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "ALTER DATABASE pdao OWNER TO pdao;"
cd ../_migrations || exit
DATABASE_URL=postgres://pdao:pdao@127.0.0.1/pdao sqlx migrate run
