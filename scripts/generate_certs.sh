#!/usr/bin/env bash
set -euo pipefail

openssl req -new -text -passout pass:abcd -subj /CN=localhost -out keys/server.req -keyout keys/privkey.pem
openssl rsa -in keys/privkey.pem -passin pass:abcd -out keys/server.key
openssl req -x509 -in keys/server.req -text -key keys/server.key -out keys/server.crt

chmod 600 keys/server.key
test $(uname -s) == Linux && chown 999:999 keys/server.key
