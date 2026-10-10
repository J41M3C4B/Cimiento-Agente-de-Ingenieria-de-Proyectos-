-- The institution's own and protected records (ADR-033 §3). Owner: the core. Never to the AI nor to a tool of an
-- agent; the code puts them in the documents. Who leaves the board or an account that closes is marked, not deleted;
-- a deletion for good (a mistake when writing it) follows the rules of ADR-028.

-- the board of trustees or governing body: one row per person
CREATE TABLE core_board_member (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  role TEXT NOT NULL,                 -- a code of core::institution::catalog::BOARD_ROLES
  role_other TEXT,                    -- the name of the role when it is «other»
  since TEXT,                         -- YYYY-MM-DD
  active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  hidden INTEGER NOT NULL DEFAULT 0 CHECK (hidden IN (0, 1)),   -- while a deletion waits for the administrator
  origin TEXT NOT NULL DEFAULT 'user' CHECK (origin IN ('user', 'document', 'ai_assumption', 'computed')),
  source_ref TEXT,
  confirmed_at TEXT,
  confirmed_by TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- the bank accounts of the institution (not those of its staff: hr_person.clabe)
CREATE TABLE core_bank_account (
  id TEXT PRIMARY KEY,
  clabe TEXT NOT NULL,                -- 18 digits, checked with its check digit
  bank TEXT,
  purpose TEXT NOT NULL,              -- JSON list of codes: donations, operations
  in_documents INTEGER NOT NULL DEFAULT 0 CHECK (in_documents IN (0, 1)),
  active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  hidden INTEGER NOT NULL DEFAULT 0 CHECK (hidden IN (0, 1)),
  origin TEXT NOT NULL DEFAULT 'user' CHECK (origin IN ('user', 'document', 'ai_assumption', 'computed')),
  source_ref TEXT,
  confirmed_at TEXT,
  confirmed_by TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX core_bank_account_clabe ON core_bank_account (clabe);

-- a deletion of a board member or of an account may wait for the administrator too: the list of kinds grows, and
-- SQLite only changes a CHECK by building the table again (same columns, same rows)
CREATE TABLE access_request_new (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN ('document','hr_person','beneficiary','project','roster_field','hr_field','board_member','bank_account')),
  target_id TEXT NOT NULL,
  target_label TEXT NOT NULL,
  requested_by TEXT NOT NULL REFERENCES app_user(id),
  requested_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected')),
  resolved_by TEXT REFERENCES app_user(id),
  resolved_at TEXT
);
INSERT INTO access_request_new SELECT id, kind, target_id, target_label, requested_by, requested_at, status, resolved_by, resolved_at FROM access_request;
DROP TABLE access_request;
ALTER TABLE access_request_new RENAME TO access_request;
CREATE UNIQUE INDEX access_request_pending ON access_request(kind, target_id) WHERE status = 'pending';
