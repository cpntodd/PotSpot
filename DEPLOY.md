# PotSpot -- Production Deployment Guide

## Proxmox LXC (Recommended -- 5 minute deploy)

### Easiest: Community-Scripts style one-liner

After creating a Docker LXC via [community-scripts.org](https://community-scripts.org/scripts):

```bash
# From inside your Docker LXC:
bash -c "$(curl -fsSL https://raw.githubusercontent.com/cpntodd/PotSpot/main/deploy/potspot.sh)"
```

This interactive script follows the community-scripts pattern:
- Checks Docker/Docker Compose
- Prompts for your domain name
- Optionally configures Google OAuth
- Generates secure random secrets
- Clones the repo, builds and starts the full stack
- Creates an `update_potspot` command for future updates
- Supports install / update / uninstall workflows

### Option A: Use Community-Scripts Docker LXC

From your Proxmox host, run:
```bash
# Set your domain
export DOMAIN=potspot.yourdomain.com

# Run the automated deployer (creates CT + installs + starts)
sudo bash deploy/proxmox-deploy.sh
```

This creates a Debian 12 LXC container (2 CPU, 2GB RAM, 20GB disk), installs Docker, clones the repo, generates secrets, and starts the full stack.

### Option B: Deploy inside an existing LXC/VPS

SSH into your container and run:
```bash
curl -sSL https://raw.githubusercontent.com/cpntodd/PotSpot/main/deploy/setup.sh | sudo DOMAIN=potspot.yourdomain.com bash
```

### After deployment

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

## Manual VPS Deployment
- Docker and Docker Compose installed
- A domain name pointed to your VPS
- Ports 80 and 443 open in firewall

## Quick Deploy

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
