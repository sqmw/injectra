#!/bin/sh
set -eu
: "${INJECTRA_BIN:?Path to the existing built Injectra CLI}"
: "${CDP_PORT:?Explicit loopback debugging port}"
: "${TARGET_ID:?Choose the exact page from targets output}"
: "${BUNDLE_MANIFEST:?Trusted local manifest file}"
exec "$INJECTRA_BIN" apply --port "$CDP_PORT" --target "$TARGET_ID" --bundle "$BUNDLE_MANIFEST"
