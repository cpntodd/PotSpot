#!/bin/bash
# PotSpot -- Proxmox LXC Deployment Script
# Run this ON THE PROXMOX HOST to create and deploy a ready-to-run LXC container.
#
# Quick start (recommended: use community-scripts Docker LXC):
#   1. Run: bash -c "$(wget -qLO - https://github.com/community-scripts/ProxmoxVE/raw/main/ct/docker.sh)"
#   2. Inside the new CT: curl -sSL https://raw.githubusercontent.com/cpntodd/PotSpot/main/deploy/setup.sh | sudo DOMAIN=your.domain.com bash
#
# Alternative (this script -- creates Ubuntu CT from scratch):
#   sudo bash deploy/proxmox-deploy.sh
#
# What it does:
#   1. Creates an Ubuntu 24.04 LXC container (2 CPU, 2GB RAM, 20GB disk)
#   2. Installs Docker + Docker Compose inside the container
#   3. Clones the PotSpot repo
#   4. Generates secure random secrets
#   5. Starts the full stack (API, PostgreSQL, MinIO, Caddy)

set -euo pipefail

# ============================================================================
# Configuration -- edit these before running
# ============================================================================
CT_ID="${CT_ID:-200}"                    # LXC container ID
CT_HOSTNAME="${CT_HOSTNAME:-potspot}"     # Container hostname
CT_ROOT_PASSWORD="${CT_ROOT_PASSWORD:-}"  # Leave empty to auto-generate
DOMAIN="${DOMAIN:-potspot.example.com}"   # Your domain pointing to the CT's IP
CT_IP="${CT_IP:-dhcp}"                   # IP address or "dhcp"
CT_MEMORY="${CT_MEMORY:-2048}"           # MB
CT_SWAP="${CT_SWAP:-512}"                # MB
CT_CORES="${CT_CORES:-2}"                # CPU cores
CT_DISK="${CT_DISK:-20}"                 # GB
STORAGE="${STORAGE:-local-lvm}"          # Proxmox storage pool
UBUNTU_TEMPLATE="${UBUNTU_TEMPLATE:-local:vztmpl/ubuntu-24.04-standard_24.04-1_amd64.tar.zst}"

# ============================================================================
# Colors
# ============================================================================
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'

echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot -- Proxmox LXC Deployer${NC}"
echo -e "${GREEN}============================================${NC}"

# ============================================================================
# Step 1: Download Ubuntu template if needed
# ============================================================================
echo -e "${YELLOW}[1/6] Checking Ubuntu 24.04 template...${NC}"
if ! pveam list "$STORAGE" 2>/dev/null | grep -q "ubuntu-24.04"; then
    echo "  Downloading template..."
    pveam update
    pveam download local ubuntu-24.04-standard_24.04-1_amd64.tar.zst
fi
echo -e "${GREEN}  Template ready.${NC}"

# ============================================================================
# Step 2: Create LXC container
# ============================================================================
echo -e "${YELLOW}[2/6] Creating LXC container (ID: $CT_ID)...${NC}"

# Generate root password if not provided
if [ -z "$CT_ROOT_PASSWORD" ]; then
    CT_ROOT_PASSWORD=$(openssl rand -base64 16)
    echo -e "  ${GREEN}Generated root password: $CT_ROOT_PASSWORD${NC}"
    echo "  Save this password! You'll need it to access the container."
fi

# Destroy existing container with same ID if it exists
if pct status "$CT_ID" &>/dev/null; then
    echo "  Container $CT_ID already exists. Destroying..."
    pct stop "$CT_ID" --skiplock 2>/dev/null || true
    pct destroy "$CT_ID" --purge 2>/dev/null || true
fi

# Create the container
pct create "$CT_ID" "$UBUNTU_TEMPLATE" \
    --hostname "$CT_HOSTNAME" \
    --memory "$CT_MEMORY" \
    --swap "$CT_SWAP" \
    --cores "$CT_CORES" \
    --rootfs "$STORAGE:$CT_DISK" \
    --net0 "name=eth0,bridge=vmbr0,ip=$CT_IP" \
    --password "$CT_ROOT_PASSWORD" \
    --features "nesting=1" \
    --unprivileged 1 \
    --onboot 1

