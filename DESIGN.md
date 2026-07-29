# PotSpot -- Architecture & Design Document

## 1. Overview

PotSpot is a community-driven cannabis strain catalog with a web frontend, a companion Android app, and a shared backend. Users maintain a **private vault** of strains they create or bookmark. Strains can be **pushed** from the private vault to the **public catalog**, where they go through a post-edit collaborative vetting process. All contributions to the public catalog are anonymous -- no user identity is attached to public strain data.

### Core Principles

- **Privacy-first.** No plaintext secrets. Public strain data is fully decoupled from user identity. Email/password auth uses argon2id. All traffic is TLS-encrypted. Android app pins certificates.
- **Offline-first Android.** The app is fully functional without a network connection. Public catalog metadata, saved strains, and private vault data are synced to a local SQLite database.
- **Lightweight & fast.** Rust (Axum) backend, SvelteKit frontend, native Kotlin Android. No heavy runtimes, no unnecessary abstractions.
- **Self-hosted.** Docker Compose for development and production. Single VPS for public release. No cloud vendor lock-in.

---

## 2. Tech Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Backend API** | Rust 1.80+ with Axum | C-level performance, memory safety, mature async ecosystem, compile-time query checking via sqlx |
| **Database** | PostgreSQL 16 | Full-text search, JSONB for semi-structured data, window functions for version history, row-level security ready |
| **Object Storage** | MinIO (S3-compatible) | Purpose-built for photos, single-binary deployment, no vendor lock-in, presigned URLs |
| **Web Frontend** | SvelteKit (static SPA mode) | Compiles to vanilla JS -- no virtual DOM, small bundle, SSR optional, professional look |
| **Android App** | Kotlin + Jetpack Compose | Native performance, Room (SQLite) for offline, WorkManager for background sync, CameraX for photo capture |
| **Reverse Proxy** | Caddy | Automatic TLS via Let's Encrypt, simple Caddyfile config, HTTP/2 |
| **Containerization** | Docker Compose (4 services) | Identical dev/prod environments, persistent volumes, single-command deploy |

### Why Not C?

Writing a production web server in C means implementing HTTP/2 parsing, TLS, JSON serialization, database drivers, async I/O, and OAuth2 flows from scratch -- or depending on thinly-maintained libraries for each. The security surface area alone (buffer overflows, use-after-free in async code, format string bugs in template rendering) makes it unsuitable for an internet-facing service handling user accounts. Rust provides the same zero-cost abstraction philosophy as C with compile-time guarantees that eliminate entire classes of vulnerabilities.

---

## 3. System Architecture

```
+-------------------+       +-------------------+       +-------------------+
|   Android App     |       |   Web Frontend    |       |   External OAuth  |
| (Kotlin/Compose)  |       |   (SvelteKit)     |       | (Google/FB/MS/    |
|                   |       |                   |       |  Apple)           |
+--------+----------+       +--------+----------+       +--------+----------+
         |                         |                           |
         | HTTPS + JWT             | HTTPS + JWT               | OAuth2/OIDC
         | (cert pinning)          | (session cookie)          |
         |                         |                           |
         v                         v                           v
+-------------------------------------------------------------------+
|                          Caddy (TLS termination)                  |
+-------------------------------+-----------------------------------+
                                |
                                v
+-------------------------------------------------------------------+
|                      Rust / Axum API Server                       |
|                                                                    |
|  +------------------+  +------------------+  +------------------+ |
|  | Auth Controller  |  | Strain Controller|  | Vault Controller | |
|  | - register       |  | - search         |  | - list           | |
|  | - login          |  | - detail         |  | - create/edit    | |
|  | - oauth callback |  | - rate           |  | - push to public | |
|  | - refresh token  |  | - similar        |  | - save from pub  | |
|  +------------------+  +------------------+  +------------------+ |
|                                                                    |
|  +------------------+  +------------------+  +------------------+ |
|  | Comment Ctrl     |  | Vetting Ctrl     |  | Photo Controller | |
|  | - thread view    |  | - pending queue  |  | - upload         | |
|  | - post/reply     |  | - approve/reject |  | - thumbnail gen  | |
|  | - vote           |  | - revision hist  |  | - presigned URL  | |
|  +------------------+  +------------------+  +------------------+ |
+-------------------------------------------------------------------+
         |                          |
         v                          v
+------------------+     +------------------+
|   PostgreSQL 16  |     |   MinIO (S3)     |
|                  |     |                  |
| - users          |     | - strain photos  |
| - strains (pub)  |     | - thumbnails     |
| - private_strains|     | - review photos  |
| - terpenes       |     |                  |
| - effects        |     |                  |
| - comments       |     |                  |
| - ratings        |     |                  |
| - revisions      |     |                  |
| - notifications  |     |                  |
+------------------+     +------------------+
```

