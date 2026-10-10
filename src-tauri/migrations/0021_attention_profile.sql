-- The attention profile (ADR-033 §2): whom and how the institution serves, instead of a fixed kind. The kind stays,
-- now worked out from the populations, because the modules still go by it. Lists are JSON arrays of codes of
-- `core::institution::catalog`.

ALTER TABLE institution ADD COLUMN populations TEXT NOT NULL DEFAULT '[]';  -- early_childhood … older_adults
ALTER TABLE institution ADD COLUMN sex_served TEXT;                          -- women, men, all
ALTER TABLE institution ADD COLUMN modalities TEXT NOT NULL DEFAULT '[]';   -- residential, day_care, …
ALTER TABLE institution ADD COLUMN care_areas TEXT NOT NULL DEFAULT '[]';   -- care, health, disability, …

-- What the kind meant until now (the same as `catalog::attention_of_kind`); the person completes the rest.
UPDATE institution SET populations = '["older_adults"]', modalities = '["residential"]', care_areas = '["care"]'
  WHERE kind = 'elderly_home';
UPDATE institution SET populations = '["early_childhood","childhood","adolescence"]', modalities = '["residential"]',
  care_areas = '["care"]'
  WHERE kind = 'children_home';
