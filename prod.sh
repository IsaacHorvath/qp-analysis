#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

pushd frontend
trunk build --release
popd

DATA_SOURCE=federal_house cargo run --bin backend --release -- --addr :: --port 8080 --static-dir ./dist
