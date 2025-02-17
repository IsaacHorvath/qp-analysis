#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

(trap 'kill 0' SIGINT; \
 bash -c 'cd frontend; trunk serve --proxy-backend=http://[::1]:8081/api/' & \
 bash -c 'cargo run --bin backend -- --port 8081')
