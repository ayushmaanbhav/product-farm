#!/bin/bash
#
# Start all Product-FARM components: DGraph, Backend, Frontend
# Fails if any required port is already in use.
#

set -e

# ==============================================================================
# Configuration
# ==============================================================================

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$SCRIPT_DIR/infrastructure"
BACKEND_DIR="$SCRIPT_DIR/backend"
FRONTEND_DIR="$SCRIPT_DIR/frontend"
DGRAPH_DATA_DIR="$INFRA_DIR/dgraph-data"

# DGraph ports
DGRAPH_ZERO_PORT=5080          # Zero server management
DGRAPH_ALPHA_INTERNAL=7080     # Alpha internal communication
DGRAPH_ALPHA_HTTP=8080         # Alpha HTTP API
DGRAPH_ALPHA_GRPC=9080         # Alpha gRPC (used by backend)

# Backend port
BACKEND_HTTP_PORT=8081         # Backend REST API (8081 to avoid conflict with DGraph)

# Frontend port
FRONTEND_PORT=5173             # Vite dev server

# PIDs for cleanup
DGRAPH_ZERO_PID=""
DGRAPH_ALPHA_PID=""
BACKEND_PID=""
FRONTEND_PID=""

# Log files
LOG_DIR="$SCRIPT_DIR/logs"
mkdir -p "$LOG_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ==============================================================================
# Helper Functions
# ==============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if a port is in use
check_port() {
    local port=$1
    local service=$2

    if ss -tuln 2>/dev/null | grep -q ":${port} " || \
       netstat -tuln 2>/dev/null | grep -q ":${port} "; then
        log_error "Port $port ($service) is already in use!"
        return 1
    fi
    return 0
}

# Check all required ports
check_all_ports() {
    log_info "Checking if required ports are available..."

    local failed=0

    check_port $DGRAPH_ZERO_PORT "DGraph Zero" || failed=1
    check_port $DGRAPH_ALPHA_INTERNAL "DGraph Alpha Internal" || failed=1
    check_port $DGRAPH_ALPHA_HTTP "DGraph Alpha HTTP" || failed=1
    check_port $DGRAPH_ALPHA_GRPC "DGraph Alpha gRPC" || failed=1
    check_port $BACKEND_HTTP_PORT "Backend HTTP" || failed=1
    check_port $FRONTEND_PORT "Frontend" || failed=1

    if [ $failed -eq 1 ]; then
        log_error "One or more ports are in use. Please free them and try again."
        echo ""
        echo "You can check what's using a port with:"
        echo "  lsof -i :<port>"
        echo "  ss -tlnp | grep <port>"
        exit 1
    fi

    log_success "All ports are available"
}

# Wait for a service to be ready
wait_for_service() {
    local url=$1
    local service=$2
    local max_attempts=${3:-30}
    local attempt=1

    log_info "Waiting for $service to be ready..."

    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            log_success "$service is ready"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done

    log_error "$service failed to start within $max_attempts seconds"
    return 1
}

# Wait for a port to be listening
wait_for_port() {
    local port=$1
    local service=$2
    local max_attempts=${3:-30}
    local attempt=1

    log_info "Waiting for $service (port $port) to be ready..."

    while [ $attempt -le $max_attempts ]; do
        if ss -tuln 2>/dev/null | grep -q ":${port} " || \
           netstat -tuln 2>/dev/null | grep -q ":${port} "; then
            log_success "$service is listening on port $port"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done

    log_error "$service failed to start on port $port within $max_attempts seconds"
    return 1
}

# Cleanup function
cleanup() {
    echo ""
    log_info "Shutting down all services..."

    # Kill frontend
    if [ -n "$FRONTEND_PID" ] && kill -0 "$FRONTEND_PID" 2>/dev/null; then
        log_info "Stopping Frontend (PID: $FRONTEND_PID)..."
        kill "$FRONTEND_PID" 2>/dev/null || true
    fi

    # Kill backend
    if [ -n "$BACKEND_PID" ] && kill -0 "$BACKEND_PID" 2>/dev/null; then
        log_info "Stopping Backend (PID: $BACKEND_PID)..."
        kill "$BACKEND_PID" 2>/dev/null || true
    fi

    # Kill DGraph Alpha
    if [ -n "$DGRAPH_ALPHA_PID" ] && kill -0 "$DGRAPH_ALPHA_PID" 2>/dev/null; then
        log_info "Stopping DGraph Alpha (PID: $DGRAPH_ALPHA_PID)..."
        kill "$DGRAPH_ALPHA_PID" 2>/dev/null || true
    fi

    # Kill DGraph Zero
    if [ -n "$DGRAPH_ZERO_PID" ] && kill -0 "$DGRAPH_ZERO_PID" 2>/dev/null; then
        log_info "Stopping DGraph Zero (PID: $DGRAPH_ZERO_PID)..."
        kill "$DGRAPH_ZERO_PID" 2>/dev/null || true
    fi

    # Also kill by process name as backup
    pkill -f "dgraph zero" 2>/dev/null || true
    pkill -f "dgraph alpha" 2>/dev/null || true

    log_success "All services stopped"
    exit 0
}

