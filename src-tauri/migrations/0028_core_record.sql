-- The record of the institution (ADR-033 §5): what it must have, with its state; the file is optional ("we have the
-- deed but did not upload it" is useful too). The types live in code (core::records); a type with no row is
-- missing. Owner: the core.
-- The clean text of a file lives in `document` (kind 'internal'); of the original, only where it is on this computer.
CREATE TABLE core_record (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,                 -- a type of core::records::TYPES, or 'other'
  custom_name TEXT,                   -- the name the person gives an 'other'
  status TEXT NOT NULL CHECK (status IN ('have', 'missing', 'in_progress', 'not_applicable')),
  document_id TEXT REFERENCES document(id) ON DELETE SET NULL,
  original_path TEXT,                 -- where the original file is on this computer; never its content
  original_name TEXT,                 -- its file name, as the person sees it
  issued_on TEXT,                     -- YYYY-MM-DD
  valid_until TEXT,                   -- YYYY-MM-DD, as written in the document
  reference TEXT,                     -- folio or number of the document: the institution's own, never to the AI
  notes TEXT,
  site_id TEXT,                       -- a site of the institution, for several sites later (ADR-033 §6)
  origin TEXT NOT NULL DEFAULT 'user' CHECK (origin IN ('user', 'document', 'ai_assumption', 'computed')),
  source_ref TEXT,
  confirmed_at TEXT,
  confirmed_by TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- one row per type and site; 'other' as many as the institution names
CREATE UNIQUE INDEX core_record_one_per_type ON core_record (type, coalesce(site_id, '')) WHERE type <> 'other';
