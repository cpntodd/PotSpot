-- PotSpot: Saved/bookmarked strains junction table
-- Migration 007: Saved strains

CREATE TABLE IF NOT EXISTS saved_strains (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    strain_id   UUID NOT NULL REFERENCES public_strains(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, strain_id)
);

CREATE INDEX IF NOT EXISTS idx_saved_strains_user ON saved_strains (user_id);
