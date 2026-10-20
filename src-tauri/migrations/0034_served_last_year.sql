-- How many people the institution served in the last year (ERP-2): asked where people come and go (community,
-- consultation, online, grants…). Where they live or spend the day, `institution_profile.served_estimate` says how
-- many are there today. Two data, so a number never changes its meaning when the ways it serves change.
ALTER TABLE institution ADD COLUMN served_last_year INTEGER CHECK (served_last_year >= 0);
