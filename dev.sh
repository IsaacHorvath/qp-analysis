#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

(trap 'kill 0' SIGINT; \
 bash -c 'cd frontend; trunk serve --address 0.0.0.0 --proxy-backend=http://[::1]:8081/api/' & \
 bash -c 'RUST_LOG=trace DATA_SOURCE=federal_house cargo run --bin backend -- --port 8081 --dummy')
