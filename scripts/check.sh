#!/usr/bin/env bash
# Everything that should pass before you tag a release.
set -euo pipefail
cd "$(dirname "$0")/.."
npm install
npm run typecheck
cargo test -p blockmc-core
npm run build
cargo check -p blockmc
echo "ALL CHECKS PASSED"