---

## 4. Data Model

### 4.1 Entity-Relationship Summary

```
users 1---* private_strains
users 1---* user_saved_strains
users 1---* ratings
users 1---* comments
users 1---* comment_votes
users 1---* notifications
users 1---* strain_photos (review photos)

private_strains *---1 public_strains (nullable, set on push)

public_strains 1---* strain_terpenes
public_strains 1---* strain_effects
public_strains 1---* strain_photos (primary photo)
public_strains 1---* ratings
public_strains 1---* comments
public_strains 1---* strain_revisions

terpenes 1---* strain_terpenes
effects 1---* strain_effects

comments 1---* comments (self-referencing parent for threading)
comments 1---* comment_votes
```

### 4.2 Table Definitions

#### `users`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | Generated server-side |
| email | TEXT UNIQUE | Lowercase, trimmed |
| password_hash | TEXT | argon2id, nullable if OAuth-only |
| display_name | TEXT | User-visible name |
| role | ENUM('user','vetter','admin') | Default 'user' |
| age_verified | BOOLEAN | True if OAuth provided age or DOB entered |
| date_of_birth | DATE | Self-declared honor system |
| created_at | TIMESTAMPTZ | |
| updated_at | TIMESTAMPTZ | |
| deleted_at | TIMESTAMPTZ | Soft delete |

#### `user_oauth_accounts`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| user_id | UUID (FK -> users) | |
| provider | ENUM('google','facebook','microsoft','apple') | |
| provider_user_id | TEXT | OAuth provider's ID for this user |
| access_token | TEXT (encrypted) | Encrypted at rest |
| refresh_token | TEXT (encrypted) | Encrypted at rest |
| created_at | TIMESTAMPTZ | |
| UNIQUE(provider, provider_user_id) | | |

#### `terpenes` (fixed picklist, seed data)
| Column | Type | Notes |
|---|---|---|
| id | SMALLINT (PK) | |
| name | TEXT UNIQUE | e.g. "Myrcene" |
| icon | TEXT | Emoji or icon identifier, e.g. "🌿" |
| description | TEXT | Brief explanation |

Seed data: Myrcene (🌿), Limonene (🍋), Pinene (🌲), Caryophyllene (🌶️), Linalool (💐), Humulene (🍺), Terpinolene (🌸), Ocimene (🌱), Valencene (🍊), Geraniol (🌹), Bisabolol (🌸), Eucalyptol (🍃), Nerolidol (🌳), Phytol (🌿), Camphene (🌲), Phellandrene (🌿), Carene (🌲), Sabinene (🌿), Terpinene (🌿), Borneol (🌿)

#### `effects` (fixed taxonomy, seed data)
| Column | Type | Notes |
|---|---|---|
| id | SMALLINT (PK) | |
| name | TEXT UNIQUE | e.g. "Relaxed" |
| category | TEXT | e.g. "Positive", "Negative", "Medical" |

Seed data categories:
- **Positive:** Relaxed, Euphoric, Happy, Uplifted, Creative, Focused, Energetic, Talkative, Giggly, Hungry, Aroused, Sleepy, Tingly
- **Negative:** Anxious, Paranoid, Dizzy, Dry Mouth, Dry Eyes, Headache, Lethargic
- **Medical:** Pain Relief, Stress Relief, Anxiety Relief, Insomnia Relief, Appetite Stimulant, Anti-inflammatory, Muscle Spasm Relief, Nausea Relief, Depression Relief, PTSD Relief, Seizure Management, Glaucoma Relief

