-- PotSpot: User profile enhancements
-- Migration 006: Avatar, banner, bio

ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_s3_key TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS banner_s3_key TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS bio TEXT;

COMMENT ON COLUMN users.avatar_s3_key IS 'MinIO object key for profile picture';
COMMENT ON COLUMN users.banner_s3_key IS 'MinIO object key for profile banner image';
COMMENT ON COLUMN users.bio IS 'Short user biography / about text';
