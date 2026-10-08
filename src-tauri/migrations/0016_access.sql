-- Access profiles (ADR-028): the accounts of the people who use the app, the requests to delete something for good,
-- and who did what in the audit log. The screen PIN of ADR-019 retires: each person's password takes its place.

CREATE TABLE app_user (
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL,
  display_name TEXT NOT NULL,
  person_id TEXT,                       -- the staff record of the person (hr_person.id), if any
  role TEXT NOT NULL CHECK (role IN ('admin','manager')),
  active INTEGER NOT NULL DEFAULT 1,
  password_hash TEXT NOT NULL,          -- Argon2id, PHC string; never the password
  must_change_password INTEGER NOT NULL DEFAULT 0,
  failed_attempts INTEGER NOT NULL DEFAULT 0,
  locked_until INTEGER,                 -- unix seconds
  created_by TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_login_at TEXT
);
CREATE UNIQUE INDEX app_user_username ON app_user(lower(username));
CREATE UNIQUE INDEX app_user_person ON app_user(person_id) WHERE person_id IS NOT NULL;

-- A deletion asked by someone who may not delete. The thing is hidden meanwhile; approving deletes it, rejecting
-- brings it back. The same table will travel to the remote module.
CREATE TABLE access_request (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN ('document','hr_person','beneficiary','project','roster_field','hr_field')),
  target_id TEXT NOT NULL,
  target_label TEXT NOT NULL,           -- what the administrator sees (stays in this computer)
  requested_by TEXT NOT NULL REFERENCES app_user(id),
  requested_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected')),
  resolved_by TEXT REFERENCES app_user(id),
  resolved_at TEXT
);
CREATE UNIQUE INDEX access_request_pending ON access_request(kind, target_id) WHERE status = 'pending';

-- who did it (app_user.id); empty for what the app did by itself or before the accounts
ALTER TABLE audit_log ADD COLUMN actor_id TEXT;

-- hidden while a request to delete it waits
ALTER TABLE document ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
ALTER TABLE project ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
ALTER TABLE roster_entry ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
ALTER TABLE roster_field ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
ALTER TABLE hr_person ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
ALTER TABLE hr_custom_field ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0;
