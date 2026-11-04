#!/bin/sh
set -eu
reference_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
reference_cache=${INJECTRA_REFERENCE_CACHE:-${INJECTRA_CACHE_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/injectra}/references}
mkdir -p "$reference_cache/npm"
UV_PROJECT_ENVIRONMENT="$reference_cache/venv" uv sync --project "$reference_root" --python 3.13 --frozen --default-index https://pypi.org/simple >"$reference_cache/python-setup.log" 2>&1
cp "$reference_root/package.json" "$reference_root/package-lock.json" "$reference_cache/npm/"
npm ci --prefix "$reference_cache/npm" --ignore-scripts --no-audit --no-fund --registry=https://registry.npmjs.org
if [ -L "$reference_root/node_modules" ]; then
    [ "$(readlink "$reference_root/node_modules")" = "$reference_cache/npm/node_modules" ] || {
        echo 'node_modules points to a different reference cache; reconcile explicitly.' >&2
        exit 1
    }
elif [ -e "$reference_root/node_modules" ]; then
    echo 'Existing node_modules will not be replaced.' >&2
    exit 1
else
    ln -s "$reference_cache/npm/node_modules" "$reference_root/node_modules"
fi
