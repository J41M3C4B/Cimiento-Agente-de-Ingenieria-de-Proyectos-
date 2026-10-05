-- The diagnosis becomes a guided conversation (ADR-017): an opening question, up to five "whys" and the root
-- cause the person confirms. It replaces the seven fixed questions; their tables (`diagnosis_answer`,
-- `diagnosis_dimension`, `diagnosis_pending`) stay as they were and are no longer written.

-- One row per message, in order. A turn of the person with no turn of the assistant after it means the AI owes a
-- reply (it failed or was interrupted): the screen offers "Reintentar" and nothing is asked twice.
CREATE TABLE conversation_turn (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES project(id) ON DELETE CASCADE,
  turn INTEGER NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('assistant','person')),
  kind TEXT NOT NULL CHECK (kind IN ('opening','why','root_proposal','root_reply')),
  level INTEGER,                        -- which "why" (1 to 5) the assistant asks, or the person answers
  text TEXT NOT NULL,                   -- already scanned: the message as the person sees it
  options_json TEXT,                    -- closed answers the assistant offered (quick replies)
  record_json TEXT,                     -- what the turn recorded: cause, quote, verified, fit, tactic
  created_at TEXT NOT NULL,
  UNIQUE (project_id, turn)
);

-- The cause the person confirmed as the heart of the project (never confirmed by silence).
CREATE TABLE conversation_root (
  project_id TEXT PRIMARY KEY REFERENCES project(id) ON DELETE CASCADE,
  text TEXT NOT NULL,
  quote TEXT,
  confirmed_at TEXT
);

-- The person confirms that the call is the right one before the project goes on.
ALTER TABLE call_reading ADD COLUMN confirmed_at TEXT;
