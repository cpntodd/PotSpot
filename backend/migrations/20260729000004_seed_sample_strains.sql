-- PotSpot: Sample strain data for development/testing
-- Run against the development database:
--   psql -U potspot -d potspot -f backend/migrations/20260729000004_seed_sample_strains.sql

-- ============================================================================
-- Sample Strains
-- ============================================================================

-- Blue Dream (Hybrid)
INSERT INTO public_strains (id, name, type, thc_percentage, cbd_percentage, description, color, smell, flavor, breeder, lineage, growing_difficulty, flowering_time_days)
VALUES (
    'a0000001-0000-0000-0000-000000000001',
    'Blue Dream',
    'hybrid',
    18.00,
    0.10,
    'Blue Dream is a sativa-dominant hybrid marijuana strain made by crossing Blueberry with Haze. This strain produces a balanced high, along with effects like cerebral stimulation and full-body relaxation. Blue Dream is a popular strain known for its sweet berry aroma and gentle, uplifting effects.',
    'Light green with blue hues and orange pistils',
    'Sweet berry, blueberry, earthy',
    'Berry, sweet, herbal',
    'DJ Short',
    'Blueberry x Haze',
    'moderate',
    70
);

-- OG Kush (Hybrid)
INSERT INTO public_strains (id, name, type, thc_percentage, cbd_percentage, description, color, smell, flavor, breeder, lineage, growing_difficulty, flowering_time_days)
VALUES (
    'a0000001-0000-0000-0000-000000000002',
    'OG Kush',
    'hybrid',
    23.00,
    0.20,
    'OG Kush is a legendary strain known for its strong, euphoric high and distinctive earthy, pine aroma. A staple in the cannabis community, OG Kush provides heavy relaxation and is best suited for evening use.',
    'Forest green with orange hairs and frosty trichomes',
    'Earthy, pine, sour lemon, woody',
    'Earthy, pine, lemon, spicy',
    'Unknown (Florida origin)',
    'Chemdawg x Lemon Thai x Hindu Kush',
    'moderate',
    60
);

-- Sour Diesel (Sativa)
INSERT INTO public_strains (id, name, type, thc_percentage, cbd_percentage, description, color, smell, flavor, breeder, lineage, growing_difficulty, flowering_time_days)
VALUES (
    'a0000001-0000-0000-0000-000000000003',
    'Sour Diesel',
    'sativa',
    20.00,
    0.10,
    'Sour Diesel, also known as Sour D, is an invigorating sativa-dominant strain named after its pungent, diesel-like aroma. It provides a dreamy, cerebral high that is energizing and long-lasting, making it ideal for daytime use.',
    'Pale green with yellow accents and orange hairs',
    'Diesel, pungent, earthy, herbal',
    'Diesel, sour, citrus, herbal',
    'Unknown (East Coast origin)',
    'Chemdawg x Super Skunk',
    'moderate',
    77
);

-- Granddaddy Purple (Indica)
INSERT INTO public_strains (id, name, type, thc_percentage, cbd_percentage, description, color, smell, flavor, breeder, lineage, growing_difficulty, flowering_time_days)
VALUES (
    'a0000001-0000-0000-0000-000000000004',
    'Granddaddy Purple',
    'indica',
    19.00,
    0.10,
    'Granddaddy Purple (GDP) is a famous indica strain created by crossing Purple Urkle with Big Bud. Known for its deep purple buds and sweet grape aroma, GDP delivers a powerful blend of cerebral euphoria and physical relaxation.',
    'Deep purple with orange hairs and white crystal trichomes',
    'Grape, berry, sweet, earthy',
    'Grape, berry, sweet, earthy',
    'Ken Estes',
    'Purple Urkle x Big Bud',
    'easy',
    56
);

-- Gelato (Hybrid)
INSERT INTO public_strains (id, name, type, thc_percentage, cbd_percentage, description, color, smell, flavor, breeder, lineage, growing_difficulty, flowering_time_days)
VALUES (
    'a0000001-0000-0000-0000-000000000005',
    'Gelato',
    'hybrid',
    21.00,
    0.10,
    'Gelato is a well-balanced hybrid strain from Cookie Fam Genetics. Known for its dessert-like aroma and potent effects, Gelato provides a euphoric high accompanied by relaxing body effects. The flavor profile is sweet, creamy, and fruity.',
    'Dark purple and green with orange hairs',
    'Sweet, fruity, creamy, lavender',
    'Sweet, creamy, berry, citrus',
    'Cookie Fam Genetics',
    'Sunset Sherbet x Thin Mint GSC',
    'difficult',
    65
);

