#!/bin/bash
#
# Stop all Product-FARM components: DGraph, Backend, Frontend
#

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo "=============================================="
echo "  Product-FARM - Stop All Components"
echo "=============================================="
echo ""

# Stop Frontend (npm/node processes on port 5173)
log_info "Stopping Frontend..."
sudo pkill -f "vite" 2>/dev/null && log_success "Frontend stopped" || log_warn "Frontend was not running"

# Stop Backend (cargo/rust processes)
log_info "Stopping Backend..."
sudo pkill -f "product-farm-api" 2>/dev/null && log_success "Backend stopped" || log_warn "Backend was not running"

# Stop DGraph Alpha
log_info "Stopping DGraph Alpha..."
sudo pkill -f "dgraph alpha" 2>/dev/null && log_success "DGraph Alpha stopped" || log_warn "DGraph Alpha was not running"

# Stop DGraph Zero
log_info "Stopping DGraph Zero..."
sudo pkill -f "dgraph zero" 2>/dev/null && log_success "DGraph Zero stopped" || log_warn "DGraph Zero was not running"

echo ""
log_success "All services stopped"
