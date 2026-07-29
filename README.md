# PotSpot

A community-driven cannabis strain catalog with web and Android interfaces.

## Stack

| Layer | Technology |
|---|---|
| Backend API | Rust 1.80+ (Axum) |
| Database | PostgreSQL 16 |
| Object Storage | MinIO (S3-compatible) |
| Web Frontend | SvelteKit |
| Android App | Kotlin + Jetpack Compose |
| Reverse Proxy | Caddy |
| Deployment | Docker Compose |

## Quick Start (Development)

```bash
# Clone and enter the project
git clone https://github.com/cpntodd/PotSpot.git
cd PotSpot

# Start all services
docker compose -f docker/docker-compose.dev.yml up -d

# The API will be available at http://localhost:3000
# MinIO console at http://localhost:9001
# PostgreSQL at localhost:5432
```

### Without Docker

**Backend:**
```bash
cd backend
cp .env.example .env    # Edit values as needed
cargo run
```

**Frontend:**
```bash
cd web
npm install
npm run dev             # http://localhost:5173
```

## Features

- **Public catalog** of cannabis strains with terpene profiles, effect tags, and community ratings
- **Private vault** for tracking personal strains privately
- **Collaborative vetting** -- trusted users review edits to maintain catalog quality
- **Threaded comments** with upvote/downvote (Reddit-style)
- **Similar strain recommendations** based on terpenes, effects, and user behavior
- **Full-text search** with filters by type, terpenes, effects, THC/CBD range, and rating
- **Version history** for all public strain edits
- **Android app** with full offline support and background sync
- **OAuth login** via Google, Facebook, Microsoft, and Apple
- **No third-party trackers** -- privacy-respecting analytics

## Architecture

See [DESIGN.md](DESIGN.md) for the full architecture document, data model, and development roadmap.

## License

PotSpot is licensed under the GNU General Public License v3.0 or later. See [LICENSE](LICENSE) for details.

## Disclaimer

Cannabis legality varies by jurisdiction. It is your responsibility to understand and comply with local laws. PotSpot does not facilitate, encourage, or enable the purchase, sale, or distribution of cannabis. All strain data is user-contributed and not verified for accuracy. Users must be 18 years of age or older.

---

(c) 2026 cpntodd
