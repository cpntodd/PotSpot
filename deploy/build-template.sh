#!/bin/bash
# PotSpot -- Proxmox LXC Template Builder
# Run this ON THE PROXMOX HOST to create a pre-built PotSpot appliance template.
#
# What it does:
#   1. Creates a temporary Debian LXC (13 Trixie by default, falls back to 12)
#   2. Installs Docker + Docker Compose
#   3. Clones PotSpot and pre-builds all Docker images
#   4. Places a first-boot init script (no secrets pre-baked)
#   5. Stops the CT and converts it to a Proxmox template
#
# Result: Clone the template in seconds -- no build wait.
#
# Usage:
#   sudo bash deploy/build-template.sh
#
# After building, deploy a clone:
#   sudo pct clone <TEMPLATE_ID> <NEW_ID> --hostname potspot
#   sudo pct start <NEW_ID>
#   sudo pct enter <NEW_ID>
#   cd /opt/potspot && sudo ./init.sh

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================
TEMPLATE_CT_ID="${TEMPLATE_CT_ID:-900}"       # Temporary CT used to build the template
TEMPLATE_NAME="${TEMPLATE_NAME:-potspot-template}"
CT_MEMORY="${CT_MEMORY:-2048}"                # MB (only needed during build)
CT_SWAP="${CT_SWAP:-512}"                     # MB
CT_CORES="${CT_CORES:-2}"
CT_DISK="${CT_DISK:-20}"                      # GB
CT_ROOT_PASSWORD="${CT_ROOT_PASSWORD:-templatebuild}"
STORAGE="${STORAGE:-local-lvm}"
DEBIAN_TEMPLATE="${DEBIAN_TEMPLATE:-local:vztmpl/debian-13-standard_13.0-1_amd64.tar.zst}"
DEBIAN_TEMPLATE_FALLBACK="local:vztmpl/debian-12-standard_12.7-1_amd64.tar.zst"

# ============================================================================
# Colors
# ============================================================================
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'

echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot -- LXC Template Builder${NC}"
echo -e "${GREEN}============================================${NC}"

# ============================================================================
# Step 0: Verify Proxmox host
# ============================================================================
echo -e "${YELLOW}[1/7] Verifying Proxmox host...${NC}"

if ! command -v pveam &>/dev/null || ! command -v pct &>/dev/null; then
    echo -e "${RED}ERROR: This script must run on a Proxmox VE host.${NC}"
    echo "  pveam and pct are required but not found."
    exit 1
fi

for tool in openssl wget curl; do
    if ! command -v "$tool" &>/dev/null; then
        apt-get install -y -qq "$tool" > /dev/null 2>&1 || {
            echo -e "${RED}ERROR: Failed to install ${tool}.${NC}"
            exit 1
        }
    fi
done

echo -e "${GREEN}  Proxmox host verified.${NC}"

# ============================================================================
# Step 1: Download Debian template if needed
# ============================================================================
echo -e "${YELLOW}[2/7] Checking Debian template...${NC}"
# Prefer Debian 13 (Trixie), fall back to Debian 12 (Bookworm)
TEMPLATE_NAME_SHORT=$(echo "$DEBIAN_TEMPLATE" | grep -oP 'debian-\d+' || echo "debian-13")
if ! pveam list "$STORAGE" 2>/dev/null | grep -q "$TEMPLATE_NAME_SHORT"; then
    echo "  ${TEMPLATE_NAME_SHORT} not found, trying fallback..."
    if ! pveam list "$STORAGE" 2>/dev/null | grep -q "debian-12"; then
        echo "  Downloading Debian template..."
        pveam update
        pveam download local debian-13-standard_13.0-1_amd64.tar.zst 2>/dev/null || \
        pveam download local debian-12-standard_12.7-1_amd64.tar.zst
    fi
fi
echo -e "${GREEN}  Template ready.${NC}"

# ============================================================================
# Step 2: Create temporary build CT
# ============================================================================
echo -e "${YELLOW}[3/7] Creating build container (ID: $TEMPLATE_CT_ID)...${NC}"

# Destroy existing CT with same ID if it exists
if pct status "$TEMPLATE_CT_ID" &>/dev/null; then
    echo "  Container $TEMPLATE_CT_ID already exists. Destroying..."
    pct stop "$TEMPLATE_CT_ID" --skiplock 2>/dev/null || true
    sleep 3
    pct destroy "$TEMPLATE_CT_ID" --purge 2>/dev/null || true
fi

pct create "$TEMPLATE_CT_ID" "$DEBIAN_TEMPLATE" \
    --hostname "$TEMPLATE_NAME" \
    --memory "$CT_MEMORY" \
    --swap "$CT_SWAP" \
    --cores "$CT_CORES" \
    --rootfs "$STORAGE:$CT_DISK" \
    --net0 "name=eth0,bridge=vmbr0,ip=dhcp" \
    --password "$CT_ROOT_PASSWORD" \
    --features "nesting=1" \
    --unprivileged 1 \
    --onboot 0