#### `public_strains`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| name | TEXT | Strain name |
| type | ENUM('sativa','indica','hybrid') | |
| thc_percentage | DECIMAL(5,2) | Nullable, 0.00 - 100.00 |
| cbd_percentage | DECIMAL(5,2) | Nullable |
| description | TEXT | Free-text description |
| color | TEXT | Color description |
| smell | TEXT | Aroma notes |
| flavor | TEXT | Flavor notes |
| breeder | TEXT | Breeder/seed bank name, nullable |
| lineage | TEXT | Parent strain lineage, nullable |
| growing_difficulty | ENUM('easy','moderate','difficult','expert') | Nullable |
| flowering_time_days | SMALLINT | Nullable |
| average_rating | DECIMAL(3,2) | Denormalized, recalculated on new rating |
| rating_count | INTEGER | Denormalized |
| is_active | BOOLEAN | False if removed by admin |
| created_at | TIMESTAMPTZ | |
| updated_at | TIMESTAMPTZ | |
| version | INTEGER | Incremented on each approved edit |

**Crucially: No `user_id` or `created_by` column.** Public strains are anonymous.

#### `private_strains`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| user_id | UUID (FK -> users) | Owner |
| public_strain_id | UUID (FK -> public_strains) | Nullable; set after push to public |
| name | TEXT | |
| type | ENUM('sativa','indica','hybrid') | |
| thc_percentage | DECIMAL(5,2) | |
| cbd_percentage | DECIMAL(5,2) | |
| description | TEXT | |
| color | TEXT | |
| smell | TEXT | |
| flavor | TEXT | |
| breeder | TEXT | |
| lineage | TEXT | |
| growing_difficulty | ENUM | |
| flowering_time_days | SMALLINT | |
| personal_rating | SMALLINT | 1-5, user's own rating |
| personal_notes | TEXT | Private notes, never shared |
| created_at | TIMESTAMPTZ | |
| updated_at | TIMESTAMPTZ | |

#### `user_saved_strains`
| Column | Type | Notes |
|---|---|---|
| user_id | UUID (FK -> users) | |
| strain_id | UUID (FK -> public_strains) | |
| saved_at | TIMESTAMPTZ | |
| PRIMARY KEY(user_id, strain_id) | | |

Saving a public strain to your vault creates a row here. This triggers the strain to be synced to the Android app's offline store.

#### `strain_terpenes`
| Column | Type | Notes |
|---|---|---|
| strain_id | UUID (FK -> public_strains) | |
| terpene_id | SMALLINT (FK -> terpenes) | |
| PRIMARY KEY(strain_id, terpene_id) | | |

#### `strain_effects`
| Column | Type | Notes |
|---|---|---|
| strain_id | UUID (FK -> public_strains) | |
| effect_id | SMALLINT (FK -> effects) | |
| PRIMARY KEY(strain_id, effect_id) | | |

#### `strain_photos`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| strain_id | UUID (FK -> public_strains) | |
| user_id | UUID (FK -> users) | NULLABLE -- NULL for primary photo uploaded by strain creator |
| is_primary | BOOLEAN | Only one primary per strain |
| s3_key | TEXT | MinIO object key |
| thumbnail_s3_key | TEXT | MinIO thumbnail key |
| content_type | TEXT | e.g. "image/webp" |
| file_size_bytes | INTEGER | |
| width | SMALLINT | |
| height | SMALLINT | |
| uploaded_at | TIMESTAMPTZ | |

Constraints:
- One primary photo per strain (enforced by partial unique index)
- Review photos: `is_primary = false`, `user_id` is set
- Primary photo: `is_primary = true`, `user_id` is NULL
- Only one row per strain where `is_primary = true`

#### `private_strain_photos` (photos for private strains)
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| private_strain_id | UUID (FK -> private_strains) | |
| is_primary | BOOLEAN | |
| s3_key | TEXT | |
| thumbnail_s3_key | TEXT | |
| content_type | TEXT | |
| file_size_bytes | INTEGER | |
| width | SMALLINT | |
| height | SMALLINT | |
| uploaded_at | TIMESTAMPTZ | |

#### `strain_ratings`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| strain_id | UUID (FK -> public_strains) | |
| user_id | UUID (FK -> users) | |
| rating | SMALLINT | 1-5 |
| created_at | TIMESTAMPTZ | |
| UNIQUE(strain_id, user_id) | | One rating per user per strain |

