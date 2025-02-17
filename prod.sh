#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

pushd frontend
trunk build
popd

cargo run --bin backend --release -- --port 8080 --static-dir ./dist
