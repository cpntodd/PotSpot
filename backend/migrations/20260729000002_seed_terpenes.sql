-- PotSpot: Seed terpene data
-- Migration 002

INSERT INTO terpenes (name, icon, description) VALUES
    ('Myrcene',      'myrcene',      'Earthy, musky, herbal. The most common terpene in cannabis. Associated with sedative and relaxing effects.'),
    ('Limonene',     'limonene',     'Citrus, lemon, orange. Uplifting and stress-relieving. Also found in citrus fruit rinds.'),
    ('Pinene',       'pinene',       'Pine, fir, rosemary. Promotes alertness and memory retention. Most abundant terpene in nature.'),
    ('Caryophyllene','caryophyllene','Peppery, spicy, woody. The only terpene known to directly activate CB2 receptors. Anti-inflammatory.'),
    ('Linalool',     'linalool',     'Floral, lavender, sweet. Calming and anti-anxiety. Also found in lavender and birch bark.'),
    ('Humulene',     'humulene',     'Hoppy, earthy, woody. Appetite suppressant and anti-inflammatory. Also found in hops.'),
    ('Terpinolene',  'terpinolene',  'Floral, herbal, piney. Complex aroma. Found in lilacs, nutmeg, and apples.'),
    ('Ocimene',      'ocimene',      'Sweet, herbal, woody. Uplifting and decongestant. Also found in mint and parsley.'),
    ('Valencene',    'valencene',    'Sweet citrus, orange. Named after Valencia oranges. Anti-inflammatory and insect repellent.'),
    ('Geraniol',     'geraniol',     'Rose, floral, sweet. Antioxidant and neuroprotective. Also found in geraniums and roses.'),
    ('Bisabolol',    'bisabolol',    'Floral, chamomile, sweet. Anti-inflammatory and skin-soothing. Also found in chamomile.'),
    ('Eucalyptol',   'eucalyptol',   'Minty, eucalyptus, cool. Decongestant and antibacterial. Also found in eucalyptus and tea tree.'),
    ('Nerolidol',    'nerolidol',    'Woody, floral, citrus. Sedative and antifungal. Also found in ginger and jasmine.'),
    ('Phytol',       'phytol',       'Grassy, green, mild. Breakdown product of chlorophyll. Mildly sedating.'),
    ('Camphene',     'camphene',     'Fir, earthy, herbal. Antioxidant and cardiovascular benefits. Also found in camphor trees.'),
    ('Phellandrene', 'phellandrene', 'Minty, citrus, peppery. Anti-inflammatory and antifungal. Also found in eucalyptus.'),
    ('Carene',       'carene',       'Sweet, pine, cedar. May promote bone health. Also found in rosemary and turpentine.'),
    ('Sabinene',     'sabinene',     'Woody, spicy, citrus. Anti-inflammatory and antibacterial. Also found in Norway spruce.'),
    ('Terpinene',    'terpinene',    'Herbal, citrus, pine. Antioxidant and antibacterial. Also found in tea tree and cardamom.'),
    ('Borneol',      'borneol',      'Minty, camphor, woody. Calming and pain-relieving. Also found in camphor and rosemary.')
ON CONFLICT (name) DO NOTHING;
