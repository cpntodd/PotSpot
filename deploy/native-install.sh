#!/bin/bash
# PotSpot -- Native Debian 12 Install Script
# Installs PotSpot directly on the OS (no Docker required).
#
# What this installs:
#   - Rust (via rustup)        -- to compile the API
#   - Node.js 20.x             -- to build the SvelteKit frontend
#   - PostgreSQL 16            -- database
#   - Caddy                    -- reverse proxy + auto-TLS
#   - potspot-api systemd unit -- runs the API as a service
#
# What it does NOT install (you must provide):
#   - MinIO (photo storage)    -- the API falls back to local disk if unavailable
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/cpntodd/PotSpot/main/deploy/native-install.sh | sudo bash
#
# Or with a pre-set domain:
#   sudo DOMAIN=potspot.example.com bash deploy/native-install.sh

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================
APP="PotSpot"
INSTALL_PATH="/opt/potspot"
REPO_URL="https://github.com/cpntodd/PotSpot.git"
CADDY_HTTP_PORT="${CADDY_HTTP_PORT:-8080}"
CADDY_HTTPS_PORT="${CADDY_HTTPS_PORT:-8443}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot -- Native Debian Installer${NC}"
echo -e "${GREEN}============================================${NC}"

# ============================================================================
# Step 0: Pre-flight checks
# ============================================================================
echo -e "${YELLOW}[0/8] Pre-flight checks...${NC}"

if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}This script must be run as root. Use: sudo bash native-install.sh${NC}"
    exit 1
fi

if ! grep -q "12" /etc/debian_version 2>/dev/null; then
    echo -e "${YELLOW}Warning: This script is designed for Debian 12 (Bookworm).${NC}"
    echo -e "${YELLOW}Detected: $(cat /etc/debian_version 2>/dev/null || echo 'unknown')${NC}"
    echo -n "Continue anyway? (y/N): "
    read -r cont < /dev/tty
    if [[ ! "${cont,,}" =~ ^(y|yes)$ ]]; then
        exit 0
    fi
fi

# Resolve domain
if [[ -z "${DOMAIN:-}" ]]; then
    echo ""
    echo -e "${CYAN}Enter your domain name (e.g. potspot.example.com):${NC}"
    read -r DOMAIN < /dev/tty
    if [[ -z "$DOMAIN" ]]; then
        echo -e "${RED}Domain is required. Exiting.${NC}"
        exit 1
    fi
fi

# Caddy ports (optional override)
echo -e "${CYAN}HTTP port [${CADDY_HTTP_PORT}]:${NC}"
read -r custom_http < /dev/tty
CADDY_HTTP_PORT="${custom_http:-$CADDY_HTTP_PORT}"

echo -e "${CYAN}HTTPS port [${CADDY_HTTPS_PORT}]:${NC}"
read -r custom_https < /dev/tty
CADDY_HTTPS_PORT="${custom_https:-$CADDY_HTTPS_PORT}"

# Google OAuth (optional)
echo -e "${CYAN}Google OAuth Client ID (leave blank to skip):${NC}"
read -r GOOGLE_CLIENT_ID < /dev/tty
echo -e "${CYAN}Google OAuth Client Secret (leave blank to skip):${NC}"
read -r GOOGLE_CLIENT_SECRET < /dev/tty

echo -e "${GREEN}  Domain: ${DOMAIN}${NC}"
echo ""

# ============================================================================
# Step 1: Install system packages
# ============================================================================
echo -e "${YELLOW}[1/8] Installing system dependencies...${NC}"

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq

apt-get install -y -qq \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    openssl \
    ca-certificates \
    gnupg \
    lsb-release \
    unzip

echo -e "${GREEN}  System packages installed.${NC}"

# ============================================================================
# Step 2: Install Rust (via rustup)
# ============================================================================
echo -e "${YELLOW}[2/8] Installing Rust...${NC}"

if command -v rustc &>/dev/null; then
    echo -e "${GREEN}  Rust $(rustc --version) already installed.${NC}"
else
    echo "  Downloading rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable > /dev/null 2>&1
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
    echo -e "${GREEN}  Rust $(rustc --version) installed.${NC}"
fi

# Ensure cargo is in PATH for this script
export PATH="$HOME/.cargo/bin:$PATH"

# ============================================================================
# Step 3: Install Node.js 20.x
# ============================================================================
echo -e "${YELLOW}[3/8] Installing Node.js 20.x...${NC}"

if command -v node &>/dev/null && node --version | grep -q "v20"; then
    echo -e "${GREEN}  Node.js $(node --version) already installed.${NC}"
else
    echo "  Adding NodeSource repository..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash - > /dev/null 2>&1
    apt-get install -y -qq nodejs > /dev/null 2>&1
    echo -e "${GREEN}  Node.js $(node --version) installed.${NC}"
fi

# ============================================================================
# Step 4: Install PostgreSQL 16
# ============================================================================
echo -e "${YELLOW}[4/8] Installing PostgreSQL 16...${NC}"

