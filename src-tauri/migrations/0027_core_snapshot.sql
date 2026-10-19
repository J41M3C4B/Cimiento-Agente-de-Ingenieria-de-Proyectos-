-- A photo per month of the figures of the institution (ADR-033 §4): what the progress charts are drawn with, without
-- keeping old records. Owner: the core (core/snapshot.rs). Only figures, never a person.
-- The photo of the current month is taken again each time the app opens; once the month is over it is fixed: a
-- past month is never changed nor deleted (triggers).
CREATE TABLE core_snapshot (
  month TEXT PRIMARY KEY,             -- YYYY-MM, in the time of this computer
  taken_at TEXT NOT NULL,
  figures_json TEXT NOT NULL          -- core::snapshot::Figures
);

CREATE TRIGGER core_snapshot_past_is_fixed BEFORE UPDATE ON core_snapshot
WHEN OLD.month < strftime('%Y-%m', 'now', 'localtime')
BEGIN
  SELECT RAISE(ABORT, 'a past month is fixed');
END;

CREATE TRIGGER core_snapshot_no_delete BEFORE DELETE ON core_snapshot
BEGIN
  SELECT RAISE(ABORT, 'core_snapshot only grows');
END;