#### `strain_revisions` (version history + vetting)
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| strain_id | UUID (FK -> public_strains) | |
| proposed_by | UUID (FK -> users) | User who submitted the change |
| change_summary | TEXT | User-provided summary of what changed |
| old_data | JSONB | Snapshot of strain fields before change |
| new_data | JSONB | Proposed new values |
| status | ENUM('pending','approved','rejected') | |
| vetted_by | UUID (FK -> users) | Vetter who acted on it, nullable |
| vetted_at | TIMESTAMPTZ | Nullable |
| rejection_reason | TEXT | Required if rejected |
| created_at | TIMESTAMPTZ | |

Flow:
1. User edits a public strain (triggers revision with status 'pending')
2. The edit goes live immediately on the public strain
3. Vetters see the pending revision in their queue
4. Vetters can approve (status -> 'approved') or revert (status -> 'rejected', and the public strain is rolled back to `old_data`)

#### `comments`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| strain_id | UUID (FK -> public_strains) | |
| user_id | UUID (FK -> users) | |
| parent_comment_id | UUID (FK -> comments) | NULL for top-level comments |
| body | TEXT | Markdown? Plain text? |
| upvotes | INTEGER | Denormalized counter |
| downvotes | INTEGER | Denormalized counter |
| created_at | TIMESTAMPTZ | |
| updated_at | TIMESTAMPTZ | |
| is_deleted | BOOLEAN | Soft delete, body retained for thread context |

#### `comment_votes`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| comment_id | UUID (FK -> comments) | |
| user_id | UUID (FK -> users) | |
| vote | SMALLINT | 1 (upvote) or -1 (downvote) |
| created_at | TIMESTAMPTZ | |
| UNIQUE(comment_id, user_id) | | |

#### `notifications`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| user_id | UUID (FK -> users) | |
| type | ENUM('comment_reply','comment_vote','vetting_action','strain_approved','strain_rejected') | |
| reference_id | UUID | Polymorphic -- ID of the related entity |
| message | TEXT | Human-readable notification text |
| is_read | BOOLEAN | |
| created_at | TIMESTAMPTZ | |

#### `refresh_tokens`
| Column | Type | Notes |
|---|---|---|
| id | UUID (PK) | |
| user_id | UUID (FK -> users) | |
| token_hash | TEXT UNIQUE | SHA-256 of the actual token |
| device_info | TEXT | User-Agent or device identifier |
| expires_at | TIMESTAMPTZ | |
| created_at | TIMESTAMPTZ | |
| revoked_at | TIMESTAMPTZ | Nullable |

---

## 5. API Design (REST + OpenAPI)

### 5.1 Authentication

```
POST   /api/v1/auth/register          # email + password + display_name + dob
POST   /api/v1/auth/login             # email + password -> JWT pair
POST   /api/v1/auth/refresh           # refresh_token -> new JWT pair
POST   /api/v1/auth/logout            # revoke refresh token
GET    /api/v1/auth/oauth/:provider   # redirect to OAuth provider
GET    /api/v1/auth/oauth/callback    # OAuth callback -> JWT pair
DELETE /api/v1/auth/account           # soft-delete account
```

**JWT strategy:**
- Access token: 15-minute expiry, signed with RS256, contains user_id + role
- Refresh token: 30-day expiry, opaque random string, stored hashed in DB
- Android app stores refresh token in EncryptedSharedPreferences
- Web app stores refresh token in httpOnly, secure, sameSite=strict cookie

### 5.2 Public Strains (authenticated, read)

```
GET    /api/v1/strains                # List with search/filter/pagination
GET    /api/v1/strains/:id            # Detail with terpenes, effects, primary photo URL
GET    /api/v1/strains/:id/similar    # Similar strains
GET    /api/v1/strains/:id/comments   # Threaded comments
GET    /api/v1/strains/:id/revisions  # Version history
```

**Query parameters for `GET /strains`:**
| Param | Type | Example |
|---|---|---|
| q | string | `?q=blue+dream` (full-text search on name, description) |
| type | enum | `?type=indica` |
| terpenes | int[] | `?terpenes=1,3,5` (must have ALL specified) |
| effects | int[] | `?effects=2,7` |
| thc_min | decimal | `?thc_min=15.0` |
| thc_max | decimal | `?thc_max=25.0` |
| cbd_min | decimal | |
| cbd_max | decimal | |
| rating_min | decimal | `?rating_min=4.0` |
| sort | enum | `rating`, `name`, `thc`, `newest` |
| order | enum | `asc`, `desc` |
| page | int | 1-based |
| per_page | int | Default 20, max 100 |

