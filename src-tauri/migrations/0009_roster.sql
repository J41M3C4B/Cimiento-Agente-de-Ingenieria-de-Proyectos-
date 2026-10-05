-- The roster (ADR-020): one record per staff member and per person served, kept APART from the profile.
-- It never reaches the AI, the scanner or the documents: the profile only receives aggregates computed from it
-- (`staff_group` and `population_group` stay without any column that identifies a person).

-- The fields of the form of each kind of record. The person can add their own (free text, selector or number).
CREATE TABLE roster_field (
  entity TEXT NOT NULL CHECK (entity IN ('staff','beneficiary')),
  key TEXT NOT NULL,
  title TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('text','select','number','money','year','email','phone','yesno')),
  options TEXT NOT NULL DEFAULT '[]',        -- JSON: [{"value": "...", "label": "..."}]
  builtin INTEGER NOT NULL DEFAULT 0,        -- the app counts on these (role, pay, category...)
  locked_options INTEGER NOT NULL DEFAULT 0, -- the options are coded and cannot be edited
  required INTEGER NOT NULL DEFAULT 0,
  position INTEGER NOT NULL,
  PRIMARY KEY (entity, key)
);

-- A record: its values by field key, as JSON text.
CREATE TABLE roster_entry (
  id TEXT PRIMARY KEY,
  entity TEXT NOT NULL CHECK (entity IN ('staff','beneficiary')),
  data TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX roster_entry_entity ON roster_entry(entity);
