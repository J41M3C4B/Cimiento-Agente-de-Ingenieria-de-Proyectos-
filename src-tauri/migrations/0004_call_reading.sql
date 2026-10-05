-- Reading of a call (ADR-015): the files the person uploaded, what was read from them and how it went.
-- The text of the files lives in `document` / `document_chunk` (one fragment per page); this table holds
-- the canonical document the reading produced, with a verified quote for every item.
CREATE TABLE call_reading (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,                  -- how the person sees it
  status TEXT NOT NULL CHECK (status IN ('waiting','reading','ready','partial','failed')),
                                       -- waiting: saved, not read yet (no key, no internet, no allowance)
  note TEXT,                           -- why it is waiting, partial or failed (plain code, the UI words it)
  canonical_json TEXT,                 -- the assembled document (schemas/canonical_call.schema.json)
  report_json TEXT,                    -- counts of the assembly and of the calls; never contents
  created_at TEXT NOT NULL,
  finished_at TEXT
);

-- Files of a reading, in the order they were given (the page numbers of the package follow it).
CREATE TABLE call_reading_file (
  reading_id TEXT NOT NULL REFERENCES call_reading(id) ON DELETE CASCADE,
  document_id TEXT NOT NULL REFERENCES document(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  PRIMARY KEY (reading_id, document_id)
);
