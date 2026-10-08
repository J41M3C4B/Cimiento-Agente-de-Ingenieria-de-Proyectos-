-- The first start (ADR-031): the data of the institution that most calls filter by, the quick figures of people
-- served and staff while their records are not there yet, when the institution finished its onboarding, and when
-- each account saw its welcome.

-- where it is and what it is, legally (none of it is personal: it reaches the AI)
ALTER TABLE institution ADD COLUMN state TEXT;               -- one of the 32 states (code)
ALTER TABLE institution ADD COLUMN municipality TEXT;
ALTER TABLE institution ADD COLUMN founded_year INTEGER;
ALTER TABLE institution ADD COLUMN legal_form TEXT;          -- ac, iap, ibp, sc, abp, religious, other
ALTER TABLE institution ADD COLUMN authorized_donee TEXT;    -- yes, in_progress, no (donataria autorizada, SAT)
ALTER TABLE institution ADD COLUMN cluni TEXT;               -- yes, in_progress, no (registro federal de OSC)
-- set once, when the onboarding is finished; it never goes back
ALTER TABLE institution ADD COLUMN onboarded_at TEXT;

-- quick figures said by the person; the records of the modules win when they exist
ALTER TABLE institution_profile ADD COLUMN served_estimate INTEGER CHECK (served_estimate >= 0);
ALTER TABLE institution_profile ADD COLUMN staff_paid_estimate INTEGER CHECK (staff_paid_estimate >= 0);
ALTER TABLE institution_profile ADD COLUMN staff_volunteer_estimate INTEGER CHECK (staff_volunteer_estimate >= 0);

ALTER TABLE app_user ADD COLUMN welcomed_at TEXT;
