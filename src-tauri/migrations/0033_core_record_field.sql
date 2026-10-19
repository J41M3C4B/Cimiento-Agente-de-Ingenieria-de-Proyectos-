-- Where each datum of a document of the record comes from (principle 4, ADR-038): read from its own file by a rule
-- (`document`), worked out by a rule from what the file says (`computed`, the validity of a 32-D), or written by the
-- person (`user`). A datum the person wrote is never written over by a new file; one read from a file is read again
-- from the file that replaces it. A datum written before this table has no line here and counts as the person's.
-- When the file is deleted its lines stay, like `core_field`: the datum and where it came from are kept. Owner: the core.
CREATE TABLE core_record_field (
  record_id TEXT NOT NULL REFERENCES core_record(id) ON DELETE CASCADE,
  field TEXT NOT NULL,                -- institution.record.issued_on, .valid_until or .reference
  origin TEXT NOT NULL CHECK (origin IN ('user', 'document', 'computed')),
  source_ref TEXT,                    -- the document and its page; the line it was read from unless it is protected
  updated_at TEXT NOT NULL,
  PRIMARY KEY (record_id, field)
);
