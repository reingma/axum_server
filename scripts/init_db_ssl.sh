#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi
if [ -x "$(ldconfig -p | grep -c libpq)" ]; then
    echo >&2 "Error: libpq is not installed."
    exit 1
fi
if ! [ -x "$(command -v diesel)" ]; then
    echo >&2 "Error: diesel is not installed."
    echo >&2 "Use:"
    echo >&2 "  cargo install diesel --no-default-features --features postgres"
    echo >&2 "to install it."
    exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"


if [[ -z "${SKIP_DOCKER}" ]]
then
    docker run -d \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -v "$PWD/keys/server.crt:/var/lib/postgresql/server.crt:ro" \
        -v "$PWD/keys/server.key:/var/lib/postgresql/server.key:ro" \
        -d postgres \
        postgres -N 1000\
        -c ssl=on \
        -c ssl_cert_file=/var/lib/postgresql/server.crt \
        -c ssl_key_file=/var/lib/postgresql/server.key
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "postgres" -c '\q'; do
    >&2 echo "POSTGRES is still unavailable - sleeping"
    sleep 1
done
>&2 echo "POSTGRES is available at port ${DB_PORT}!"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
diesel setup
diesel migration run
>&2 echo "Postgres has been migrated, ready to go."
