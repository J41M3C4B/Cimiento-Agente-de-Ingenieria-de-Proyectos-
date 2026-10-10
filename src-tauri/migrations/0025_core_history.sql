-- Where each datum of the institution comes from (audit D6) and its light history (ADR-033 §4). Owner: the core.
-- A field is an id of the catalog of `core::institution::forms` (`institution.legal_name`).

-- the origin of each datum written today: who confirmed it and when. A datum written before this table, without a
-- row, was written by a person (no document nor AI wrote the institution before).
CREATE TABLE core_field (
  field TEXT PRIMARY KEY,
  origin TEXT NOT NULL CHECK (origin IN ('user', 'document', 'ai_assumption', 'computed')),
  source_ref TEXT,                    -- a document and its page, when it came from one
  confirmed_at TEXT,
  confirmed_by TEXT,                  -- the account that wrote or accepted it
  updated_at TEXT NOT NULL
);

-- one line per change of a datum, only added. With it, how a datum was on any day can be told. What is the
-- institution's own and protected (address, RFC, folios, keys) says only that it changed: value_json stays empty.
CREATE TABLE core_change (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  field TEXT NOT NULL,
  value_json TEXT,                    -- the new value; NULL when it was emptied or when it is protected
  protected INTEGER NOT NULL DEFAULT 0 CHECK (protected IN (0, 1)),
  origin TEXT NOT NULL CHECK (origin IN ('user', 'document', 'ai_assumption', 'computed')),
  changed_at TEXT NOT NULL,
  changed_by TEXT
);

CREATE INDEX core_change_field ON core_change (field, changed_at);

CREATE TRIGGER core_change_no_update BEFORE UPDATE ON core_change
BEGIN
  SELECT RAISE(ABORT, 'core_change only grows');
END;

CREATE TRIGGER core_change_no_delete BEFORE DELETE ON core_change
BEGIN
  SELECT RAISE(ABORT, 'core_change only grows');
END;
