-- PotSpot: Seed effect taxonomy
-- Migration 003

INSERT INTO effects (name, category) VALUES
    -- Positive effects
    ('Relaxed',      'positive'),
    ('Euphoric',     'positive'),
    ('Happy',        'positive'),
    ('Uplifted',     'positive'),
    ('Creative',     'positive'),
    ('Focused',      'positive'),
    ('Energetic',    'positive'),
    ('Talkative',    'positive'),
    ('Giggly',       'positive'),
    ('Hungry',       'positive'),
    ('Aroused',      'positive'),
    ('Sleepy',       'positive'),
    ('Tingly',       'positive'),

    -- Negative effects
    ('Anxious',      'negative'),
    ('Paranoid',     'negative'),
    ('Dizzy',        'negative'),
    ('Dry Mouth',    'negative'),
    ('Dry Eyes',     'negative'),
    ('Headache',     'negative'),
    ('Lethargic',    'negative'),

    -- Medical effects
    ('Pain Relief',           'medical'),
    ('Stress Relief',         'medical'),
    ('Anxiety Relief',        'medical'),
    ('Insomnia Relief',       'medical'),
    ('Appetite Stimulant',    'medical'),
    ('Anti-inflammatory',     'medical'),
    ('Muscle Spasm Relief',   'medical'),
    ('Nausea Relief',         'medical'),
    ('Depression Relief',     'medical'),
    ('PTSD Relief',           'medical'),
    ('Seizure Management',    'medical'),
    ('Glaucoma Relief',       'medical')
ON CONFLICT (name) DO NOTHING;
