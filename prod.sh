#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

pushd frontend
trunk build --release
popd

cargo run --bin backend --release -- --addr :: --port 8080 --static-dir ./dist
