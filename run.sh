#!/usr/bin/env bash
# Runs eh3d-tools with its sample. Extra arguments replace the sample: ./run.sh [ARGUMENTS]
set -u
cd "$(dirname "$0")"
pause() { [ -n "${NOPAUSE:-}" ] || { read -r -p "Press Enter to close..." _ || true; }; }
trap pause EXIT
if [ "$#" -eq 0 ]; then set -- calc drill 0.8 fr4 24000; fi
cargo run --release --quiet -- "$@"
