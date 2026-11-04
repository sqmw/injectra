#!/bin/sh
set -eu
: "${APP_EXECUTABLE:?WorkBuddy executable path}"
: "${RUN_DIR:?Independent external run directory}"
: "${CDP_PORT:?Unused loopback debugging port}"
mkdir -p "$RUN_DIR/config" "$RUN_DIR/profile"
exec env WORKBUDDY_CONFIG_DIR="$RUN_DIR/config" WORKBUDDY_USER_DATA_DIR="$RUN_DIR/profile" \
  "$APP_EXECUTABLE" "--remote-debugging-port=$CDP_PORT" --remote-debugging-address=127.0.0.1 \
  "--user-data-dir=$RUN_DIR/profile" --no-first-run