if command -v psql &>/dev/null && psql --version | grep -q "16"; then
    echo -e "${GREEN}  PostgreSQL 16 already installed.${NC}"
else
    echo "  Adding PostgreSQL repository..."
    curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /usr/share/keyrings/postgresql-archive-keyring.gpg 2>/dev/null
    echo "deb [signed-by=/usr/share/keyrings/postgresql-archive-keyring.gpg] http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list
    apt-get update -qq > /dev/null 2>&1
    apt-get install -y -qq postgresql-16 > /dev/null 2>&1
    echo -e "${GREEN}  PostgreSQL 16 installed.${NC}"
fi

# Create potspot database and user
echo "  Configuring database..."
DB_PASSWORD=$(openssl rand -hex 16)

su - postgres -c "psql -tc \"SELECT 1 FROM pg_roles WHERE rolname='potspot'\"" 2>/dev/null | grep -q 1 || \
    su - postgres -c "psql -c \"CREATE USER potspot WITH PASSWORD '${DB_PASSWORD}';\"" > /dev/null 2>&1

su - postgres -c "psql -tc \"SELECT 1 FROM pg_database WHERE datname='potspot'\"" 2>/dev/null | grep -q 1 || \
    su - postgres -c "psql -c \"CREATE DATABASE potspot OWNER potspot;\"" > /dev/null 2>&1

echo -e "${GREEN}  Database 'potspot' ready.${NC}"

# ============================================================================
# Step 5: Install Caddy
# ============================================================================
echo -e "${YELLOW}[5/8] Installing Caddy...${NC}"

if command -v caddy &>/dev/null; then
    echo -e "${GREEN}  Caddy $(caddy version | head -1) already installed.${NC}"
else
    echo "  Adding Caddy repository..."
    curl -fsSL https://dl.cloudsmith.io/public/caddy/stable/gpg.key | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg 2>/dev/null
    echo "deb [signed-by=/usr/share/keyrings/caddy-stable-archive-keyring.gpg] https://dl.cloudsmith.io/public/caddy/stable/deb/debian any-version main" > /etc/apt/sources.list.d/caddy-stable.list
    apt-get update -qq > /dev/null 2>&1
    apt-get install -y -qq caddy > /dev/null 2>&1
    echo -e "${GREEN}  Caddy installed.${NC}"
echo -e "${GREEN}  Caddy installed.${NC}"
fi

# ============================================================================
# Step 6: Install MinIO (Object Storage)
# ============================================================================
echo -e "${YELLOW}[6/9] Installing MinIO...${NC}"

MINIO_BINARY="/usr/local/bin/minio"
MC_BINARY="/usr/local/bin/mc"
MINIO_DATA="/var/lib/minio"

if [[ -x "$MINIO_BINARY" ]] && [[ -x "$MC_BINARY" ]]; then
    echo -e "${GREEN}  MinIO already installed.${NC}"
else
    echo "  Downloading MinIO server..."
    curl -fsSL -o "$MINIO_BINARY" https://dl.min.io/server/minio/release/linux-amd64/minio
    chmod +x "$MINIO_BINARY"

    echo "  Downloading MinIO client (mc)..."
    curl -fsSL -o "$MC_BINARY" https://dl.min.io/client/mc/release/linux-amd64/mc
    chmod +x "$MC_BINARY"

    echo -e "${GREEN}  MinIO binaries installed.${NC}"
fi

# Create minio system user
if ! id -u minio &>/dev/null; then
    useradd --system --home-dir "$MINIO_DATA" --shell /usr/sbin/nologin minio
fi

# Create data directory
mkdir -p "$MINIO_DATA/data"
chown -R minio:minio "$MINIO_DATA"

echo -e "${GREEN}  MinIO ready.${NC}"

# ============================================================================
# Step 7: Clone and build
# ============================================================================
echo -e "${YELLOW}[7/9] Cloning and building PotSpot...${NC}"

# Create potspot system user
if ! id -u potspot &>/dev/null; then
    useradd --system --home-dir "$INSTALL_PATH" --shell /usr/sbin/nologin potspot
fi

# Clone repo
if [ -d "$INSTALL_PATH" ]; then
    echo "  Directory exists, pulling latest..."
    cd "$INSTALL_PATH"
    git pull origin main
else
    git clone "$REPO_URL" "$INSTALL_PATH"
fi

cd "$INSTALL_PATH"

# Build backend (as root -- potspot user only needed at runtime)
echo "  Building Rust API (this takes several minutes)..."
cd "$INSTALL_PATH/backend"
cargo build --release 2>&1 | tail -5
echo -e "${GREEN}  Backend built.${NC}"

# Build frontend
echo "  Building SvelteKit frontend..."
cd "$INSTALL_PATH/web"
npm ci --silent 2>&1 | tail -3
npm run build 2>&1 | tail -3
echo -e "${GREEN}  Frontend built.${NC}"

# Make the API binary accessible to the potspot user
chown -R potspot:potspot "$INSTALL_PATH"

