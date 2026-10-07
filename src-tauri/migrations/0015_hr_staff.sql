-- The staff module (ADR-027): the first base of a future HR module, in its own tables (prefix hr_), apart from
-- the profile and from the roster of the people served. Only anonymous aggregates leave it (`hr::api`).
-- The staff records of the old roster are moved here by code right after this file (`storage::migrations`).

-- The institution's own modalities; the built-in ones live in code. Each one follows the rules of a built-in one.
CREATE TABLE hr_modality (
  code TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  behaves_as TEXT NOT NULL,
  created_at TEXT NOT NULL
);

-- The positions (the seat, not the person).
CREATE TABLE hr_position (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  area TEXT,
  duties TEXT,
  default_modality TEXT,
  default_schedule TEXT,
  reference_pay_mxn INTEGER CHECK (reference_pay_mxn IS NULL OR reference_pay_mxn >= 0),
  authorized_seats INTEGER CHECK (authorized_seats IS NULL OR authorized_seats >= 0),
  reports_to TEXT REFERENCES hr_position(id) ON DELETE SET NULL,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX hr_position_title ON hr_position(lower(title));

-- A person: personal, contact, tax and bank data. Personal data on purpose: it never leaves this computer.
CREATE TABLE hr_person (
  id TEXT PRIMARY KEY,
  first_names TEXT NOT NULL,
  last_name_1 TEXT,
  last_name_2 TEXT,
  birth_date TEXT,
  sex TEXT,
  curp TEXT,
  marital_status TEXT,
  nationality TEXT,
  education TEXT,
  professional_license TEXT,
  address_street TEXT,
  address_number TEXT,
  address_neighborhood TEXT,
  address_municipality TEXT,
  address_state TEXT,
  address_zip TEXT,
  phone TEXT,
  email TEXT,
  bank TEXT,
  clabe TEXT,
  rfc TEXT,
  nss TEXT,
  tax_regime TEXT,
  tax_zip TEXT,
  infonavit_credit INTEGER,
  extra TEXT NOT NULL DEFAULT '{}',   -- the institution's own fields, by key (JSON)
  account_id TEXT,                    -- the app account of this person, once access profiles exist
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- The job of a person. Today one per person; the history of jobs is the next step of the module.
CREATE TABLE hr_job (
  id TEXT PRIMARY KEY,
  person_id TEXT NOT NULL REFERENCES hr_person(id) ON DELETE CASCADE,
  position_id TEXT REFERENCES hr_position(id),
  modality TEXT NOT NULL,
  start_date TEXT,
  start_date_approx INTEGER NOT NULL DEFAULT 0,   -- 1: only the year was known (moved from the old roster)
  end_date TEXT,
  schedule TEXT,
  shift TEXT,
  work_days TEXT NOT NULL DEFAULT '[]',
  weekly_hours INTEGER,
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','vacation','sick_leave','leave','left')),
  left_date TEXT,
  left_reason TEXT,
  pay_amount_mxn INTEGER CHECK (pay_amount_mxn IS NULL OR pay_amount_mxn >= 0),
  pay_period TEXT CHECK (pay_period IS NULL OR pay_period IN ('weekly','biweekly','monthly')),
  pay_method TEXT CHECK (pay_method IS NULL OR pay_method IN ('transfer','cash','check')),
  current INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX hr_job_current ON hr_job(person_id) WHERE current = 1;

CREATE TABLE hr_emergency_contact (
  id TEXT PRIMARY KEY,
  person_id TEXT NOT NULL REFERENCES hr_person(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  full_name TEXT NOT NULL,
  relationship TEXT,
  phone TEXT,
  phone_alt TEXT
);

-- The institution's own fields of the record (they used to be the roster's own fields of the staff).
CREATE TABLE hr_custom_field (
  key TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('text','select','number')),
  options TEXT NOT NULL DEFAULT '[]',
  position INTEGER NOT NULL
);

-- The profile keeps which kind of relation each anonymous line of staff is (where its money counts).
ALTER TABLE staff_group ADD COLUMN relation TEXT;
