#!/bin/sh
set -eu
: "${APP_EXECUTABLE:?Figma executable path}"
: "${TEST_HOME:?Independent external test home}"
: "${CDP_PORT:?Unused loopback debugging port}"
[ "$TEST_HOME" != "$HOME" ] || { echo 'Use an independent test home.' >&2; exit 1; }
mkdir -p "$TEST_HOME/Library/Application Support"
set -- "$APP_EXECUTABLE" "--remote-debugging-port=$CDP_PORT" --remote-debugging-address=127.0.0.1 --no-first-run
if [ -n "${PROXY_SERVER:-}" ]; then set -- "$@" "--proxy-server=$PROXY_SERVER"; fi
exec env HOME="$TEST_HOME" CFFIXED_USER_HOME="$TEST_HOME" FIGMA_TEST=1 "$@"
