-- The module of the people served (ADR-029): the base of a future specialized module, in its own tables (prefix
-- care_), apart from the profile. Only anonymous aggregates leave it (`care::api`). Health goes only as categories.
-- The people served of the old roster are moved here by code right after this file (`storage::migrations`); then
-- the roster, now empty, is dropped.

CREATE TABLE care_group (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
);
CREATE UNIQUE INDEX care_group_title ON care_group(lower(title));

CREATE TABLE care_person (
  id TEXT PRIMARY KEY,
  -- 1. identification
  first_names TEXT NOT NULL,
  last_name_1 TEXT,
  last_name_2 TEXT,
  birth_date TEXT,
  birth_date_approx INTEGER NOT NULL DEFAULT 0,
  sex TEXT,
  curp TEXT,
  origin_municipality TEXT,
  origin_state TEXT,
  indigenous_language TEXT,
  education TEXT,
  literate INTEGER,
  attends_school INTEGER,
  school_grade TEXT,
  school_lag INTEGER,
  group_id TEXT REFERENCES care_group(id) ON DELETE SET NULL,
  -- 2. admission and stay
  entry_date TEXT,
  entry_date_approx INTEGER NOT NULL DEFAULT 0,
  stay_mode TEXT,
  referred_by TEXT,
  admission_reasons TEXT NOT NULL DEFAULT '[]',
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','hospitalized','discharged','deceased')),
  status_date TEXT,
  discharge_reason TEXT,
  -- 3. care and health: categories only
  dependency TEXT,
  mobility TEXT,
  disabilities TEXT NOT NULL DEFAULT '[]',
  chronic_conditions TEXT NOT NULL DEFAULT '[]',
  continence TEXT,
  orientation TEXT,
  psych_care INTEGER,
  vaccines_up_to_date INTEGER,
  -- 4. family
  visits TEXT,
  legal_status TEXT,
  -- 5. contribution and support
  monthly_fee_mxn INTEGER CHECK (monthly_fee_mxn IS NULL OR monthly_fee_mxn >= 0),
  fee_payer TEXT,
  programs TEXT NOT NULL DEFAULT '[]',
  consent_date TEXT,
  consent_signer TEXT,
  extra TEXT NOT NULL DEFAULT '{}',
  hidden INTEGER NOT NULL DEFAULT 0,      -- hidden while a request to delete it waits (ADR-028)
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE care_contact (
  id TEXT PRIMARY KEY,
  person_id TEXT NOT NULL REFERENCES care_person(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  full_name TEXT NOT NULL,
  relationship TEXT,
  phone TEXT,
  phone_alt TEXT,
  legal_guardian INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE care_custom_field (
  key TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('text','select','number')),
  options TEXT NOT NULL DEFAULT '[]',
  position INTEGER NOT NULL,
  hidden INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE care_waitlist (
  id TEXT PRIMARY KEY,
  requested_on TEXT NOT NULL,
  name TEXT,
  phone TEXT,
  sex TEXT,
  approx_age INTEGER,
  dependency TEXT,
  reason TEXT,
  status TEXT NOT NULL DEFAULT 'waiting' CHECK (status IN ('waiting','admitted','declined','withdrawn')),
  person_id TEXT REFERENCES care_person(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
