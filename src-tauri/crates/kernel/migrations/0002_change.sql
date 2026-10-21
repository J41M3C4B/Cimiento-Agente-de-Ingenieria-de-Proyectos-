-- Preparing and applying a change (K4), its history with its origin (K5) and its fact (K8), for any entity of any
-- module. They are written together, in one transaction, or not at all.

-- Every change applied: one line per fingerprint, so a fingerprint is applied once.
CREATE TABLE kernel_change (
    id TEXT PRIMARY KEY,
    fingerprint TEXT NOT NULL UNIQUE,
    module TEXT NOT NULL,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    base_version INTEGER NOT NULL,
    version INTEGER NOT NULL,
    origin TEXT NOT NULL CHECK (origin IN ('person', 'document', 'calculated', 'task')),
    account_id TEXT,                 -- the person, or who accepted what a task proposed
    document_id TEXT,
    page INTEGER,
    rule TEXT,                       -- the code of the calculation
    task_id TEXT,
    at TEXT NOT NULL
);
CREATE INDEX kernel_change_by_entity ON kernel_change(module, entity, entity_id);

-- What changed, field by field. A protected field says that it changed, never its value.
CREATE TABLE kernel_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    change_id TEXT NOT NULL REFERENCES kernel_change(id),
    field TEXT NOT NULL,
    before TEXT,                     -- JSON; NULL when protected
    after TEXT,                      -- JSON; NULL when protected
    protected INTEGER NOT NULL DEFAULT 0 CHECK (protected IN (0, 1))
);
CREATE INDEX kernel_history_by_change ON kernel_history(change_id);

-- The history only grows.
CREATE TRIGGER kernel_history_no_update BEFORE UPDATE ON kernel_history
BEGIN SELECT RAISE(ABORT, 'the history only grows'); END;
CREATE TRIGGER kernel_history_no_delete BEFORE DELETE ON kernel_history
BEGIN SELECT RAISE(ABORT, 'the history only grows'); END;
CREATE TRIGGER kernel_change_no_update BEFORE UPDATE ON kernel_change
BEGIN SELECT RAISE(ABORT, 'the history only grows'); END;
CREATE TRIGGER kernel_change_no_delete BEFORE DELETE ON kernel_change
BEGIN SELECT RAISE(ABORT, 'the history only grows'); END;

-- The facts, for whoever reacts to them (K8): ids and a code, never data of a person.
CREATE TABLE kernel_outbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    change_id TEXT NOT NULL REFERENCES kernel_change(id),
    fact TEXT NOT NULL,              -- `muestra.nota_guardada`
    module TEXT NOT NULL,
    entity TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    at TEXT NOT NULL,
    delivered_at TEXT
);
