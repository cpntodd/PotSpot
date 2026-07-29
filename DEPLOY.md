# PotSpot -- Production Deployment Guide

## Proxmox LXC (Recommended)

Three deployment paths, from fastest to most customizable:

### Option A: Pre-built Template (fastest -- seconds to deploy)

Build the template once, then clone infinite instances with zero build time.
Docker images are pre-compiled -- services start in seconds.

**Build the template (one-time, on Proxmox host):**
```bash
sudo bash deploy/build-template.sh
```

**Deploy a clone from the template:**
```bash
# Clone (template ID defaults to 900)
pct clone 900 201 --hostname my-potspot --full 1
pct start 201

# Run first-boot setup (generates fresh secrets, starts services)
pct exec 201 -- env DOMAIN=potspot.example.com bash /opt/potspot/init.sh
```

The init script prompts for your domain, optionally OAuth credentials, generates unique secrets, and starts the stack. Nothing is shared between clones.

### Option B: Community-Scripts one-liner

After creating a Docker LXC via [community-scripts.org](https://community-scripts.org/scripts):

```bash
# From inside your Docker LXC:
bash -c "$(curl -fsSL https://raw.githubusercontent.com/cpntodd/PotSpot/main/deploy/potspot.sh)"
```

This interactive script follows the community-scripts pattern:
- Checks/installs Docker and Docker Compose
- Prompts for your domain name and Caddy ports
- Optionally configures Google OAuth
- Generates secure random secrets
- Clones the repo, builds and starts the full stack
- Creates an `update_potspot` command for future updates
- Supports install / update / uninstall workflows

### Option C: Automated Proxmox CT creation

From your Proxmox host, creates a fresh Debian 12 LXC and deploys PotSpot into it:

```bash
export DOMAIN=potspot.yourdomain.com
sudo bash deploy/proxmox-deploy.sh
```

### Option D: Deploy inside an existing LXC/VPS

SSH into your container and run:
```bash
curl -sSL https://raw.githubusercontent.com/cpntodd/PotSpot/main/deploy/setup.sh | sudo DOMAIN=potspot.yourdomain.com bash
```

### After deployment (Options B/C/D)

1. Point DNS A record for your domain to the container's IP
2. Wait for Let's Encrypt to issue the certificate (automatic via Caddy, ~30s)
3. Visit `https://yourdomain.com`

### Container specs

| Resource | Minimum | Recommended |
|---|---|---|
| CPU | 1 core | 2-4 cores |
| RAM | 1 GB | 2-4 GB |
| Disk | 10 GB | 20+ GB |
| OS | Debian 12 | Debian 12 |

---

## Native Debian Install (No Docker)

Installs PotSpot directly on the OS: Rust, Node.js, PostgreSQL, Caddy, systemd.
Best for bare-metal Debian 12 servers or LXCs where you don't want container overhead.

```bash
curl -sSL https://raw.githubusercontent.com/cpntodd/PotSpot/main/deploy/native-install.sh | sudo bash
```

With a pre-set domain:
```bash
sudo DOMAIN=potspot.example.com bash deploy/native-install.sh
```

What it does:
- Installs Rust (rustup), Node.js 20.x, PostgreSQL 16, Caddy
- Creates `potspot` system user, database, and systemd service
- Builds the Rust API binary and SvelteKit frontend
- Installs MinIO for S3-compatible photo storage
- Generates secrets, configures Caddy reverse proxy
- Starts everything via systemd

MinIO Console is available at `http://localhost:9001` (credentials in `/opt/potspot/.env`).

### Managing the native install

```bash
systemctl status potspot-api    # API health
systemctl status potspot-minio  # Object storage
systemctl status caddy          # Reverse proxy
systemctl status postgresql     # Database
journalctl -u potspot-api -f    # API logs
journalctl -u potspot-minio -f  # MinIO logs
journalctl -u caddy -f          # Caddy logs

# Updates
cd /opt/potspot && git pull origin main
cd backend && cargo build --release
cd ../web && npm ci && npm run build
systemctl restart potspot-api caddy
```

---

## Manual VPS Deployment (Docker)
- Docker and Docker Compose installed
- A domain name pointed to your VPS
- Ports 80 and 443 open in firewall

## Quick Deploy (Docker)

```bash
# 1. Clone the repository
git clone https://github.com/cpntodd/PotSpot.git
cd PotSpot

# 2. Create production environment file
cp backend/.env.example backend/.env.production
# Edit backend/.env.production with SECURE random values for JWT secrets

# 3. Create Docker environment file
cat > docker/.env << 'EOF'
POSTGRES_USER=potspot
POSTGRES_PASSWORD=<generate-a-strong-password>
POSTGRES_DB=potspot
DATABASE_URL=postgres://potspot:<password>@db:5432/potspot
JWT_SECRET=<generate-64-char-random-string>
JWT_REFRESH_SECRET=<generate-another-64-char-random-string>
MINIO_ACCESS_KEY=<generate-access-key>
MINIO_SECRET_KEY=<generate-secret-key>
MINIO_BUCKET=potspot-photos
GOOGLE_CLIENT_ID=<your-google-oauth-client-id>
GOOGLE_CLIENT_SECRET=<your-google-oauth-client-secret>
EOF

# 4. Update Caddyfile with your domain
sed -i 's/potspot.example.com/your-domain.com/g' docker/Caddyfile

# 5. Build and start
docker compose -f docker/docker-compose.prod.yml up -d --build

# 6. Verify
curl https://your-domain.com/api/v1/strains
```

## Architecture

```
Internet --> :443 (Caddy) --> :3000 (Rust API)
                          --> :80 redirect to :443
                |
        Internal Docker network:
        - api:3000 (Rust/Axum)
        - db:5432 (PostgreSQL)
        - minio:9000 (Object storage)
```

## Backup

```bash
# Daily PostgreSQL backup (add to crontab)
docker exec potspot-db-1 pg_dump -U potspot potspot > /backup/potspot_$(date +%Y%m%d).sql

# MinIO backup
docker run --rm --network potspot_default \
  -v /backup:/backup \
  minio/mc mirror local/potspot-photos /backup/photos/
```

## Updates

```bash
git pull origin main
docker compose -f docker/docker-compose.prod.yml up -d --build
```

## Monitoring

```bash
# Check service health
docker compose -f docker/docker-compose.prod.yml ps

# View logs
docker compose -f docker/docker-compose.prod.yml logs -f api

# Database stats
docker exec potspot-db-1 psql -U potspot -d potspot -c "
  SELECT 'users' AS entity, COUNT(*) FROM users
  UNION ALL SELECT 'strains', COUNT(*) FROM public_strains
  UNION ALL SELECT 'comments', COUNT(*) FROM comments
  UNION ALL SELECT 'ratings', COUNT(*) FROM strain_ratings;
"
```

## Security Checklist

- [ ] Change all default passwords in `.env`
- [ ] Use `openssl rand -hex 32` for JWT secrets
- [ ] Configure firewall: only 80/443 open
- [ ] Set up unattended-upgrades for OS patches
- [ ] Enable Docker daemon log rotation
- [ ] Configure fail2ban for SSH
- [ ] Test backup restoration process
- [ ] Set up monitoring alerts for disk space

## Scaling

For >10k users:
- Move PostgreSQL to a managed service (e.g., DigitalOcean Managed DB)
- Add a second API container behind a load balancer
- Move MinIO to S3-compatible cloud storage (Backblaze B2, AWS S3, Cloudflare R2)
- Add Redis for session/notification caching
