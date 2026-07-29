#!/usr/bin/env bash

# Copyright (c) 2021-2026 community-scripts ORG
# Author: cpntodd
# License: MIT | https://github.com/community-scripts/ProxmoxVE/raw/main/LICENSE
# Source: https://github.com/cpntodd/PotSpot

# Source common functions from community-scripts
if ! command -v curl &>/dev/null; then
  printf "\r\e[2K%b" '\033[93m Setup Source \033[m' >&2
  apt-get update >/dev/null 2>&1
  apt-get install -y curl >/dev/null 2>&1
fi
source <(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/core.func)
source <(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/tools.func)
source <(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/error_handler.func)
source <(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/api.func) 2>/dev/null || true
declare -f init_tool_telemetry &>/dev/null && init_tool_telemetry "potspot" "addon"

# Enable error handling
set -Eeuo pipefail
trap 'error_handler' ERR

# ==============================================================================
# CONFIGURATION
# ==============================================================================
APP="PotSpot"
APP_TYPE="addon"
INSTALL_PATH="/opt/potspot"
COMPOSE_FILE="${INSTALL_PATH}/docker/docker-compose.prod.yml"
ENV_FILE="${INSTALL_PATH}/docker/.env"
CADDY_FILE="${INSTALL_PATH}/docker/Caddyfile"
REPO_URL="https://github.com/cpntodd/PotSpot.git"

# Initialize all core functions (colors, formatting, icons, STD mode)
load_functions

# ==============================================================================
# HEADER
# ==============================================================================
function header_info {
  clear
  cat <<"EOF"
  _____     _   _____             _
  |  _  |___| |_|   __|___ ___ _ _| |_
  |   __| . |  _|__   | . |_ -| | |  _|
  |__|  |___|_| |_____|  _|___|___|_|
                      |_|
  Community Cannabis Strain Catalog
EOF
}

# ==============================================================================
# UNINSTALL
# ==============================================================================
function uninstall() {
  msg_info "Uninstalling ${APP}"
  if [[ -f "$COMPOSE_FILE" ]]; then
    msg_info "Stopping and removing Docker containers"
    cd "$INSTALL_PATH"
    $STD docker compose -f "$COMPOSE_FILE" down --volumes --remove-orphans
    msg_ok "Stopped and removed Docker containers"
  fi
  rm -rf "$INSTALL_PATH"
  rm -f "/usr/local/bin/update_potspot"
  msg_ok "${APP} has been uninstalled"
}

# ==============================================================================
# UPDATE
# ==============================================================================
function update() {
  msg_info "Pulling latest ${APP} changes"
  cd "$INSTALL_PATH"
  $STD git pull origin main
  msg_ok "Pulled latest changes"

  msg_info "Rebuilding and restarting ${APP}"
  $STD docker compose -f "$COMPOSE_FILE" up -d --build
  msg_ok "Restarted ${APP}"
  msg_ok "Updated successfully"
  exit
}

# ==============================================================================
# CHECK DOCKER
# ==============================================================================
function check_docker() {
  if ! command -v docker &>/dev/null; then
    msg_error "Docker is not installed. Please use the Docker LXC template first. Exiting."
    exit 10
  fi
  if ! docker compose version &>/dev/null; then
    msg_error "Docker Compose plugin is not available. Please install it before running this script. Exiting."
    exit 10
  fi
  msg_ok "Docker $(docker --version | cut -d' ' -f3 | tr -d ',') and Docker Compose are available"
}

