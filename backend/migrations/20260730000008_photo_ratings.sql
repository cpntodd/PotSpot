-- Add photo ratings support
ALTER TABLE strain_photos
  ADD COLUMN IF NOT EXISTS average_rating DECIMAL(3,2),
  ADD COLUMN IF NOT EXISTS rating_count INTEGER NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS photo_ratings (
    photo_id UUID NOT NULL REFERENCES strain_photos(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (photo_id, user_id)
);
