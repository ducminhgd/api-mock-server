#!/bin/bash
set -euo pipefail

# Nginx writes temp files at runtime; /tmp is writable by any user.
mkdir -p /tmp/nginx_client_body /tmp/nginx_proxy /tmp/nginx_fastcgi \
         /tmp/nginx_uwsgi /tmp/nginx_scgi

/app/server &
BACKEND_PID=$!

nginx -g "daemon off;" &
NGINX_PID=$!

cleanup() {
    kill "$BACKEND_PID" "$NGINX_PID" 2>/dev/null || true
    wait "$BACKEND_PID" "$NGINX_PID" 2>/dev/null || true
}
trap cleanup TERM INT

# If either child exits the container should restart — signal the orchestrator
# by exiting with a non-zero code.
wait -n "$BACKEND_PID" "$NGINX_PID"
echo "child process exited unexpectedly" >&2
cleanup
exit 1