# ==============================================================================
# INSTALL
# ==============================================================================
function install() {
  check_docker

  # Prompt for domain
  if [[ -z "${DOMAIN:-}" ]]; then
    echo -e "${TAB}${YW}Enter your domain name (e.g. potspot.example.com):${CL}"
    read -r DOMAIN
    if [[ -z "$DOMAIN" ]]; then
      msg_error "Domain is required. Exiting."
      exit 1
    fi
  fi

  # Prompt for Google OAuth (optional)
  echo -e "${TAB}${YW}Enter Google OAuth Client ID (leave blank to skip):${CL}"
  read -r GOOGLE_CLIENT_ID
  echo -e "${TAB}${YW}Enter Google OAuth Client Secret (leave blank to skip):${CL}"
  read -r GOOGLE_CLIENT_SECRET

  msg_info "Cloning ${APP} repository"
  if [[ -d "$INSTALL_PATH" ]]; then
    rm -rf "$INSTALL_PATH"
  fi
  $STD git clone "$REPO_URL" "$INSTALL_PATH"
  msg_ok "Cloned to ${INSTALL_PATH}"

  msg_info "Generating secure secrets"
  local DB_PASSWORD JWT_SECRET JWT_REFRESH MINIO_KEY MINIO_SECRET
  DB_PASSWORD=$(openssl rand -hex 16)
  JWT_SECRET=$(openssl rand -hex 32)
  JWT_REFRESH=$(openssl rand -hex 32)
  MINIO_KEY=$(openssl rand -hex 16)
  MINIO_SECRET=$(openssl rand -hex 16)

  cat > "$ENV_FILE" << ENVEOF
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
GOOGLE_CLIENT_ID=${GOOGLE_CLIENT_ID:-}
GOOGLE_CLIENT_SECRET=${GOOGLE_CLIENT_SECRET:-}
ENVEOF
  chmod 600 "$ENV_FILE"
  msg_ok "Generated secrets"

  msg_info "Configuring Caddy for ${DOMAIN}"
  sed -i "s/potspot.example.com/${DOMAIN}/g" "$CADDY_FILE"
  msg_ok "Configured Caddy"

  msg_info "Building and starting ${APP}"
  cd "$INSTALL_PATH"
  $STD docker compose -f "$COMPOSE_FILE" up -d --build
  msg_ok "Started ${APP}"

  # Create update script
  msg_info "Creating update script"
  cat <<'UPDATEEOF' >/usr/local/bin/update_potspot
#!/usr/bin/env bash
# PotSpot Update Script
type=update bash -c "$(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/tools/addon/potspot.sh)"
UPDATEEOF
  chmod +x /usr/local/bin/update_potspot
  msg_ok "Created update script (/usr/local/bin/update_potspot)"

  # Wait for services
  msg_info "Waiting for services to start (this may take 1-2 minutes)..."
  sleep 15
  for i in {1..12}; do
    if curl -sf http://localhost:3000/api/v1/strains?per_page=1 > /dev/null 2>&1; then
      msg_ok "API is responding"
      break
    fi
    sleep 10
  done

  echo ""
  msg_ok "${APP} is reachable at: ${BL}https://${DOMAIN}${CL}"
  echo ""
  echo -e "${TAB}${INFO} Useful commands:"
  echo -e "${TAB}  cd ${INSTALL_PATH}"
  echo -e "${TAB}  docker compose -f docker/docker-compose.prod.yml ps"
  echo -e "${TAB}  docker compose -f docker/docker-compose.prod.yml logs -f api"
  echo -e "${TAB}  update_potspot   # Update to latest version"
  echo ""
  msg_warn "Ensure DNS for ${DOMAIN} points to this server's IP for TLS to work."
}

# ==============================================================================
# MAIN
# ==============================================================================

# Handle type=update (called from update script)
if [[ "${type:-}" == "update" ]]; then
  header_info
  if [[ -f "$COMPOSE_FILE" ]]; then
    update
  else
    msg_error "${APP} is not installed. Nothing to update."
    exit 233
  fi
  exit 0
fi

header_info
get_lxc_ip

# Check if already installed
if [[ -f "$COMPOSE_FILE" ]]; then
  msg_warn "${APP} is already installed at ${INSTALL_PATH}."
  echo ""
  echo -n "${TAB}Uninstall ${APP}? (y/N): "
  read -r uninstall_prompt
  if [[ "${uninstall_prompt,,}" =~ ^(y|yes)$ ]]; then
    uninstall
    exit 0
  fi
  echo -n "${TAB}Update ${APP}? (y/N): "
  read -r update_prompt
  if [[ "${update_prompt,,}" =~ ^(y|yes)$ ]]; then
    update
    exit 0
  fi
  msg_warn "No action selected. Exiting."
  exit 0
fi

# Fresh installation
msg_warn "${APP} is not installed."
echo ""
echo -e "${TAB}${INFO} This will install:"
echo -e "${TAB}  - Rust/Axum API server"
echo -e "${TAB}  - PostgreSQL 16 (database)"
echo -e "${TAB}  - MinIO (photo storage)"
echo -e "${TAB}  - Caddy (reverse proxy + auto-TLS)"
echo -e "${TAB}  - SvelteKit web frontend"
echo ""

echo -n "${TAB}Install ${APP}? (y/N): "
read -r install_prompt
if [[ "${install_prompt,,}" =~ ^(y|yes)$ ]]; then
  install
else
  msg_warn "Installation cancelled. Exiting."
  exit 0
fi
