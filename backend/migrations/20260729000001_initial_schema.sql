-- PotSpot: Initial database schema
-- Migration 001: Core tables

-- ============================================================================
-- Extensions
-- ============================================================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";       -- For fuzzy text search

-- ============================================================================
-- Users & Authentication
-- ============================================================================

CREATE TYPE user_role AS ENUM ('user', 'vetter', 'admin');
CREATE TYPE oauth_provider AS ENUM ('google', 'facebook', 'microsoft', 'apple');

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT,                           -- NULL if OAuth-only account
    display_name    TEXT NOT NULL,
    role            user_role NOT NULL DEFAULT 'user',
    age_verified    BOOLEAN NOT NULL DEFAULT FALSE,
    date_of_birth   DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ                     -- Soft delete
);

CREATE INDEX idx_users_email ON users (email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_role ON users (role);

CREATE TABLE user_oauth_accounts (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider        oauth_provider NOT NULL,
    provider_user_id TEXT NOT NULL,
    access_token    TEXT,                           -- Encrypted at rest
    refresh_token   TEXT,                           -- Encrypted at rest
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, provider_user_id)
);

CREATE INDEX idx_user_oauth_accounts_user ON user_oauth_accounts (user_id);

CREATE TABLE refresh_tokens (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      TEXT NOT NULL UNIQUE,           -- SHA-256 of the actual token
    device_info     TEXT,                           -- User-Agent or device identifier
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ
);

CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens (token_hash);
CREATE INDEX idx_refresh_tokens_user ON refresh_tokens (user_id);

-- ============================================================================
-- Terpenes (fixed picklist)
-- ============================================================================

CREATE TABLE terpenes (
    id              SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name            TEXT NOT NULL UNIQUE,
    icon            TEXT NOT NULL,                  -- SVG icon identifier
    description     TEXT
);

-- ============================================================================
-- Effects (fixed taxonomy)
-- ============================================================================

CREATE TABLE effects (
    id              SMALLINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name            TEXT NOT NULL UNIQUE,
    category        TEXT NOT NULL                   -- 'positive', 'negative', 'medical'
);

CREATE INDEX idx_effects_category ON effects (category);

-- ============================================================================
-- Public Strain Catalog
-- ============================================================================

CREATE TYPE strain_type AS ENUM ('sativa', 'indica', 'hybrid');
CREATE TYPE growing_difficulty AS ENUM ('easy', 'moderate', 'difficult', 'expert');

CREATE TABLE public_strains (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name                TEXT NOT NULL UNIQUE,       -- Enforced unique in catalog
    type                strain_type NOT NULL,
    thc_percentage      DECIMAL(5,2),               -- 0.00 - 100.00
    cbd_percentage      DECIMAL(5,2),
    description         TEXT,
    color               TEXT,
    smell               TEXT,
    flavor              TEXT,
    breeder             TEXT,
    lineage             TEXT,
    growing_difficulty  growing_difficulty,
    flowering_time_days SMALLINT,
    average_rating      DECIMAL(3,2),               -- Denormalized
    rating_count        INTEGER NOT NULL DEFAULT 0, -- Denormalized
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version             INTEGER NOT NULL DEFAULT 1,

    -- NOTE: No created_by / user_id column. Public strains are anonymous.
    CONSTRAINT chk_thc_range CHECK (thc_percentage >= 0.00 AND thc_percentage <= 100.00),
    CONSTRAINT chk_cbd_range CHECK (cbd_percentage >= 0.00 AND cbd_percentage <= 100.00)
);

-- Full-text search index
CREATE INDEX idx_public_strains_fts ON public_strains
    USING GIN (to_tsvector('english', coalesce(name, '') || ' ' || coalesce(description, '')));

CREATE INDEX idx_public_strains_type ON public_strains (type);
CREATE INDEX idx_public_strains_rating ON public_strains (average_rating DESC);
CREATE INDEX idx_public_strains_active ON public_strains (is_active) WHERE is_active = TRUE;

-- ============================================================================
-- Strain-Terpene Junction
-- ============================================================================

CREATE TABLE strain_terpenes (
    strain_id   UUID NOT NULL REFERENCES public_strains(id) ON DELETE CASCADE,
    terpene_id  SMALLINT NOT NULL REFERENCES terpenes(id) ON DELETE CASCADE,
    PRIMARY KEY (strain_id, terpene_id)
);