### 5.3 Private Vault (authenticated, owner-only)

```
GET    /api/v1/vault                  # List user's private strains + saved strains
POST   /api/v1/vault                  # Create new private strain
GET    /api/v1/vault/:id              # Detail (only if owned by user)
PUT    /api/v1/vault/:id              # Update private strain
DELETE /api/v1/vault/:id              # Delete private strain
POST   /api/v1/vault/:id/push         # Push private strain to public catalog
POST   /api/v1/vault/:id/push-update  # Push updated fields to public strain
POST   /api/v1/vault/save/:strain_id  # Save a public strain to vault
DELETE /api/v1/vault/save/:strain_id  # Remove saved strain from vault
POST   /api/v1/vault/:id/photo       # Upload photo for private strain
```

### 5.4 Public Strain Contributions (authenticated)

```
PUT    /api/v1/strains/:id            # Edit public strain (creates pending revision)
POST   /api/v1/strains/:id/rate       # Rate 1-5
DELETE /api/v1/strains/:id/rate       # Remove own rating
POST   /api/v1/strains/:id/comments   # Post top-level comment
POST   /api/v1/comments/:id/reply     # Reply to a comment
PUT    /api/v1/comments/:id           # Edit own comment
DELETE /api/v1/comments/:id           # Soft-delete own comment
POST   /api/v1/comments/:id/vote      # Upvote or downvote
DELETE /api/v1/comments/:id/vote      # Remove vote
POST   /api/v1/strains/:id/photos     # Upload review photo (max 1 MB)
```

### 5.5 Vetting (vetter+ role)

```
GET    /api/v1/vetting/queue          # Pending revisions
POST   /api/v1/vetting/:revision_id/approve
POST   /api/v1/vetting/:revision_id/reject   # Body: { reason: "..." }
```

### 5.6 Admin

```
GET    /api/v1/admin/users            # List users
PUT    /api/v1/admin/users/:id/role   # Change role
DELETE /api/v1/admin/strains/:id      # Deactivate public strain
GET    /api/v1/admin/stats            # Dashboard: user count, strain count, etc.
```

### 5.7 Photos

```
POST   /api/v1/photos/upload          # Multipart upload, returns presigned URL
GET    /api/v1/photos/:id             # Redirect to presigned MinIO URL (5-min expiry)
```

### 5.8 Notifications

```
GET    /api/v1/notifications          # List user's notifications (paginated)
POST   /api/v1/notifications/:id/read # Mark as read
POST   /api/v1/notifications/read-all # Mark all as read
```

### 5.9 Sync (Android offline)

```
GET    /api/v1/sync/catalog           # Full catalog metadata (since timestamp)
GET    /api/v1/sync/vault             # User's vault + saved strains (since timestamp)
GET    /api/v1/sync/ratings           # User's ratings (since timestamp)
```

All sync endpoints accept `?since=ISO8601_timestamp` for incremental updates. The initial sync (no `since` param) returns the full dataset.

---

## 6. Authentication & Authorization Flow

### 6.1 Password Auth

1. User registers with email, password, display_name, date_of_birth
2. Password hashed with argon2id (memory: 64MB, iterations: 3, parallelism: 4)
3. On login: verify hash, issue JWT access token (15 min) + refresh token (30 days)
4. Refresh token stored as SHA-256 hash in `refresh_tokens` table
5. On refresh: validate token hash, rotate both tokens, revoke old refresh token

### 6.2 OAuth Flow

1. User clicks "Sign in with Google" etc.
2. Redirect to provider's OAuth2/OIDC endpoint
3. Callback receives authorization code
4. Exchange code for tokens (server-side, never exposed to client)
5. If provider returns age/DOB claim, set `age_verified = true`
6. If email matches existing user, link OAuth account to existing user
7. If new user, create account, prompt for display_name + DOB (honor system) if not provided by OAuth

### 6.3 Age Gating

- Minimum age: 18
- OAuth providers checked for age claims first
- Fallback: self-declared date_of_birth
- All strain-related endpoints return 403 if `age_verified = false` AND DOB indicates < 18
- Generic warning displayed on landing page and app launch screen

### 6.4 Authorization Middleware (Axum)

