-- Diagnosis flow state (Phase 2). Migration 0001 is never edited.

-- One open follow-up question per project at a time (so reloading the screen
-- never asks the AI again and never bills twice).
CREATE TABLE diagnosis_pending (
  project_id TEXT PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
  dimension INTEGER NOT NULL CHECK (dimension BETWEEN 1 AND 7),
  followup_index INTEGER NOT NULL CHECK (followup_index BETWEEN 1 AND 2),
  question TEXT NOT NULL,
  created_at TEXT NOT NULL
);

-- A dimension is closed when the AI says it is enough, the 2 follow-ups are used,
-- or there is no AI. The code decides which dimension comes next from this table.
CREATE TABLE diagnosis_dimension (
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  dimension INTEGER NOT NULL CHECK (dimension BETWEEN 1 AND 7),
  closed_at TEXT NOT NULL,
  PRIMARY KEY (project_id, dimension)
);

-- Going back a stage marks later work as "por revisar" instead of deleting it.
ALTER TABLE project ADD COLUMN needs_review INTEGER NOT NULL DEFAULT 0;

-- Initial request in the person's own words (can be corrected, scanned like any text).
ALTER TABLE project ADD COLUMN initial_request TEXT;