CREATE INDEX idx_strain_terpenes_terpene ON strain_terpenes (terpene_id);

-- ============================================================================
-- Strain-Effect Junction
-- ============================================================================

CREATE TABLE strain_effects (
    strain_id   UUID NOT NULL REFERENCES public_strains(id) ON DELETE CASCADE,
    effect_id   SMALLINT NOT NULL REFERENCES effects(id) ON DELETE CASCADE,
    PRIMARY KEY (strain_id, effect_id)
);

CREATE INDEX idx_strain_effects_effect ON strain_effects (effect_id);

-- ============================================================================
-- Strain Photos
-- ============================================================================

CREATE TABLE strain_photos (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    strain_id           UUID NOT NULL REFERENCES public_strains(id) ON DELETE CASCADE,
    user_id             UUID REFERENCES users(id) ON DELETE SET NULL,
    is_primary          BOOLEAN NOT NULL DEFAULT FALSE,
    s3_key              TEXT NOT NULL,
    thumbnail_s3_key    TEXT NOT NULL,
    content_type        TEXT NOT NULL,
    file_size_bytes     INTEGER NOT NULL,
    width               SMALLINT NOT NULL,
    height              SMALLINT NOT NULL,
    uploaded_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Only one primary photo per strain
CREATE UNIQUE INDEX idx_strain_photos_primary ON strain_photos (strain_id) WHERE is_primary = TRUE;
CREATE INDEX idx_strain_photos_strain ON strain_photos (strain_id);

-- ============================================================================
-- Private Strains (User Vault)
-- ============================================================================

CREATE TABLE private_strains (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_strain_id    UUID REFERENCES public_strains(id) ON DELETE SET NULL,
    name                TEXT NOT NULL,
    type                strain_type NOT NULL,
    thc_percentage      DECIMAL(5,2),
    cbd_percentage      DECIMAL(5,2),
    description         TEXT,
    color               TEXT,
    smell               TEXT,
    flavor              TEXT,
    breeder             TEXT,
    lineage             TEXT,
    growing_difficulty  growing_difficulty,
    flowering_time_days SMALLINT,
    personal_rating     SMALLINT CHECK (personal_rating >= 1 AND personal_rating <= 5),
    personal_notes      TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_private_strains_user ON private_strains (user_id);
CREATE INDEX idx_private_strains_public ON private_strains (public_strain_id);

-- ============================================================================
-- Private Strain Photos
-- ============================================================================

CREATE TABLE private_strain_photos (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    private_strain_id   UUID NOT NULL REFERENCES private_strains(id) ON DELETE CASCADE,
    is_primary          BOOLEAN NOT NULL DEFAULT FALSE,
    s3_key              TEXT NOT NULL,
    thumbnail_s3_key    TEXT NOT NULL,
    content_type        TEXT NOT NULL,
    file_size_bytes     INTEGER NOT NULL,
    width               SMALLINT NOT NULL,
    height              SMALLINT NOT NULL,
    uploaded_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_private_strain_photos_primary
    ON private_strain_photos (private_strain_id) WHERE is_primary = TRUE;

-- ============================================================================
-- Saved Strains (bookmarks from public catalog)
-- ============================================================================

CREATE TABLE user_saved_strains (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    strain_id   UUID NOT NULL REFERENCES public_strains(id) ON DELETE CASCADE,
    saved_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, strain_id)
);

-- ============================================================================
-- Strain Ratings
-- ============================================================================

CREATE TABLE strain_ratings (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    strain_id   UUID NOT NULL REFERENCES public_strains(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating      SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(strain_id, user_id)
);

CREATE INDEX idx_strain_ratings_strain ON strain_ratings (strain_id);
CREATE INDEX idx_strain_ratings_user ON strain_ratings (user_id);

-- ============================================================================
-- Strain Revisions (Version History + Vetting)
-- ============================================================================

CREATE TYPE revision_status AS ENUM ('pending', 'approved', 'rejected');

CREATE TABLE strain_revisions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    strain_id       UUID NOT NULL REFERENCES public_strains(id) ON DELETE CASCADE,
    proposed_by     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    change_summary  TEXT,
    old_data        JSONB NOT NULL,
    new_data        JSONB NOT NULL,
    status          revision_status NOT NULL DEFAULT 'pending',
    vetted_by       UUID REFERENCES users(id) ON DELETE SET NULL,
    vetted_at       TIMESTAMPTZ,
    rejection_reason TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_strain_revisions_strain ON strain_revisions (strain_id);
CREATE INDEX idx_strain_revisions_status ON strain_revisions (status) WHERE status = 'pending';
CREATE INDEX idx_strain_revisions_proposed_by ON strain_revisions (proposed_by);

-- ============================================================================
-- Comments (threaded, Reddit-style)
-- ============================================================================

CREATE TABLE comments (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    strain_id           UUID NOT NULL REFERENCES public_strains(id) ON DELETE CASCADE,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_comment_id   UUID REFERENCES comments(id) ON DELETE CASCADE,
    body                TEXT NOT NULL,
    upvotes             INTEGER NOT NULL DEFAULT 0,
    downvotes           INTEGER NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted          BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_comments_strain ON comments (strain_id, created_at);
CREATE INDEX idx_comments_parent ON comments (parent_comment_id);
CREATE INDEX idx_comments_user ON comments (user_id);

-- ============================================================================
-- Comment Votes
-- ============================================================================

CREATE TABLE comment_votes (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    comment_id  UUID NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    vote        SMALLINT NOT NULL CHECK (vote IN (1, -1)),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(comment_id, user_id)
);

CREATE INDEX idx_comment_votes_comment ON comment_votes (comment_id);

-- ============================================================================
-- Notifications
-- ============================================================================

CREATE TYPE notification_type AS ENUM (
    'comment_reply',
    'comment_vote',
    'vetting_action',
    'strain_approved',
    'strain_rejected'
);

CREATE TABLE notifications (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type        notification_type NOT NULL,
    reference_id UUID,                              -- Polymorphic reference
    message     TEXT NOT NULL,
    is_read     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_unread ON notifications (user_id, created_at DESC)
    WHERE is_read = FALSE;

-- ============================================================================
-- Notification Settings (per-type opt-in/opt-out)
-- ============================================================================

CREATE TABLE user_notification_settings (
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type   notification_type NOT NULL,
    enabled             BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (user_id, notification_type)
);

-- ============================================================================
-- Admin Audit Log
-- ============================================================================

CREATE TYPE audit_action AS ENUM (
    'user_role_change',
    'strain_deactivate',
    'strain_merge',
    'revision_override',
    'user_delete'
);

CREATE TABLE admin_audit_log (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    admin_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action      audit_action NOT NULL,
    target_id   UUID,                               -- Polymorphic reference
    details     JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- Server-Side Analytics (no third-party trackers)
-- ============================================================================

CREATE TABLE analytics_page_views (
    id          BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    path        TEXT NOT NULL,
    viewed_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE analytics_search_queries (
    id          BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    query       TEXT NOT NULL,
    results_count INTEGER NOT NULL DEFAULT 0,
    searched_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- Functions & Triggers
-- ============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_public_strains_updated_at
    BEFORE UPDATE ON public_strains
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_private_strains_updated_at
    BEFORE UPDATE ON private_strains
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_comments_updated_at
    BEFORE UPDATE ON comments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Recalculate average rating when a rating is inserted, updated, or deleted
CREATE OR REPLACE FUNCTION recalculate_strain_rating()
RETURNS TRIGGER AS $$
DECLARE
    target_strain_id UUID;
BEGIN
    IF TG_OP = 'DELETE' THEN
        target_strain_id := OLD.strain_id;
    ELSE
        target_strain_id := NEW.strain_id;
    END IF;

    UPDATE public_strains
    SET
        average_rating = (
            SELECT ROUND(AVG(rating)::numeric, 2)
            FROM strain_ratings
            WHERE strain_id = target_strain_id
        ),
        rating_count = (
            SELECT COUNT(*)
            FROM strain_ratings
            WHERE strain_id = target_strain_id
        )
    WHERE id = target_strain_id;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_strain_ratings_recalc
    AFTER INSERT OR UPDATE OR DELETE ON strain_ratings
    FOR EACH ROW EXECUTE FUNCTION recalculate_strain_rating();