```rust
// Pseudo-code layers applied to routes:
.route("/strains", get(list_strains))
    .layer(require_auth)           // Must be logged in
    .layer(require_age_verified)   // Must be 18+

.route("/vetting/queue", get(vetting_queue))
    .layer(require_auth)
    .layer(require_role(Role::Vetter))  // Must be vetter+
```

---

## 7. Offline Sync Strategy (Android)

### 7.1 Local SQLite Schema (Room)

The Android app maintains a subset of the server data in Room (SQLite):

| Table | Content | Sync Strategy |
|---|---|---|
| `strains` | Public catalog metadata (no description body, no comments) | Full sync on install, incremental via `updated_at` |
| `terpenes` | Full terpene list | Sync once, rarely changes |
| `effects` | Full effects list | Sync once, rarely changes |
| `strain_terpenes` | Join table | Included in catalog sync |
| `strain_effects` | Join table | Included in catalog sync |
| `private_strains` | User's private vault (full data) | Full sync, user-owned |
| `saved_strains` | Bookmarks | Included in vault sync |
| `ratings` | User's own ratings | Included in vault sync |
| `thumbnail_cache` | Local file paths for downloaded thumbnails | LRU cache, max ~200 thumbnails |

### 7.2 What's NOT Available Offline

- Comments (too large, rapidly changing, low offline value)
- Review photos (too large, bandwidth cost)
- Vetting queue (requires live data)
- Notifications (requires live data)
- Similar strains computation (server-side algorithm)

### 7.3 Sync Flow

```
[Android App Start]
    |
    v
[Check last_sync_timestamp in DataStore]
    |
    v
[GET /api/v1/sync/catalog?since={timestamp}]  --> incremental metadata
[GET /api/v1/sync/vault?since={timestamp}]    --> private + saved strains
[GET /api/v1/sync/ratings?since={timestamp}]  --> user's ratings
    |
    v
[Upsert into Room DB]
    |
    v
[Background: Download thumbnails for saved strains via WorkManager]
    |
    v
[Done -- app fully functional offline]
```

### 7.4 Conflict Resolution

- Server is always the source of truth for public data
- Last-write-wins for vault data (client timestamp)
- If offline edits conflict: server version wins, client changes are NOT lost -- they're stored in a `pending_changes` table and the user is notified to review on next online session

---

## 8. Photo Handling

### 8.1 Upload Flow

```
[Client] --POST multipart--> [Rust API]
                                |
                                v
                        [Validate: size <= 1MB (review) or 10MB (primary)]
                        [Validate: image/jpeg, image/png, image/webp]
                        [Strip EXIF data with kamadak-exif crate]
                                |
                                v
                        [Generate thumbnail: 300x300 WebP, quality 80%]
                        [Generate full-size: max 1920px width, WebP, quality 85%]
                                |
                                v
                        [Upload both to MinIO]
                        [Store keys in strain_photos or private_strain_photos]
                                |
                                v
                        [Return photo metadata + presigned GET URL (5-min expiry)]
```

### 8.2 Serving Photos

- The API never streams photos directly -- it returns a 302 redirect to a presigned MinIO URL
- Presigned URLs expire after 5 minutes
- This keeps photo traffic off the Rust API server entirely
- Caddy can optionally cache MinIO responses for frequently-accessed thumbnails

### 8.3 Privacy

- EXIF data (GPS coordinates, camera model, timestamp) is stripped on upload
- Filenames are replaced with UUIDs
- No user metadata is attached to public strain primary photos
- Thumbnails are generated server-side and stored alongside originals

---

## 9. Similar Strains Algorithm

### 9.1 Weighted Scoring

```
similarity_score = (terpene_overlap * 0.40)
                 + (effect_overlap  * 0.30)
                 + (type_match       * 0.15)
                 + (collaborative    * 0.15)
```

### 9.2 Terpene Overlap (Jaccard Index)

```
terpene_overlap = |A_terpenes ∩ B_terpenes| / |A_terpenes ∪ B_terpenes|
```
Ranges 0.0 to 1.0. Two strains sharing 3 out of 5 total distinct terpenes = 0.6.

### 9.3 Effect Overlap

Same Jaccard calculation on effect tags.

### 9.4 Type Match

```
type_match = 1.0 if same type, 0.5 if one is hybrid, 0.0 if sativa vs indica
```

