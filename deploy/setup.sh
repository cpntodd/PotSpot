#!/bin/bash
# PotSpot -- LXC Container Setup Script
# Run this INSIDE an existing Debian LXC container or VPS.
#
# Usage (inside the container):
#   curl -sSL https://raw.githubusercontent.com/cpntodd/PotSpot/main/deploy/setup.sh | sudo bash

set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'

# Configuration -- provide via environment or edit below
DOMAIN="${DOMAIN:-}"
POTSPOT_DIR="${POTSPOT_DIR:-/opt/potspot}"
CADDY_HTTP_PORT="${CADDY_HTTP_PORT:-8080}"
CADDY_HTTPS_PORT="${CADDY_HTTPS_PORT:-8443}"

# Detect LAN IP for fallback when no domain
LAN_IP=$(ip -4 addr show eth0 2>/dev/null | awk '/inet / {print $2}' | cut -d/ -f1 | head -n1)
LAN_IP="${LAN_IP:-$(ip -4 addr show | awk '/inet / && !/127\.0\.0\.1/ {print $2}' | cut -d/ -f1 | head -n1)}"
LAN_IP="${LAN_IP:-127.0.0.1}"

if [[ -z "$DOMAIN" ]]; then
    USING_DOMAIN=false
    PUBLIC_URL="http://${LAN_IP}:${CADDY_HTTP_PORT}"
else
    USING_DOMAIN=true
    PUBLIC_URL="https://${DOMAIN}"
fi

echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot -- Container Setup${NC}"
echo -e "${GREEN}============================================${NC}"

# Check root
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}This script must be run as root. Use: sudo bash setup.sh${NC}"
    exit 1
fi

# ============================================================================
# 1. Install dependencies
# ============================================================================
echo -e "${YELLOW}[1/4] Installing dependencies...${NC}"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq

# Install missing core tools
missing_tools=()
for tool in curl git openssl; do
  if ! command -v "$tool" &>/dev/null; then
    missing_tools+=("$tool")
  fi
done
if [ ${#missing_tools[@]} -gt 0 ]; then
  apt-get install -y -qq "${missing_tools[@]}" > /dev/null 2>&1
  echo "  Installed: ${missing_tools[*]}"
fi

if ! command -v docker &>/dev/null; then
    echo "  Installing Docker..."
    curl -fsSL https://get.docker.com | sh > /dev/null 2>&1
    systemctl enable --now docker
fi

echo -e "${GREEN}  Docker $(docker --version | cut -d' ' -f3 | cut -d',' -f1)${NC}"

# ============================================================================
# 2. Clone and configure
# ============================================================================
echo -e "${YELLOW}[2/4] Setting up PotSpot...${NC}"

if [ -d "$POTSPOT_DIR" ]; then
    echo "  Directory exists, pulling latest..."
    cd "$POTSPOT_DIR"
    git pull origin main
else
    git clone https://github.com/cpntodd/PotSpot.git "$POTSPOT_DIR"
    cd "$POTSPOT_DIR"
fi

# Generate secrets if not present
if [ ! -f docker/.env ]; then
    cat > docker/.env << ENVEOF
POSTGRES_USER=potspot
POSTGRES_PASSWORD=$(openssl rand -hex 16)
POSTGRES_DB=potspot
DATABASE_URL=postgres://potspot:\$POSTGRES_PASSWORD@db:5432/potspot
JWT_SECRET=$(openssl rand -hex 32)
JWT_REFRESH_SECRET=$(openssl rand -hex 32)
MINIO_ACCESS_KEY=$(openssl rand -hex 16)
MINIO_SECRET_KEY=$(openssl rand -hex 16)
MINIO_BUCKET=potspot-photos
PUBLIC_URL=${PUBLIC_URL}
CORS_ORIGIN=${PUBLIC_URL}
CADDY_HTTP_PORT=${CADDY_HTTP_PORT}
CADDY_HTTPS_PORT=${CADDY_HTTPS_PORT}
GOOGLE_CLIENT_ID=${GOOGLE_CLIENT_ID:-}
GOOGLE_CLIENT_SECRET=${GOOGLE_CLIENT_SECRET:-}
ENVEOF
    echo "  Generated secrets in docker/.env"
fi

# Update domain in Caddyfile
if $USING_DOMAIN; then
    sed -i "s/potspot.example.com/$DOMAIN/g" docker/Caddyfile
else
    sed -i "s/potspot.example.com/:${CADDY_HTTP_PORT}/g" docker/Caddyfile
fi

echo -e "${GREEN}  Repository ready.${NC}"

# ============================================================================
# 3. Build and start
# ============================================================================
echo -e "${YELLOW}[3/4] Building and starting containers...${NC}"
cd "$POTSPOT_DIR"
CADDY_HTTP_PORT="${CADDY_HTTP_PORT}" CADDY_HTTPS_PORT="${CADDY_HTTPS_PORT}" \
  docker compose -f docker/docker-compose.prod.yml up -d --build

echo -e "${GREEN}  Containers started.${NC}"

# ============================================================================
# 4. Verify
# ============================================================================
echo -e "${YELLOW}[4/4] Verifying...${NC}"
sleep 5

echo ""
echo "--- Container Status ---"
docker compose -f "$POTSPOT_DIR/docker/docker-compose.prod.yml" ps

echo ""
echo "--- Testing API ---"
curl -sf http://localhost:3000/api/v1/strains?per_page=1 > /dev/null 2>&1 && \
    echo -e "${GREEN}  API is responding!${NC}" || \
    echo -e "${YELLOW}  API starting up (may need 30-60s for first build)...${NC}"

# ============================================================================
# Summary
# ============================================================================
echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot is running!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
if $USING_DOMAIN; then
  echo -e "  Domain:    ${YELLOW}https://$DOMAIN${NC}"
  if [[ "${CADDY_HTTP_PORT}" != "80" ]]; then
    echo -e "  HTTP port: ${YELLOW}${CADDY_HTTP_PORT}${NC} (reverse proxy target)"
    echo -e "  HTTPS port:${YELLOW}${CADDY_HTTPS_PORT}${NC}"
  fi
else
  echo -e "  URL:       ${YELLOW}${PUBLIC_URL}${NC}"
  echo -e "  ${YELLOW}No domain -- serving via HTTP on LAN IP${NC}"
fi
echo -e "  Directory: ${YELLOW}$POTSPOT_DIR${NC}"
echo ""
echo "  Useful commands:"
echo "    cd $POTSPOT_DIR"
echo "    docker compose -f docker/docker-compose.prod.yml ps"
echo "    docker compose -f docker/docker-compose.prod.yml logs -f api"
echo "    docker compose -f docker/docker-compose.prod.yml restart"
echo ""
echo "  To add OAuth:"
echo "    Edit docker/.env and add GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET"
echo "    Then: docker compose -f docker/docker-compose.prod.yml up -d"