# ==============================================================================
# Main Script
# ==============================================================================

echo "=============================================="
echo "  Product-FARM - Start All Components"
echo "=============================================="
echo ""
echo "Ports:"
echo "  DGraph Zero:        $DGRAPH_ZERO_PORT"
echo "  DGraph Alpha HTTP:  $DGRAPH_ALPHA_HTTP"
echo "  DGraph Alpha gRPC:  $DGRAPH_ALPHA_GRPC"
echo "  Backend REST API:   $BACKEND_HTTP_PORT"
echo "  Frontend:           $FRONTEND_PORT"
echo ""

# Set up cleanup trap
trap cleanup SIGINT SIGTERM EXIT

# Check all ports first
check_all_ports

# ==============================================================================
# Start DGraph Zero
# ==============================================================================
log_info "Starting DGraph Zero..."

cd "$INFRA_DIR"
"$INFRA_DIR/dgraph" zero \
    --my="localhost:$DGRAPH_ZERO_PORT" \
    --replicas=1 \
    --raft="idx=1" \
    > "$LOG_DIR/dgraph-zero.log" 2>&1 &
DGRAPH_ZERO_PID=$!

wait_for_port $DGRAPH_ZERO_PORT "DGraph Zero" 30 || exit 1
sleep 2  # Give Zero a moment to fully initialize

# ==============================================================================
# Start DGraph Alpha
# ==============================================================================
log_info "Starting DGraph Alpha..."

"$INFRA_DIR/dgraph" alpha \
    --my="localhost:$DGRAPH_ALPHA_INTERNAL" \
    --zero="localhost:$DGRAPH_ZERO_PORT" \
    > "$LOG_DIR/dgraph-alpha.log" 2>&1 &
DGRAPH_ALPHA_PID=$!

wait_for_service "http://localhost:$DGRAPH_ALPHA_HTTP/health" "DGraph Alpha" 30 || exit 1

# ==============================================================================
# Start Backend
# ==============================================================================
log_info "Starting Backend (Rust)... (building in debug mode)"

cd "$BACKEND_DIR"

RUST_LOG=product_farm_api=info cargo run -p product-farm-api -- 0 $BACKEND_HTTP_PORT \
    > "$LOG_DIR/backend.log" 2>&1 &
BACKEND_PID=$!

wait_for_port $BACKEND_HTTP_PORT "Backend HTTP" 120 || exit 1

# ==============================================================================
# Start Frontend
# ==============================================================================
log_info "Starting Frontend (Vite)..."

cd "$FRONTEND_DIR"
VITE_API_URL="http://localhost:$BACKEND_HTTP_PORT" npm run dev > "$LOG_DIR/frontend.log" 2>&1 &
FRONTEND_PID=$!

wait_for_port $FRONTEND_PORT "Frontend" 30 || exit 1

# ==============================================================================
# Summary
# ==============================================================================
echo ""
echo "=============================================="
log_success "All services started successfully!"
echo "=============================================="
echo ""
echo "Services:"
echo "  DGraph Zero:     http://localhost:$DGRAPH_ZERO_PORT (PID: $DGRAPH_ZERO_PID)"
echo "  DGraph Alpha:    http://localhost:$DGRAPH_ALPHA_HTTP (PID: $DGRAPH_ALPHA_PID)"
echo "  Backend REST:    http://localhost:$BACKEND_HTTP_PORT (PID: $BACKEND_PID)"
echo "  Frontend:        http://localhost:$FRONTEND_PORT (PID: $FRONTEND_PID)"
echo ""
echo "Logs:"
echo "  DGraph Zero:  $LOG_DIR/dgraph-zero.log"
echo "  DGraph Alpha: $LOG_DIR/dgraph-alpha.log"
echo "  Backend:      $LOG_DIR/backend.log"
echo "  Frontend:     $LOG_DIR/frontend.log"
echo ""
echo "Press Ctrl+C to stop all services"
echo ""

# Keep script running and forward to services
wait