### 9.5 Collaborative Filtering

Simple item-based approach:
1. Find users who rated THIS strain highly (> 3)
2. Find other strains those users rated highly
3. Score = average rating of those other strains by the overlapping users

This is computed lazily (on request) and cached for 1 hour. The cache is invalidated when new ratings are added.

### 9.6 Implementation

A PostgreSQL function or a materialized view refreshed periodically. For thousands of strains, computing on-the-fly is feasible. If performance degrades, pre-compute a `strain_similarities` table via a background job.

---

## 10. Vetting Workflow (Post-Edit Model)

### 10.1 Edit Flow

```
[User edits public strain]
    |
    v
[System snapshots old_data as JSONB]
    |
    v
[Public strain is UPDATED immediately with new values]
    |
    v
[strain_revisions row created: status='pending', old_data, new_data]
    |
    v
[Notification sent to all vetters]
    |
    v
[Vetter reviews revision in /vetting/queue]
    |
    +---> [Approve] --> status='approved', vetted_by set, vetted_at set
    |
    +---> [Reject]  --> status='rejected', public strain ROLLED BACK to old_data
                        rejection_reason recorded
                        notification sent to original editor
```

### 10.2 Abuse Prevention

- Users cannot edit the same strain more than once every 24 hours (prevents spam)
- Users with > 3 rejected revisions in 30 days are temporarily restricted from editing public strains
- Admins can bypass all restrictions

---

## 11. Security Measures

