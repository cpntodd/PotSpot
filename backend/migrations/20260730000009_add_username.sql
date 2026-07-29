-- Add unique username for email-or-username login
ALTER TABLE users
  ADD COLUMN IF NOT EXISTS username text;

-- Derive initial usernames from email prefixes (guaranteed unique since email is unique)
UPDATE users SET username = split_part(email, '@', 1) WHERE username IS NULL;

-- Ensure uniqueness
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users (username) WHERE deleted_at IS NULL;

-- Make it NOT NULL going forward
ALTER TABLE users ALTER COLUMN username SET NOT NULL;
