-- The facilities module (ADR-030): the base of a future specialized module, in its own tables (prefix fac_), apart
-- from the profile. A site is a building with its services and safety; its spaces and its equipment go in groups
-- («Baños · planta alta · 4») that count how many are in each state. The screen handles one site, the tables allow
-- several. The spaces of the old profile list are moved here by code right after this file
-- (`storage::migrations`); then the old table goes away.

CREATE TABLE fac_site (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  -- the building
  land_m2 INTEGER CHECK (land_m2 > 0),
  built_m2 INTEGER CHECK (built_m2 > 0),
  floors INTEGER CHECK (floors BETWEEN 1 AND 20),
  floor_access TEXT NOT NULL DEFAULT '[]',     -- ramp, elevator, stair_lift
  built_year INTEGER,
  tenure TEXT,                                  -- own, loan (comodato), rent, borrowed, other
  tenure_until INTEGER,                         -- year a loan or a lease ends
  tenure_documented INTEGER,                    -- deed or contract on paper: 0/1/NULL
  -- services
  water_sources TEXT NOT NULL DEFAULT '[]',     -- network, truck, well, rain
  water_shortage TEXT,                          -- never, sometimes, often
  water_storage_liters INTEGER CHECK (water_storage_liters >= 0),
  power_outages TEXT,                           -- never, sometimes, often
  gas TEXT,                                     -- lp_tank, lp_cylinders, natural, none
  drainage TEXT,                                -- sewer, septic, none
  internet INTEGER,
  -- safety and civil protection
  extinguishers INTEGER CHECK (extinguishers >= 0),
  extinguishers_current INTEGER,
  smoke_detectors INTEGER CHECK (smoke_detectors >= 0),
  marked_exits INTEGER,
  emergency_lights INTEGER,
  first_aid_kit INTEGER,
  internal_program TEXT,                        -- yes, in_progress, no
  civil_protection_opinion INTEGER,
  opinion_year INTEGER,
  drills_per_year INTEGER CHECK (drills_per_year >= 0),
  notes TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- A group of spaces of one kind on one floor. good + fair + poor + unusable may add up to less than count: the
-- rest have not been checked (not knowing is not «good»).
CREATE TABLE fac_space (
  id TEXT PRIMARY KEY,
  site_id TEXT NOT NULL REFERENCES fac_site(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  label TEXT,
  floor INTEGER NOT NULL DEFAULT 0,             -- -1 basement, 0 ground floor, 1 first floor…
  count INTEGER NOT NULL CHECK (count >= 1),
  good INTEGER NOT NULL DEFAULT 0 CHECK (good >= 0),
  fair INTEGER NOT NULL DEFAULT 0 CHECK (fair >= 0),
  poor INTEGER NOT NULL DEFAULT 0 CHECK (poor >= 0),
  unusable INTEGER NOT NULL DEFAULT 0 CHECK (unusable >= 0),
  problems TEXT NOT NULL DEFAULT '[]',
  accessible INTEGER,
  beds INTEGER CHECK (beds >= 0),               -- bedrooms and infirmary
  hospital_beds INTEGER CHECK (hospital_beds >= 0),
  grab_bars INTEGER,                            -- bathrooms
  accessible_shower INTEGER,
  notes TEXT,
  origin TEXT NOT NULL DEFAULT 'user',
  source_ref TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (good + fair + poor + unusable <= count)
);
CREATE INDEX fac_space_site ON fac_space(site_id);

CREATE TABLE fac_equipment (
  id TEXT PRIMARY KEY,
  site_id TEXT NOT NULL REFERENCES fac_site(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  label TEXT,
  count INTEGER NOT NULL CHECK (count >= 1),
  good INTEGER NOT NULL DEFAULT 0 CHECK (good >= 0),
  fair INTEGER NOT NULL DEFAULT 0 CHECK (fair >= 0),
  poor INTEGER NOT NULL DEFAULT 0 CHECK (poor >= 0),
  unusable INTEGER NOT NULL DEFAULT 0 CHECK (unusable >= 0),
  notes TEXT,
  origin TEXT NOT NULL DEFAULT 'user',
  source_ref TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (good + fair + poor + unusable <= count)
);
CREATE INDEX fac_equipment_site ON fac_equipment(site_id);