| Concern | Implementation |
|---|---|
| Password storage | argon2id (memory: 64MB, iterations: 3, parallelism: 4) |
| Token storage (server) | SHA-256 hash of refresh token in DB; JWT access tokens never stored |
| Token storage (Android) | Android EncryptedSharedPreferences (AES-256-GCM, hardware-backed) |
| Token storage (Web) | httpOnly + secure + sameSite=strict cookie for refresh token; access token in memory only |
| Transport | TLS 1.3 via Caddy (Let's Encrypt) |
| Certificate pinning (Android) | SHA-256 pin of Let's Encrypt intermediate cert |
| SQL injection | sqlx compile-time checked queries -- impossible by construction |
| XSS | Svelte auto-escapes; Content-Security-Policy header |
| CSRF | SameSite=strict cookie + CORS allowlist + per-request CSRF token for state-changing operations |
| Rate limiting | Per-IP and per-user token bucket (Arc<AtomicU64> in Axum layer) |
| File upload | Validate content-type, strip EXIF, enforce size limits, scan for polyglot attacks |
| Database encryption | PostgreSQL TDE or filesystem-level LUKS encryption on the VPS |
| Secrets management | `.env` file (dev), Docker secrets or vault (prod), never committed |
| API key / secret rotation | OAuth client secrets rotatable via provider dashboards |

---

## 12. Deployment

### 12.1 Development (Home Desktop/Server)

```bash
docker compose -f docker-compose.dev.yml up -d
```

Services:
- `api`: Rust binary with hot-reload via `cargo watch`
- `db`: PostgreSQL 16, port 5432 exposed to host for debugging
- `minio`: MinIO server + MinIO Client for bucket setup
- `web`: SvelteKit dev server with HMR (port 5173)

### 12.2 Production (VPS)

Recommended VPS: **Hetzner CX41** (4 vCPU, 8 GB RAM, 160 GB SSD, ~$15/mo) or **DigitalOcean Droplet** equivalent.

```bash
docker compose -f docker-compose.prod.yml up -d
```

Services:
- `caddy`: TLS termination, reverse proxy to API, static file serving for SvelteKit build
- `api`: Rust binary, built with `--release`, no ports exposed (internal Docker network only)
- `db`: PostgreSQL 16, persistent volume, daily pg_dump cron
- `minio`: MinIO server, persistent volume

### 12.3 Backup Strategy

- PostgreSQL: `pg_dump` to MinIO bucket nightly, retained 30 days
- MinIO: `mc mirror` to a second MinIO instance or external S3 bucket weekly
- Docker volumes: bind-mounted to host directory, included in host backup

### 12.4 Monitoring

- Healthcheck endpoints on all services
- Docker healthchecks with auto-restart
- Optional: Prometheus + Grafana for metrics (post-launch)

---

## 13. Development Roadmap

### Phase 1: Foundation (Weeks 1-3)
- [ ] Project scaffolding: Rust/Axum API, SvelteKit web, Kotlin Android skeleton
- [ ] Docker Compose setup (dev + prod)
- [ ] Database schema migration system (sqlx migrate)
- [ ] Seed data: terpenes, effects
- [ ] Authentication: register, login, JWT, OAuth (Google first)
- [ ] Age verification logic

### Phase 2: Core Strain Catalog (Weeks 4-6)
- [ ] Public strains CRUD (admin-only initially)
- [ ] Search + filter + pagination
- [ ] Strain detail page (web)
- [ ] Terpene & effect assignment
- [ ] Photo upload + EXIF stripping + thumbnail generation

### Phase 3: User Features (Weeks 7-9)
- [ ] Private vault (create, edit, delete private strains)
- [ ] Push private strain to public (creates anonymous public copy)
- [ ] Save public strains to vault
- [ ] Star ratings (1-5) with denormalized aggregation
- [ ] Comments (threaded) + upvote/downvote
- [ ] User profiles (display name, stats)

### Phase 4: Vetting & Version History (Weeks 10-11)
- [ ] Public strain edit -> pending revision
- [ ] Vetter queue + approve/reject
- [ ] Rollback on rejection
- [ ] Version history view
- [ ] Vetter promotion by admin

### Phase 5: Similar Strains & Discovery (Week 12)
- [ ] Jaccard similarity on terpenes + effects
- [ ] Collaborative filtering
- [ ] Cached recommendations
- [ ] "Similar strains" section on strain detail page

### Phase 6: Android App (Weeks 13-16)
- [ ] Jetpack Compose UI (Material 3)
- [ ] Room database + DataStore
- [ ] Offline catalog sync
- [ ] Vault management (offline-capable)
- [ ] Photo capture (CameraX) + gallery picker
- [ ] Background sync with WorkManager
- [ ] Certificate pinning

### Phase 7: Polish & Launch (Weeks 17-18)
- [ ] OAuth: Facebook, Microsoft, Apple
- [ ] Notifications (in-app + web)
- [ ] Rate limiting + abuse prevention
- [ ] Landing page (cpntodd branding)
- [ ] Age gate wall
- [ ] Generic legal disclaimer
- [ ] Production deployment

---

## 14. Resolved Design Decisions

1. **Terpene icons:** Custom SVG icons (not emoji) to avoid any potential copyright or platform rendering issues.
2. **Comment format:** Markdown with server-side sanitization (no raw HTML, no images, allow bold/italic/links/lists).
3. **Strain name deduplication:** Public catalog enforces unique strain names (case-insensitive). If a user attempts to push a strain with an existing name, the API returns a 409 Conflict with the existing strain ID, prompting: "This strain already exists. Is this the same strain?" If yes, the user's rating, comments, and photos are attached to the existing public strain. If no, the user must choose a different name.
4. **Notifications:** Per-type opt-in/opt-out via `user_notification_settings` table. Default: all enabled. Types: `comment_reply`, `comment_vote`, `vetting_action`, `strain_approved`, `strain_rejected`.
5. **Strain merging:** Admins can merge two public strains. The target strain absorbs all ratings, comments, and photos from the source strain. The source strain is soft-deleted (redirects to target). Merge history is logged.
6. **Data export:** GDPR-compliant data export endpoint (`GET /api/v1/account/export`) returns a JSON archive of all user data: profile, vault strains, ratings, comments, uploaded photos (as presigned URLs). Generated asynchronously; user receives a notification when ready.
7. **Analytics:** No third-party trackers. Server-side analytics only: page view counters (anonymous, no IP logging), search query trends (aggregated), strain popularity rankings. Admin dashboard displays these. All analytics are derived from server logs and database aggregates -- nothing is sent to external services.

---

## 15. Legal Disclaimer (Draft)

> **Notice:** PotSpot is a community-driven informational resource. Cannabis legality varies by jurisdiction. It is your responsibility to understand and comply with local laws. PotSpot does not facilitate, encourage, or enable the purchase, sale, or distribution of cannabis. All strain data is user-contributed and not verified for accuracy. Users must be 18 years of age or older. By using PotSpot, you acknowledge that you have read and understood this disclaimer.

---

*Design version: 1.0 -- 2026-07-29*