-- ============================================================================
-- Terpene Assignments
-- ============================================================================

-- Blue Dream terpenes
INSERT INTO strain_terpenes (strain_id, terpene_id) VALUES
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM terpenes WHERE name = 'Myrcene')),
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM terpenes WHERE name = 'Pinene')),
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM terpenes WHERE name = 'Caryophyllene'));

-- OG Kush terpenes
INSERT INTO strain_terpenes (strain_id, terpene_id) VALUES
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM terpenes WHERE name = 'Myrcene')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM terpenes WHERE name = 'Limonene')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM terpenes WHERE name = 'Caryophyllene')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM terpenes WHERE name = 'Humulene'));

-- Sour Diesel terpenes
INSERT INTO strain_terpenes (strain_id, terpene_id) VALUES
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM terpenes WHERE name = 'Limonene')),
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM terpenes WHERE name = 'Myrcene')),
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM terpenes WHERE name = 'Caryophyllene'));

-- Granddaddy Purple terpenes
INSERT INTO strain_terpenes (strain_id, terpene_id) VALUES
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM terpenes WHERE name = 'Myrcene')),
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM terpenes WHERE name = 'Pinene')),
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM terpenes WHERE name = 'Linalool'));

-- Gelato terpenes
INSERT INTO strain_terpenes (strain_id, terpene_id) VALUES
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM terpenes WHERE name = 'Limonene')),
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM terpenes WHERE name = 'Caryophyllene')),
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM terpenes WHERE name = 'Linalool')),
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM terpenes WHERE name = 'Humulene'));

-- ============================================================================
-- Effect Assignments
-- ============================================================================

-- Blue Dream effects
INSERT INTO strain_effects (strain_id, effect_id) VALUES
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM effects WHERE name = 'Relaxed')),
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM effects WHERE name = 'Happy')),
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM effects WHERE name = 'Uplifted')),
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM effects WHERE name = 'Creative')),
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM effects WHERE name = 'Stress Relief')),
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM effects WHERE name = 'Anxiety Relief')),
    ('a0000001-0000-0000-0000-000000000001', (SELECT id FROM effects WHERE name = 'Depression Relief'));

-- OG Kush effects
INSERT INTO strain_effects (strain_id, effect_id) VALUES
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM effects WHERE name = 'Relaxed')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM effects WHERE name = 'Euphoric')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM effects WHERE name = 'Happy')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM effects WHERE name = 'Sleepy')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM effects WHERE name = 'Pain Relief')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM effects WHERE name = 'Stress Relief')),
    ('a0000001-0000-0000-0000-000000000002', (SELECT id FROM effects WHERE name = 'Insomnia Relief'));

-- Sour Diesel effects
INSERT INTO strain_effects (strain_id, effect_id) VALUES
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM effects WHERE name = 'Energetic')),
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM effects WHERE name = 'Uplifted')),
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM effects WHERE name = 'Creative')),
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM effects WHERE name = 'Focused')),
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM effects WHERE name = 'Stress Relief')),
    ('a0000001-0000-0000-0000-000000000003', (SELECT id FROM effects WHERE name = 'Depression Relief'));

-- Granddaddy Purple effects
INSERT INTO strain_effects (strain_id, effect_id) VALUES
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM effects WHERE name = 'Relaxed')),
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM effects WHERE name = 'Euphoric')),
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM effects WHERE name = 'Sleepy')),
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM effects WHERE name = 'Hungry')),
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM effects WHERE name = 'Pain Relief')),
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM effects WHERE name = 'Insomnia Relief')),
    ('a0000001-0000-0000-0000-000000000004', (SELECT id FROM effects WHERE name = 'Muscle Spasm Relief'));

-- Gelato effects
INSERT INTO strain_effects (strain_id, effect_id) VALUES
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM effects WHERE name = 'Happy')),
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM effects WHERE name = 'Euphoric')),
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM effects WHERE name = 'Relaxed')),
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM effects WHERE name = 'Uplifted')),
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM effects WHERE name = 'Anxiety Relief')),
    ('a0000001-0000-0000-0000-000000000005', (SELECT id FROM effects WHERE name = 'Stress Relief'));
