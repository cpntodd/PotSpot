-- PotSpot: Remove seed data ratings and test user
-- Migration 005: Clean fake data

-- Remove fake ratings (seed data that shouldn't have been included)
DELETE FROM strain_ratings
WHERE user_id = 'b0000001-0000-0000-0000-000000000001';

-- Remove test admin user
DELETE FROM users
WHERE id = 'b0000001-0000-0000-0000-000000000001';

-- Reset denormalized rating columns
UPDATE public_strains SET average_rating = NULL, rating_count = 0;
