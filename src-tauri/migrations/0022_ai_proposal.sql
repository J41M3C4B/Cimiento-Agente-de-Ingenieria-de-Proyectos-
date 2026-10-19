-- What the AI proposes to change (ADR-034 §2). The AI never writes: a proposal waits here until a person accepts it
-- (then the usual save runs, with its checks and its scanner) or rejects it. Owner: the core.

CREATE TABLE ai_proposal (
  id TEXT PRIMARY KEY,
  form_id TEXT NOT NULL,              -- a form of the catalog (`institution.identity`)
  record_id TEXT,                     -- the record of a module, when the form is one of its records
  field_id TEXT NOT NULL,             -- a field of that form (`institution.mission`)
  value_json TEXT NOT NULL,           -- the value as the form takes it, already through the scanner
  origin TEXT NOT NULL CHECK (origin IN ('ai_assumption', 'document')),
  source_ref TEXT,                    -- where it comes from: a document and page, or the chat
  agent TEXT NOT NULL,                -- who proposes it (`capturist`, `reader`)
  status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected')),
  created_at TEXT NOT NULL,
  resolved_by TEXT,                   -- the account that accepted or rejected it
  resolved_at TEXT
);

CREATE INDEX ai_proposal_pending ON ai_proposal (form_id, record_id, status);
