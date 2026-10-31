#!/bin/sh
set -eu
# Keep high-churn build products outside synchronized source trees.
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cache_root=${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$cache_root/cargo-target}
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
cd "$root"
exec cargo "$@"