# ============================================================================
# Step 7: Configure
# ============================================================================
echo -e "${YELLOW}[8/9] Configuring PotSpot...${NC}"

cd "$INSTALL_PATH"

# Generate secrets
JWT_SECRET=$(openssl rand -hex 32)
JWT_REFRESH=$(openssl rand -hex 32)
MINIO_KEY=$(openssl rand -hex 16)
MINIO_SECRET=$(openssl rand -hex 16)

cat > "$INSTALL_PATH/.env" << ENVEOF
# PotSpot Environment
DATABASE_URL=postgres://potspot:${DB_PASSWORD}@localhost:5432/potspot
JWT_SECRET=${JWT_SECRET}
JWT_REFRESH_SECRET=${JWT_REFRESH}
PUBLIC_URL=https://${DOMAIN}
CORS_ORIGIN=https://${DOMAIN}
MINIO_ENDPOINT=http://localhost:9000
MINIO_ACCESS_KEY=${MINIO_KEY}
MINIO_SECRET_KEY=${MINIO_SECRET}
MINIO_BUCKET=potspot-photos
GOOGLE_CLIENT_ID=${GOOGLE_CLIENT_ID:-}
GOOGLE_CLIENT_SECRET=${GOOGLE_CLIENT_SECRET:-}
RUST_LOG=info
ENVEOF
chmod 600 "$INSTALL_PATH/.env"
chown potspot:potspot "$INSTALL_PATH/.env"

# Configure Caddy
cat > /etc/caddy/Caddyfile << CADDYEOF
# PotSpot -- auto-generated by native-install.sh
${DOMAIN} {
    root * /opt/potspot/web/build
    file_server
    try_files {path} /index.html

    handle /api/* {
        reverse_proxy localhost:3000
    }

    header {
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        Referrer-Policy "strict-origin-when-cross-origin"
    }
}
CADDYEOF

# Caddy listens on configured ports
if [[ "$CADDY_HTTP_PORT" != "80" ]] || [[ "$CADDY_HTTPS_PORT" != "443" ]]; then
    # Add global options for custom ports
    sed -i "1s/^/{ http_port ${CADDY_HTTP_PORT} https_port ${CADDY_HTTPS_PORT} }\n/" /etc/caddy/Caddyfile
fi

echo -e "${GREEN}  Configuration written.${NC}"

# ============================================================================
# Step 8: Install systemd service and start
# ============================================================================
echo -e "${YELLOW}[9/9] Starting services...${NC}"

# Install systemd units
cp "$INSTALL_PATH/deploy/potspot-api.service" /etc/systemd/system/potspot-api.service
cp "$INSTALL_PATH/deploy/potspot-minio.service" /etc/systemd/system/potspot-minio.service
systemctl daemon-reload

# Start and enable services (order matters: minio before api)
systemctl enable --now postgresql
systemctl enable --now potspot-minio
sleep 3  # Let MinIO start and create the bucket
systemctl enable --now potspot-api
systemctl enable --now caddy

# Reload Caddy to pick up new config
systemctl reload caddy 2>/dev/null || systemctl restart caddy

# Wait for API
echo "  Waiting for API..."
sleep 5
for i in {1..12}; do
    if curl -sf http://localhost:3000/api/v1/strains?per_page=1 > /dev/null 2>&1; then
        echo -e "${GREEN}  API is responding.${NC}"
        break
    fi
    sleep 5
done

# ============================================================================
# Summary
# ============================================================================
echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot is running!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "  Domain:     ${YELLOW}https://${DOMAIN}${NC}"
if [[ "$CADDY_HTTP_PORT" != "80" ]]; then
    echo -e "  HTTP port:  ${YELLOW}${CADDY_HTTP_PORT}${NC}"
    echo -e "  HTTPS port: ${YELLOW}${CADDY_HTTPS_PORT}${NC}"
    echo -e "  ${CYAN}Point your reverse proxy at localhost:${CADDY_HTTP_PORT}${NC}"
fi
echo ""
echo -e "  ${GREEN}Services:${NC}"
echo -e "    systemctl status potspot-api"
echo -e "    systemctl status potspot-minio"
echo -e "    systemctl status caddy"
echo -e "    systemctl status postgresql"
echo ""
echo -e "  ${GREEN}Logs:${NC}"
echo -e "    journalctl -u potspot-api -f"
echo -e "    journalctl -u potspot-minio -f"
echo -e "    journalctl -u caddy -f"
echo ""
echo -e "  ${GREEN}Config:${NC}"
echo -e "    ${INSTALL_PATH}/.env"
echo -e "    /etc/caddy/Caddyfile"
echo ""
echo -e "  ${GREEN}MinIO Console:${NC}  http://localhost:9001"
echo -e "    Login: MINIO_ACCESS_KEY / MINIO_SECRET_KEY from ${INSTALL_PATH}/.env"
echo ""
echo -e "  ${YELLOW}Point DNS for ${DOMAIN} to this server's IP.${NC}"
