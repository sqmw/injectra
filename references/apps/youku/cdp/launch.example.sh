#!/bin/sh
set -eu
: "${APP_EXECUTABLE:?Youku executable path}"
: "${RUN_DIR:?New independent directory inside Youku container caches}"
: "${CDP_PORT:?Unused loopback debugging port}"
case "$RUN_DIR/" in
  "$HOME/Library/Containers/com.youku.mac/Data/Library/Caches/"*) ;;
  *) echo 'Recorded sandbox recipe requires Youku container caches.' >&2; exit 1 ;;
esac
case "$RUN_DIR" in */../*|*/..) echo 'Parent traversal is not accepted.' >&2; exit 1 ;; esac
mkdir -p "$RUN_DIR/profile"
exec "$APP_EXECUTABLE" "--remote-debugging-port=$CDP_PORT" --remote-debugging-address=127.0.0.1 \
  "--user-data-dir=$RUN_DIR/profile" --no-first-run
