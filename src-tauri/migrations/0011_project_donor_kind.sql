-- Who gives the support of a project: an institution, a private company or an individual donor.
-- NULL = not said (projects made before this).
ALTER TABLE project ADD COLUMN donor_kind TEXT CHECK (donor_kind IS NULL OR donor_kind IN ('institutional','private','individual'));
