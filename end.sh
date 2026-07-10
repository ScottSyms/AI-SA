#!/usr/bin/env bash
set -euo pipefail
echo "Stopping AI-SA platform..."

kill_port() {
  local port=$1
  local pids
  pids=$(lsof -ti :"$port" 2>/dev/null || true)
  if [ -n "$pids" ]; then
    echo "Killing process(es) on port $port: $pids"
    kill $pids 2>/dev/null && echo "Port $port freed." || true
  else
    echo "Nothing listening on port $port."
  fi
}

kill_port 3001
kill_port 5174

rm -f /tmp/aisa-frontend.pid /tmp/aisa-backend.pid
echo "Done."
