#!/bin/bash
HOST_IP=${HOST_IP:-"127.0.0.1"}
HOST_PUBLIC_API_PORT=${HOST_PUBLIC_API_PORT:-"8200"}
SIMPLE_CONFIG_PATH=${SIMPLE_CONFIG_PATH:-"config.toml"}
SIMPLE_DB_PATH=${SIMPLE_DB_PATH:-"data/db"}

# Runs command
${1:-$SIMPLE_BIN_PATH} run --node-config $SIMPLE_CONFIG_PATH --db-path $SIMPLE_DB_PATH --public-api-address $HOST_IP:$HOST_PUBLIC_API_PORT