echo -e "${GREEN}  Build container created.${NC}"

# ============================================================================
# Step 3: Start CT and install dependencies
# ============================================================================
echo -e "${YELLOW}[4/7] Installing Docker + dependencies...${NC}"
pct start "$TEMPLATE_CT_ID"

echo "  Waiting for network..."
sleep 15

pct exec "$TEMPLATE_CT_ID" -- bash -c '
    set -e
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq curl git ca-certificates openssl > /dev/null 2>&1

    # Install Docker
    curl -fsSL https://get.docker.com | sh > /dev/null 2>&1
    systemctl enable docker

    # Disable Docker auto-start in template (clones will start it)
    # We leave docker enabled but containers stopped -- clones just run docker compose up

    echo "Docker: $(docker --version)"
'

echo -e "${GREEN}  Docker installed.${NC}"

# ============================================================================
# Step 4: Clone PotSpot and pre-build images
# ============================================================================
echo -e "${YELLOW}[5/7] Cloning PotSpot and pre-building images...${NC}"
echo "  This will take several minutes (downloading + compiling Rust)..."

pct exec "$TEMPLATE_CT_ID" -- bash -c '
    set -e
    cd /opt
    git clone https://github.com/cpntodd/PotSpot.git potspot
    cd potspot

    # Generate placeholder .env so docker compose build doesn'\''t fail on variable checks
    cat > docker/.env << "ENVEOF"
POSTGRES_USER=potspot
POSTGRES_PASSWORD=template_placeholder
POSTGRES_DB=potspot
DATABASE_URL=postgres://potspot:template_placeholder@db:5432/potspot
JWT_SECRET=template_placeholder
JWT_REFRESH_SECRET=template_placeholder
MINIO_ACCESS_KEY=template_placeholder
MINIO_SECRET_KEY=template_placeholder
MINIO_BUCKET=potspot-photos
PUBLIC_URL=https://potspot.example.com
CORS_ORIGIN=https://potspot.example.com
CADDY_HTTP_PORT=8080
CADDY_HTTPS_PORT=8443
ENVEOF

    # Place Caddyfile placeholder
    cat > docker/Caddyfile << "CADDYEOF"
