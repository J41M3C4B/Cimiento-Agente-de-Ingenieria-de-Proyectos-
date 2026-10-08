-- Finanzas becomes a module of its own (ADR-032). The money stops being part of the versions of the profile: what
-- the person wrote lives here, each line with its origin and confirmation. The lines of the current profile move in;
-- the old tables keep the earlier versions as history and are not written any more.

CREATE TABLE fin_income (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,             -- "Padrinos", "Colecta anual"
  kind TEXT NOT NULL DEFAULT 'other'
    CHECK (kind IN ('fee_estimate','recurring_donor','occasional_donation','project_grant','other')),
  amount_mxn INTEGER CHECK (amount_mxn IS NULL OR amount_mxn >= 0),
  period TEXT NOT NULL DEFAULT 'annual' CHECK (period IN ('monthly','annual')),
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

CREATE TABLE fin_expense (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,             -- "Alimentos", "Luz y agua", "Medicinas"
  amount_mxn INTEGER CHECK (amount_mxn IS NULL OR amount_mxn >= 0),
  period TEXT NOT NULL DEFAULT 'annual' CHECK (period IN ('monthly','annual')),
  origin TEXT NOT NULL, source_ref TEXT, confirmed_at TEXT, confirmed_by TEXT
);

-- One row: the approximate figure of what is spent in a year (the quick way to start, ADR-026).
CREATE TABLE fin_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  annual_budget_mxn INTEGER CHECK (annual_budget_mxn IS NULL OR annual_budget_mxn >= 0),
  origin TEXT NOT NULL DEFAULT 'user', confirmed_at TEXT, confirmed_by TEXT
);

INSERT INTO fin_income (id, label, kind, amount_mxn, period, origin, source_ref, confirmed_at, confirmed_by)
  SELECT id, label, kind, amount_mxn, period, origin, source_ref, confirmed_at, confirmed_by FROM income_source
  WHERE profile_id = (SELECT id FROM institution_profile ORDER BY version DESC LIMIT 1)
  ORDER BY rowid;

INSERT INTO fin_expense (id, label, amount_mxn, period, origin, source_ref, confirmed_at, confirmed_by)
  SELECT id, label, amount_mxn, period, origin, source_ref, confirmed_at, confirmed_by FROM expense_item
  WHERE profile_id = (SELECT id FROM institution_profile ORDER BY version DESC LIMIT 1)
  ORDER BY rowid;

INSERT INTO fin_settings (id, annual_budget_mxn, origin, confirmed_at)
  SELECT 1, annual_budget_mxn, 'user', confirmed_at FROM institution_profile
  ORDER BY version DESC LIMIT 1;
