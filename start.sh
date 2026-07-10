#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

cleanup() {
  echo ""
  echo "Shutting down..."
  if [ -n "${FRONTEND_PID:-}" ]; then kill "$FRONTEND_PID" 2>/dev/null || true; fi
  if [ -n "${BACKEND_PID:-}" ]; then kill "$BACKEND_PID" 2>/dev/null || true; fi
  rm -f /tmp/aisa-frontend.pid /tmp/aisa-backend.pid
  exit 0
}
trap cleanup SIGINT SIGTERM

echo "Starting AI-SA platform..."

# Source .env
export $(grep -v '^#' .env | xargs) 2>/dev/null || true

# Generate seed data if missing
if [ ! -f data/seed/world_ports.parquet ]; then
  echo "Seeding port data..."
  (cd backend && cargo run --bin seed-ports --quiet 2>/dev/null)
fi

# Start backend
echo "Starting backend (port 3001)..."
(cd backend && cargo run --bin tommy3-backend 2>&1 | grep -v 'cargo:') &
BACKEND_PID=$!
echo "$BACKEND_PID" > /tmp/aisa-backend.pid

for i in $(seq 1 30); do
  if curl -s http://localhost:3001/api/health >/dev/null 2>&1; then
    echo "Backend ready."
    break
  fi
  if [ "$i" -eq 30 ]; then
    echo "ERROR: Backend failed to start" >&2
    cleanup
  fi
  sleep 1
done

# Start frontend
echo "Starting frontend (port 5174)..."
(cd frontend && pnpm dev) &
FRONTEND_PID=$!
echo "$FRONTEND_PID" > /tmp/aisa-frontend.pid

echo ""
echo "  Frontend: http://localhost:5174"
echo "  Backend:  http://localhost:3001"
echo ""
echo "Press Ctrl+C or run ./end.sh to stop."
wait