potspot.example.com {
    root * /srv/web
    file_server
    handle /api/* {
        reverse_proxy api:3000
    }
}
CADDYEOF

    # Pre-build all images (this is the slow part -- cached in template)
    cd docker
    docker compose -f docker-compose.prod.yml build --progress=plain

    echo "Images pre-built successfully."
'

echo -e "${GREEN}  PotSpot cloned and images pre-built.${NC}"

# ============================================================================
# Step 5: Place init script and clean up
# ============================================================================
echo -e "${YELLOW}[6/7] Placing first-boot init script...${NC}"

pct exec "$TEMPLATE_CT_ID" -- bash -c '
    set -e

    # Remove placeholder secrets (clones must generate their own)
    rm -f /opt/potspot/docker/.env
    rm -f /opt/potspot/docker/Caddyfile

    # Restore Caddyfile from git
    cd /opt/potspot
    git checkout -- docker/Caddyfile

    # Create first-boot init script
    cat > /opt/potspot/init.sh << "INITEOF"
#!/bin/bash
# PotSpot -- First Boot Initialization
# Run this once after cloning from the template.
#
# Usage:
#   cd /opt/potspot && sudo ./init.sh
#
# Or non-interactively:
#   DOMAIN=potspot.example.com CADDY_HTTP_PORT=8080 CADDY_HTTPS_PORT=8443 sudo -E ./init.sh

set -euo pipefail

RED="\033[0;31m"; GREEN="\033[0;32m"; YELLOW="\033[1;33m"; CYAN="\033[0;36m"; NC="\033[0m"

echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot -- First Boot Setup${NC}"
echo -e "${GREEN}============================================${NC}"

# Check root
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}This script must be run as root. Use: sudo ./init.sh${NC}"
    exit 1
fi

cd /opt/potspot

# --- Domain ---
if [[ -z "${DOMAIN:-}" ]]; then
    echo ""
    echo -e "${CYAN}Enter your domain name (e.g. potspot.example.com):${NC}"
    read -r DOMAIN
    if [[ -z "$DOMAIN" ]]; then
        echo -e "${RED}Domain is required. Exiting.${NC}"
        exit 1
    fi
fi

# --- Ports ---
CADDY_HTTP_PORT="${CADDY_HTTP_PORT:-8080}"
CADDY_HTTPS_PORT="${CADDY_HTTPS_PORT:-8443}"

# --- OAuth (optional) ---
if [[ -z "${GOOGLE_CLIENT_ID:-}" ]]; then
    echo ""
    echo -e "${CYAN}Enter Google OAuth Client ID (leave blank to skip):${NC}"
    read -r GOOGLE_CLIENT_ID
fi
if [[ -z "${GOOGLE_CLIENT_SECRET:-}" ]]; then
    echo -e "${CYAN}Enter Google OAuth Client Secret (leave blank to skip):${NC}"
    read -r GOOGLE_CLIENT_SECRET
fi

echo ""
echo -e "${YELLOW}Generating secrets...${NC}"

# Generate fresh secrets
DB_PASSWORD=$(openssl rand -hex 16)
JWT_SECRET=$(openssl rand -hex 32)
JWT_REFRESH=$(openssl rand -hex 32)
MINIO_KEY=$(openssl rand -hex 16)
MINIO_SECRET=$(openssl rand -hex 16)

cat > docker/.env << ENVEOF
POSTGRES_USER=potspot
POSTGRES_PASSWORD=${DB_PASSWORD}
POSTGRES_DB=potspot
DATABASE_URL=postgres://potspot:${DB_PASSWORD}@db:5432/potspot
JWT_SECRET=${JWT_SECRET}
JWT_REFRESH_SECRET=${JWT_REFRESH}
MINIO_ACCESS_KEY=${MINIO_KEY}
MINIO_SECRET_KEY=${MINIO_SECRET}
MINIO_BUCKET=potspot-photos
PUBLIC_URL=https://${DOMAIN}
CORS_ORIGIN=https://${DOMAIN}
CADDY_HTTP_PORT=${CADDY_HTTP_PORT}
CADDY_HTTPS_PORT=${CADDY_HTTPS_PORT}
GOOGLE_CLIENT_ID=${GOOGLE_CLIENT_ID:-}
GOOGLE_CLIENT_SECRET=${GOOGLE_CLIENT_SECRET:-}
ENVEOF
chmod 600 docker/.env

# Configure Caddy
sed -i "s/potspot.example.com/${DOMAIN}/g" docker/Caddyfile

echo -e "${GREEN}  Secrets generated.${NC}"

# --- Start services ---
echo -e "${YELLOW}Starting services...${NC}"
cd /opt/potspot
CADDY_HTTP_PORT="${CADDY_HTTP_PORT}" CADDY_HTTPS_PORT="${CADDY_HTTPS_PORT}" \
    docker compose -f docker/docker-compose.prod.yml up -d

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot is running!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "  Domain:     ${YELLOW}https://${DOMAIN}${NC}"
if [[ "${CADDY_HTTP_PORT}" != "80" ]]; then
    echo -e "  HTTP port:  ${YELLOW}${CADDY_HTTP_PORT}${NC}"
    echo -e "  HTTPS port: ${YELLOW}${CADDY_HTTPS_PORT}${NC}"
    echo -e "  ${CYAN}Point your reverse proxy at localhost:${CADDY_HTTP_PORT}${NC}"
fi
echo ""
echo -e "  Useful commands:"
echo -e "    cd /opt/potspot"
echo -e "    docker compose -f docker/docker-compose.prod.yml ps"
echo -e "    docker compose -f docker/docker-compose.prod.yml logs -f api"
INITEOF

    chmod +x /opt/potspot/init.sh

    echo "Init script created at /opt/potspot/init.sh"
'

echo -e "${GREEN}  Init script placed.${NC}"

# ============================================================================
# Step 6: Stop CT and convert to template
# ============================================================================
echo -e "${YELLOW}[7/7] Converting to Proxmox template...${NC}"

pct stop "$TEMPLATE_CT_ID" --skiplock

# Wait for stop
for i in {1..10}; do
    if pct status "$TEMPLATE_CT_ID" 2>/dev/null | grep -q "stopped"; then
        break
    fi
    sleep 2
done

# Convert to template
pct template "$TEMPLATE_CT_ID"

echo -e "${GREEN}  Template created.${NC}"

# ============================================================================
# Summary
# ============================================================================
echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot Template Ready!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "  Template ID:   ${YELLOW}${TEMPLATE_CT_ID}${NC}"
echo -e "  Template name: ${YELLOW}${TEMPLATE_NAME}${NC}"
echo ""
echo -e "  ${GREEN}To deploy a new PotSpot instance:${NC}"
echo ""
echo -e "  ${CYAN}# 1. Clone the template${NC}"
echo -e "  pct clone ${TEMPLATE_CT_ID} <NEW_ID> --hostname my-potspot"
echo ""
echo -e "  ${CYAN}# 2. Start the new container${NC}"
echo -e "  pct start <NEW_ID>"
echo ""
echo -e "  ${CYAN}# 3. Run first-boot setup (generates secrets, starts services)${NC}"
echo -e "  pct enter <NEW_ID>"
echo -e "  cd /opt/potspot && sudo ./init.sh"
echo ""
echo -e "  ${CYAN}# Or non-interactively:${NC}"
echo -e "  pct exec <NEW_ID> -- env DOMAIN=potspot.example.com bash /opt/potspot/init.sh"
echo ""
echo -e "  ${YELLOW}Notes:${NC}"
echo -e "  - Each clone gets fresh, unique secrets"
echo -e "  - Docker images are pre-built -- services start in seconds"
echo -e "  - Set CADDY_HTTP_PORT=80 if you are not behind a reverse proxy"
