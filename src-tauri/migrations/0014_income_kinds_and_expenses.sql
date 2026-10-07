-- The money of «Mi institución» (ADR-026). Each income line says what kind it is and whether its amount is per
-- month or per year (the code turns it into a year); what the institution spends gets its own list. The stay
-- fees of the roster are computed, never stored here.

ALTER TABLE income_source RENAME COLUMN annual_amount_mxn TO amount_mxn;
ALTER TABLE income_source ADD COLUMN period TEXT NOT NULL DEFAULT 'annual' CHECK (period IN ('monthly','annual'));
ALTER TABLE income_source ADD COLUMN kind TEXT NOT NULL DEFAULT 'other'
  CHECK (kind IN ('fee_estimate','recurring_donor','occasional_donation','project_grant','other'));
-- what was written before as «Cuotas…» was the fees of the people served, written by hand
UPDATE income_source SET kind = 'fee_estimate' WHERE label LIKE '%cuota%';

CREATE TABLE expense_item (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES institution_profile(id) ON DELETE CASCADE,
  label TEXT NOT NULL,             -- "Alimentos", "Luz y agua", "Medicinas"
  amount_mxn INTEGER CHECK (amount_mxn IS NULL OR amount_mxn >= 0),
  period TEXT NOT NULL DEFAULT 'annual' CHECK (period IN ('monthly','annual')),
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);
