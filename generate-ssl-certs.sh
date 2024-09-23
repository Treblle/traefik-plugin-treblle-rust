#!/bin/sh
mkdir -p certs

openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout certs/local.key -out certs/local.crt \
    -subj "/CN=localhost" \
    -addext "subjectAltName = DNS:localhost,DNS:consumer,DNS:consumer.treblle-network,DNS:treblle-api,DNS:treblle-api.treblle-network"