echo -e "${GREEN}  Container created.${NC}"

# ============================================================================
# Step 3: Start container and install Docker
# ============================================================================
echo -e "${YELLOW}[3/6] Starting container and installing Docker...${NC}"
pct start "$CT_ID"

# Wait for networking
echo "  Waiting for container network..."
sleep 10

# Install Docker + dependencies
pct exec "$CT_ID" -- bash -c '
    set -e
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq curl git ca-certificates > /dev/null 2>&1

    # Install Docker
    curl -fsSL https://get.docker.com | sh > /dev/null 2>&1
    systemctl enable --now docker

    # Verify
    docker --version
'

echo -e "${GREEN}  Docker installed.${NC}"

# ============================================================================
# Step 4: Clone PotSpot and configure
# ============================================================================
echo -e "${YELLOW}[4/6] Cloning PotSpot and generating secrets...${NC}"

pct exec "$CT_ID" -- bash -c "
    set -e
    cd /opt
    git clone https://github.com/cpntodd/PotSpot.git potspot
    cd potspot

    # Generate secure random secrets
    cat > docker/.env << 'ENVEOF'
POSTGRES_USER=potspot
POSTGRES_PASSWORD=$(openssl rand -hex 16)
POSTGRES_DB=potspot
DATABASE_URL=postgres://potspot:\$POSTGRES_PASSWORD@db:5432/potspot
JWT_SECRET=$(openssl rand -hex 32)
JWT_REFRESH_SECRET=$(openssl rand -hex 32)
MINIO_ACCESS_KEY=$(openssl rand -hex 16)
MINIO_SECRET_KEY=$(openssl rand -hex 16)
MINIO_BUCKET=potspot-photos
PUBLIC_URL=https://$DOMAIN
CORS_ORIGIN=https://$DOMAIN
ENVEOF

    # Update Caddyfile with the real domain
    sed -i \"s/potspot.example.com/$DOMAIN/g\" docker/Caddyfile

    echo 'Secrets generated.'
"

echo -e "${GREEN}  Repository cloned and configured.${NC}"

# ============================================================================
# Step 5: Build and start
# ============================================================================
echo -e "${YELLOW}[5/6] Building and starting PotSpot...${NC}"

pct exec "$CT_ID" -- bash -c '
    set -e
    cd /opt/potspot
    docker compose -f docker/docker-compose.prod.yml up -d --build
'

echo -e "${GREEN}  Containers starting...${NC}"
sleep 10

# ============================================================================
# Step 6: Verify
# ============================================================================
echo -e "${YELLOW}[6/6] Verifying deployment...${NC}"

pct exec "$CT_ID" -- bash -c '
    echo "--- Container Status ---"
    cd /opt/potspot
    docker compose -f docker/docker-compose.prod.yml ps

    echo ""
    echo "--- API Health ---"
    curl -s http://localhost:3000/api/v1/strains?per_page=1 | head -c 200 || echo "API not ready yet (may need a moment)"
'

# ============================================================================
# Summary
# ============================================================================
echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  PotSpot Deployed!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "  Container ID:    ${YELLOW}$CT_ID${NC}"
echo -e "  Hostname:        ${YELLOW}$CT_HOSTNAME${NC}"
echo -e "  Domain:          ${YELLOW}https://$DOMAIN${NC}"
echo -e "  Root password:   ${YELLOW}$CT_ROOT_PASSWORD${NC}"
echo ""
echo -e "  ${GREEN}Next steps:${NC}"
echo "  1. Point DNS for $DOMAIN to this container's IP"
echo "  2. Ensure ports 80 and 443 are reachable"
echo "  3. SSH into container: pct enter $CT_ID"
echo "  4. View logs: docker compose -f /opt/potspot/docker/docker-compose.prod.yml logs -f"
echo "  5. Set up OAuth: edit /opt/potspot/docker/.env and add Google/Facebook client credentials"
echo ""
echo -e "  ${YELLOW}Save the root password above!${NC}"
