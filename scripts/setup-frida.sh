#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cache=${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}
mkdir -p "$cache/frida"
UV_PROJECT_ENVIRONMENT="$cache/frida/venv" uv sync --project "$root/crates/frida/helper" --python 3.13 --frozen --default-index https://pypi.org/simple
printf '%s\n' 'Optional Frida runtime ready; no target application was started.'

